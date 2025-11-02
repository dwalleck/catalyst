# File I/O Safety Deep Dive

This guide covers safe file I/O patterns in Rust, from atomic writes to proper testing. These patterns prevent data corruption and ensure robust file handling.

## What This Guide Covers

1. **[Atomic File Writes](#1-atomic-file-writes)** - Preventing data corruption with atomic operations
2. **[Parent Directory Creation](#2-parent-directory-creation)** - Handling missing directories safely
3. **[Complete Production Pattern](#3-complete-production-pattern)** - Combining all best practices
4. **[Testing File I/O](#4-testing-file-io)** - Integration tests with tempfile crate

**Quick Reference:** See [quick-reference.md](quick-reference.md) for scannable checklists

---

## 1. Atomic File Writes

### The Problem

Writing files directly can result in data corruption if the process crashes or is interrupted mid-write. This leaves the file in a partially-written, invalid state.

```rust
// ❌ WRONG - Direct write can corrupt file if interrupted
pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(self)?;
    fs::write(path.as_ref(), json)?;  // File can be corrupted!
    Ok(())
}
```

**Failure scenarios:**
- Process killed mid-write
- Disk full during write
- I/O error after partial write
- Power loss during write

**Result:** File contains partial JSON that can't be parsed

### Solution 1: Manual Atomic Write (Temp File + Rename)

```rust
use std::fs;
use std::io::Write;
use anyhow::{Context, Result};

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)
        .context("Failed to serialize settings")?;

    // Write to temporary file first
    let temp_path = path.with_extension("tmp");
    let mut temp_file = fs::File::create(&temp_path)
        .context("Failed to create temporary file")?;

    temp_file.write_all(json.as_bytes())
        .context("Failed to write to temporary file")?;

    // Ensure data is flushed to disk
    temp_file.sync_all()
        .context("Failed to sync temporary file")?;

    // Atomic rename (POSIX guarantees atomicity)
    fs::rename(&temp_path, path)
        .context("Failed to rename temporary file")?;

    Ok(())
}
```

**Why this works:**

1. If rename succeeds, the new file is complete and valid
2. If rename fails, the old file remains unchanged
3. No intermediate state where file is partially written
4. On POSIX systems (Linux, macOS), rename is atomic even across overwrites

**Problem with manual approach:** If `fs::rename()` fails, the `.tmp` file remains as garbage.

### Solution 2: Production Pattern with NamedTempFile (Recommended)

```rust
use tempfile::NamedTempFile;
use std::io::Write;

// ✅ BEST - Automatic cleanup on any error
pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)?;

    // Create temp file in same directory (important for atomic rename)
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir)?;

    temp_file.write_all(json.as_bytes())?;
    temp_file.as_file().sync_all()?;

    // Atomic persist to final location
    temp_file.persist(path)?;

    Ok(())
}
```

**Benefits of NamedTempFile:**

1. **Automatic cleanup:** If any error occurs before `persist()`, temp file is deleted
2. **RAII pattern:** Leverages Rust's drop semantics for cleanup
3. **Atomic rename:** `persist()` uses same atomic rename as manual approach
4. **Unique names:** Generates unique temp filenames automatically
5. **Same directory:** `new_in(dir)` ensures temp file in same filesystem for atomic rename

### When to Use Atomic Writes

**Use atomic writes for:**
- ✅ Configuration files (settings.json)
- ✅ State files (databases, caches)
- ✅ Any file where corruption would break functionality
- ✅ Files that are read by other processes

**Don't need atomic writes for:**
- ❌ Log files (append-only, partial writes are acceptable)
- ❌ Temporary scratch files
- ❌ Files that are write-once, never-overwritten

**[↑ Back to Quick Reference](quick-reference.md#9-atomic-file-writes)**

---

## 2. Parent Directory Creation

### The Problem

Writing a file fails if parent directories don't exist, even if the path is valid. This is especially common when creating new configuration files in subdirectories.

```rust
// ❌ WRONG - Fails if parent directory doesn't exist
pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(self)?;
    fs::write(path.as_ref(), json)?;  // Error: No such file or directory
    Ok(())
}

// Example failure:
settings.write("config/user/settings.json")?;  // Fails if config/user/ doesn't exist
```

**Error message:**
```
Error: No such file or directory (os error 2)
```

### Solution: Create Parent Directories First

```rust
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)
        .context("Failed to serialize settings")?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create parent directories")?;
    }

    // Now write the file
    fs::write(path, json)
        .context("Failed to write settings file")?;

    Ok(())
}
```

### Why create_dir_all() is Safe

**`fs::create_dir_all()` is idempotent:**

- If directory exists, does nothing (no error)
- If parent directories exist, creates only missing ones
- Creates entire path in one call
- Returns success if directory already exists

```rust
// All of these succeed, even if directories exist:
fs::create_dir_all("/existing/path")?;      // OK
fs::create_dir_all("/new/nested/path")?;    // Creates all levels
fs::create_dir_all(".")?;                   // OK (current dir exists)
```

**[↑ Back to Quick Reference](quick-reference.md#10-parent-directory-creation)**

---

## 3. Complete Production Pattern

Combining atomic writes, parent directory creation, and NamedTempFile:

```rust
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    // 1. Serialize data
    let json = serde_json::to_string_pretty(self)
        .context("Failed to serialize settings")?;

    // 2. Create parent directories
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create parent directories")?;
    }

    // 3. Atomic write with NamedTempFile
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir)
        .context("Failed to create temporary file")?;

    temp_file.write_all(json.as_bytes())
        .context("Failed to write to temporary file")?;

    temp_file.as_file().sync_all()
        .context("Failed to sync temporary file")?;

    temp_file.persist(path)
        .context("Failed to persist temporary file")?;

    Ok(())
}
```

### Checklist for Robust File Writes

Every production file write should:

- [ ] Use NamedTempFile for atomic writes
- [ ] Create parent directories with `fs::create_dir_all()`
- [ ] Call `sync_all()` before persisting
- [ ] Use `.context()` to add error messages
- [ ] Create temp file in same directory as target (for atomic rename)
- [ ] Have integration tests (see next section)

### Dependencies

```toml
[dependencies]
tempfile = "3.8"  # For production code
anyhow = "1.0"    # For error handling
```

**Note:** While `tempfile` is often a dev-dependency for tests, it's appropriate as a regular dependency for production code that needs atomic writes.

**[↑ Back to Quick Reference](quick-reference.md#15-using-namedtempfile-for-automatic-cleanup)**

---

## 4. Testing File I/O

### The Problem

Testing file I/O operations without integration tests leaves file handling bugs undetected. Unit tests alone can't catch issues like:

- Files not written correctly
- Race conditions in file access
- Permission errors
- Parent directory creation failures

```rust
// ❌ WRONG - Only unit tests, no actual file I/O
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_roundtrip() {
        let settings = ClaudeSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: ClaudeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }

    // No tests for:
    // - Actually reading from files
    // - Actually writing to files
    // - Error handling when file doesn't exist
    // - Error handling when parent directory doesn't exist
}
```

### Solution: Integration Tests with tempfile

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_write_and_read_roundtrip() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        // Create settings
        let mut settings = ClaudeSettings::default();
        settings.enable_all_project_mcp_servers = true;

        // Write to file
        settings.write(&settings_path).unwrap();

        // Verify file exists
        assert!(settings_path.exists());

        // Read back from file
        let loaded = ClaudeSettings::read(&settings_path).unwrap();

        // Verify contents match
        assert_eq!(settings, loaded);
    }

    #[test]
    fn test_write_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Path with nested non-existent directories
        let settings_path = temp_dir.path()
            .join("config")
            .join("user")
            .join("settings.json");

        let settings = ClaudeSettings::default();

        // Should create parent directories automatically
        settings.write(&settings_path).unwrap();

        assert!(settings_path.exists());
        assert!(settings_path.parent().unwrap().exists());
    }

    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("nonexistent.json");

        // Should return error, not panic
        let result = ClaudeSettings::read(&settings_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON
        fs::write(&settings_path, "{ not valid json }").unwrap();

        // Should return parse error
        let result = ClaudeSettings::read(&settings_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_overwrite_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        // Write first settings
        let mut settings1 = ClaudeSettings::default();
        settings1.enable_all_project_mcp_servers = true;
        settings1.write(&settings_path).unwrap();

        // Overwrite with different settings
        let mut settings2 = ClaudeSettings::default();
        settings2.enabled_mcpjson_servers.push("mysql".to_string());
        settings2.write(&settings_path).unwrap();

        // Verify new settings
        let loaded = ClaudeSettings::read(&settings_path).unwrap();
        assert_eq!(loaded.enabled_mcpjson_servers.len(), 1);
        assert!(!loaded.enable_all_project_mcp_servers);
    }
}
```

### Using the tempfile Crate

**Add to Cargo.toml:**

```toml
[dev-dependencies]
tempfile = "3.8"
```

**Key tempfile types:**

```rust
use tempfile::{TempDir, NamedTempFile};

// Temporary directory (deleted when dropped)
let temp_dir = TempDir::new()?;
let path = temp_dir.path().join("file.txt");

// Temporary file (deleted when dropped)
let temp_file = NamedTempFile::new()?;
let path = temp_file.path();

// Keep temp file after test (for debugging)
let (file, path) = temp_file.keep()?;
```

### Testing CLI Commands

```rust
use std::process::Command;

#[test]
fn test_cli_validate_command() {
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.json");

    // Create valid settings file
    let settings = ClaudeSettings::default();
    settings.write(&settings_path).unwrap();

    // Run CLI command
    let output = Command::new("./target/debug/settings-manager")
        .arg("validate")
        .arg(settings_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid"));
}
```

### Test Coverage Checklist

For file I/O operations, test:

- [ ] Round-trip write + read produces identical data
- [ ] Writing to non-existent directories creates parents
- [ ] Reading non-existent file returns error (doesn't panic)
- [ ] Reading invalid file format returns error
- [ ] Overwriting existing file works correctly
- [ ] File permissions are correct (if applicable)
- [ ] Atomic write behavior (no partial files after crashes)

**[↑ Back to Quick Reference](quick-reference.md#12-file-io-testing-with-tempfile)**

---

## Related Topics

### Error Handling
- **[Option handling](error-handling-deep-dive.md#1-understanding-option-types)** - Path operations return Option
- **[Path footguns](error-handling-deep-dive.md#2-common-footgun-path-operations)** - Common mistakes with Path methods

### Common Footguns
- **[TOCTOU Races](common-footguns.md#toctou-races)** - Time-of-check-time-of-use issues
- **[Path operations](common-footguns.md#path-operations)** - Path-specific edge cases

### Best Practices
- **[CLI user feedback](fundamentals-deep-dive.md#cli-user-feedback)** - Inform users about file operations
- **[Error messages](error-handling-deep-dive.md#3-expect-vs-unwrap-vs--decision-guide)** - Using .context() for clear errors

---

**[← Back to Index](index.md)** | **[Quick Reference →](quick-reference.md)**

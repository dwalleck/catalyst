# Catalyst CLI - Helper Function Specifications

**Last Updated:** 2025-01-04
**Status:** Phase 0 - Specifications
**Related:** catalyst-cli-plan.md, catalyst-cli-tasks.md, catalyst-cli-data-structures.md

---

## Overview

This document specifies all helper functions needed for the Catalyst CLI implementation. Each function includes:
- Complete signature with types
- Behavior specification
- Platform-specific variations
- Error handling
- Example usage

**Categories:**
1. Platform & Environment Helpers
2. Filesystem Helpers
3. Hash & Validation Helpers
4. Path Manipulation Helpers
5. Template & String Helpers

---

## Table of Contents

1. [Platform & Environment Helpers](#platform--environment-helpers)
2. [Filesystem Helpers](#filesystem-helpers)
3. [Hash & Validation Helpers](#hash--validation-helpers)
4. [Path Manipulation Helpers](#path-manipulation-helpers)
5. [Template & String Helpers](#template--string-helpers)

---

## Platform & Environment Helpers

### get_home_dir()

**Purpose:** Get the user's home directory in a cross-platform way.

**Signature:**
```rust
fn get_home_dir() -> Result<PathBuf, CatalystError>
```

**Behavior:**

| Platform | Environment Variable | Example Path |
|----------|---------------------|--------------|
| Linux/WSL | `HOME` | `/home/username` |
| macOS | `HOME` | `/Users/username` |
| Windows | `USERPROFILE` | `C:\Users\username` |

**Implementation Strategy:**

```
FUNCTION get_home_dir() -> Result<PathBuf>
    // Use dirs crate for cross-platform home directory
    MATCH dirs::home_dir()
        CASE Some(path)
            RETURN Ok(path)

        CASE None
            // Fallback: Try environment variables directly
            platform = Platform::detect()

            env_var = platform.home_dir_var()  // "HOME" or "USERPROFILE"

            MATCH env::var(env_var)
                CASE Ok(value)
                    RETURN Ok(PathBuf::from(value))

                CASE Err(_)
                    RETURN Err(CatalystError::NoHomeDirectory)
            END MATCH
    END MATCH
END FUNCTION
```

**Error Handling:**
- Returns `CatalystError::NoHomeDirectory` if not found
- Suggests setting appropriate environment variable

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| Running in container | May return `/root` or empty |
| Sudo execution | Returns root's home, not original user |
| Service account | May have non-standard home |
| Environment variable unset | Error with helpful message |

**Example Usage:**
```rust
let home = get_home_dir()?;
let bin_dir = home.join(".claude-hooks/bin");
```

---

### is_executable()

**Purpose:** Check if a file has executable permissions (Unix) or exists (Windows).

**Signature:**
```rust
fn is_executable(path: &Path) -> bool
```

**Behavior:**

| Platform | Check |
|----------|-------|
| Linux/macOS/WSL | File has `+x` permission bit |
| Windows | File exists (all files are "executable") |

**Implementation Strategy:**

```
FUNCTION is_executable(path: &Path) -> bool
    // 1. Check file exists
    IF NOT path.exists() THEN
        RETURN false
    END IF

    // 2. Platform-specific check
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        metadata = match fs::metadata(path)
            Ok(m) => m,
            Err(_) => return false
        END

        mode = metadata.permissions().mode()

        // Check if any execute bit is set (user, group, or other)
        RETURN (mode & 0o111) != 0
    }

    #[cfg(windows)]
    {
        // On Windows, all files are executable if they exist
        RETURN true
    }
END FUNCTION
```

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| Symbolic link | Follow link, check target |
| No read permission on file | May return false (can't read metadata) |
| File on network share | May have different permission semantics |
| Windows .exe extension | Not required for detection |

**Example Usage:**
```rust
if !is_executable(&wrapper_path) {
    return Err(CatalystError::PermissionDenied {
        path: wrapper_path,
        operation: "execute".to_string(),
    });
}
```

---

### set_executable()

**Purpose:** Set executable permissions on a file (Unix only, no-op on Windows).

**Signature:**
```rust
fn set_executable(path: &Path) -> Result<(), CatalystError>
```

**Behavior:**

| Platform | Action |
|----------|--------|
| Linux/macOS/WSL | Sets mode to `0o755` (rwxr-xr-x) |
| Windows | No-op (returns Ok) |

**Implementation Strategy:**

```
FUNCTION set_executable(path: &Path) -> Result<()>
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Get current permissions
        metadata = fs::metadata(path)
            .map_err(|e| CatalystError::IoError {
                source: e,
                context: format!("Reading permissions: {}", path.display())
            })?;

        mut permissions = metadata.permissions();

        // Set mode to 0o755 (rwxr-xr-x)
        permissions.set_mode(0o755);

        // Apply permissions
        fs::set_permissions(path, permissions)
            .map_err(|e| CatalystError::PermissionDenied {
                path: path.to_path_buf(),
                operation: format!("set executable: {}", e)
            })?;
    }

    #[cfg(windows)]
    {
        // No-op on Windows
    }

    RETURN Ok(())
END FUNCTION
```

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| Read-only filesystem | Error with PermissionDenied |
| File doesn't exist | Error with IoError |
| Symbolic link | Sets permission on target, not link |
| Network filesystem | May fail on some network shares |

---

### get_binary_version()

**Purpose:** Get version of installed binary (MVP: returns None).

**Signature:**
```rust
fn get_binary_version(binary_name: &str) -> Result<Option<String>, CatalystError>
```

**Behavior:**

**MVP Implementation:**
```
FUNCTION get_binary_version(binary_name: &str) -> Result<Option<String>>
    // MVP: Version detection not implemented
    RETURN Ok(None)
END FUNCTION
```

**Future Implementation (Post-MVP):**
```
FUNCTION get_binary_version(binary_name: &str) -> Result<Option<String>>
    // Execute: binary_name --version
    output = Command::new(binary_name)
        .arg("--version")
        .output()?

    IF output.status.success() THEN
        version_str = String::from_utf8_lossy(&output.stdout)
        // Parse version from output
        RETURN Ok(Some(parse_version(version_str)))
    ELSE
        RETURN Ok(None)
    END IF
END FUNCTION
```

**Example Usage:**
```rust
let version = get_binary_version("skill-activation-prompt")?;
// MVP: version is always None
```

---

## Filesystem Helpers

### write_file_atomic()

**Purpose:** Write file atomically with fallback for network filesystems.

**Signature:**
```rust
fn write_file_atomic(path: &Path, content: impl AsRef<[u8]>) -> Result<(), CatalystError>
```

**Behavior:**

Attempts atomic write (write to temp file, then rename). Falls back to regular write if atomic operation fails (network FS, Docker volumes).

**Implementation Strategy:**

```
FUNCTION write_file_atomic(path: &Path, content: &[u8]) -> Result<()>
    // Ensure parent directory exists
    IF let Some(parent) = path.parent() THEN
        fs::create_dir_all(parent)?
    END IF

    // Try atomic write using tempfile
    MATCH try_atomic_write(path, content)
        CASE Ok(())
            RETURN Ok(())

        CASE Err(e) IF is_cross_device_error(&e)
            // Network FS or cross-device link error
            warn!("Atomic write failed, using fallback: {}", e)
            fallback_write(path, content)?
            RETURN Ok(())

        CASE Err(e)
            // Other error, propagate
            RETURN Err(CatalystError::AtomicWriteFailed {
                path: path.to_path_buf(),
                fallback_used: false
            })
    END MATCH
END FUNCTION

FUNCTION try_atomic_write(path: &Path, content: &[u8]) -> Result<()>
    // Create temp file in same directory
    temp_dir = path.parent().unwrap_or(Path::new("."))

    mut temp_file = NamedTempFile::new_in(temp_dir)?

    // Write content to temp file
    temp_file.write_all(content)?

    // Flush to disk
    temp_file.flush()?

    // Atomically rename (fails on network FS)
    temp_file.persist(path)?

    RETURN Ok(())
END FUNCTION

FUNCTION fallback_write(path: &Path, content: &[u8]) -> Result<()>
    // Regular write (not atomic)
    fs::write(path, content)?
    RETURN Ok(())
END FUNCTION

FUNCTION is_cross_device_error(error: &std::io::Error) -> bool
    // Check for EXDEV error (cross-device link)
    error.raw_os_error() == Some(libc::EXDEV)
END FUNCTION
```

**Error Handling:**

| Error | Handling |
|-------|----------|
| Permission denied | CatalystError::PermissionDenied |
| Cross-device link (EXDEV) | Fallback to regular write, warn user |
| Disk full | CatalystError::IoError |
| Parent directory doesn't exist | Create it first |

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| Network filesystem | Fallback to regular write |
| Docker volume | Fallback to regular write |
| Read-only filesystem | Error |
| Concurrent writes | Atomic write prevents corruption |

---

### ensure_directory_exists()

**Purpose:** Create directory and all parents if they don't exist.

**Signature:**
```rust
fn ensure_directory_exists(path: &Path) -> Result<(), CatalystError>
```

**Implementation:**
```
FUNCTION ensure_directory_exists(path: &Path) -> Result<()>
    IF path.exists() THEN
        IF NOT path.is_dir() THEN
            RETURN Err(CatalystError::InvalidPath {
                path: path.to_path_buf(),
                reason: "Path exists but is not a directory".to_string()
            })
        END IF
        RETURN Ok(())  // Already exists and is directory
    END IF

    fs::create_dir_all(path)
        .map_err(|e| CatalystError::IoError {
            source: e,
            context: format!("Creating directory: {}", path.display())
        })?

    RETURN Ok(())
END FUNCTION
```

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| Path exists as file | Error: InvalidPath |
| Parent is read-only | Error: PermissionDenied |
| Path already exists as dir | No-op, return Ok |
| Symbolic link to directory | Treat as directory |

---

## Hash & Validation Helpers

### hash_file()

**Purpose:** Compute SHA256 hash of file content.

**Signature:**
```rust
fn hash_file(path: &Path) -> Result<String, CatalystError>
```

**Implementation:**

```
FUNCTION hash_file(path: &Path) -> Result<String>
    use sha2::{Sha256, Digest};

    // 1. Read file as raw bytes (preserves exact content)
    content = fs::read(path)
        .map_err(|e| CatalystError::IoError {
            source: e,
            context: format!("Reading file for hashing: {}", path.display())
        })?;

    // 2. Create hasher
    mut hasher = Sha256::new();

    // 3. Feed content to hasher
    hasher.update(&content);

    // 4. Finalize and get result
    result = hasher.finalize();

    // 5. Convert to lowercase hex string (always 64 characters)
    hash_string = format!("{:x}", result);

    RETURN Ok(hash_string)
END FUNCTION
```

**Behavior:**

- Always returns lowercase hex string
- Exactly 64 characters (SHA256 = 256 bits = 32 bytes = 64 hex chars)
- Reads entire file into memory (acceptable for .md files <1MB)

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| File doesn't exist | IoError |
| Empty file | Valid hash (e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855) |
| Very large file | May use excessive memory (future: streaming) |
| Binary file | Works correctly (bytes are bytes) |
| Symbolic link | Hashes target content |

**Example Usage:**
```rust
let hash = hash_file(&skill_file)?;
hashes.insert(rel_path, hash);
```

---

### validate_json_file()

**Purpose:** Validate that a file contains valid JSON.

**Signature:**
```rust
fn validate_json_file(path: &Path) -> Result<serde_json::Value, CatalystError>
```

**Implementation:**

```
FUNCTION validate_json_file(path: &Path) -> Result<serde_json::Value>
    // 1. Read file content
    content = fs::read_to_string(path)
        .map_err(|e| CatalystError::IoError {
            source: e,
            context: format!("Reading JSON file: {}", path.display())
        })?;

    // 2. Parse as JSON
    value = serde_json::from_str(&content)
        .map_err(|e| CatalystError::JsonError {
            source: e,
            file: path.to_path_buf()
        })?;

    RETURN Ok(value)
END FUNCTION
```

**Error Handling:**

Errors include line and column numbers for debugging:
```
Error: Failed to parse JSON file: .claude/settings.json
  --> line 5, column 12
  |
5 |     "command" "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
  |              ^ expected `:` after key
```

**Example Usage:**
```rust
// Just validate
validate_json_file(&settings_path)?;

// Validate and use
let value = validate_json_file(&settings_path)?;
let hooks = value["hooks"].as_object().unwrap();
```

---

### is_valid_sha256()

**Purpose:** Check if string is valid SHA256 hash.

**Signature:**
```rust
fn is_valid_sha256(hash: &str) -> bool
```

**Implementation:**

```
FUNCTION is_valid_sha256(hash: &str) -> bool
    // Must be exactly 64 characters
    IF hash.len() != 64 THEN
        RETURN false
    END IF

    // All characters must be hex digits (0-9, a-f)
    FOR each char IN hash.chars()
        IF NOT char.is_ascii_hexdigit() THEN
            RETURN false
        END IF
    END FOR

    RETURN true
END FUNCTION
```

**Example Usage:**
```rust
if !is_valid_sha256(&stored_hash) {
    return Err(CatalystError::InvalidPath {
        path: hashes_file,
        reason: format!("Invalid SHA256 hash: {}", stored_hash),
    });
}
```

---

## Path Manipulation Helpers

### validate_skill_path()

**Purpose:** Validate that a skill file path stays within the skill's directory (prevents path traversal attacks).

**Signature:**
```rust
fn validate_skill_path(skill_id: &str, rel_path: &Path, base_dir: &Path) -> Result<(), CatalystError>
```

**Implementation:**

```
FUNCTION validate_skill_path(skill_id: &str, rel_path: &Path, base_dir: &Path) -> Result<()>
    // 1. Build expected base path for this skill
    expected_base = base_dir.join(".claude/skills").join(skill_id)

    // 2. Build actual path (may contain ..)
    actual_path = expected_base.join(rel_path)

    // 3. Canonicalize to resolve .. and symlinks
    canonical_actual = canonicalize_path(&actual_path)?

    // 4. Canonicalize expected base
    canonical_base = canonicalize_path(&expected_base)?

    // 5. Verify actual path starts with expected base
    IF NOT canonical_actual.starts_with(&canonical_base) THEN
        RETURN Err(CatalystError::PathTraversalDetected {
            skill_id: skill_id.to_string(),
            attempted_path: rel_path.to_path_buf(),
            reason: "Path escapes skill directory".to_string()
        })
    END IF

    RETURN Ok(())
END FUNCTION
```

**Security Guarantees:**

| Attack Vector | Detection Method |
|---------------|------------------|
| Parent directory (`../../../etc/passwd`) | Canonicalization resolves to path outside base |
| Absolute path (`/etc/passwd`) | Doesn't start with base directory |
| Symlink escape | Canonicalization follows symlink, detects escape |
| Windows drive letter (`C:\...`) | Doesn't start with base |
| UNC path (`\\?\...`) | Doesn't start with base |

**Edge Cases:**

| Case | Handling |
|------|----------|
| Path doesn't exist yet | Use parent directory for validation |
| Symlink within `.claude/` | ✅ Allowed (resolves within base) |
| Very long path | May fail canonicalization (returns error) |
| Case-insensitive filesystem | Canonicalization normalizes case |

**Example Usage:**
```rust
// Before writing any skill file
validate_skill_path("skill-developer", Path::new("resources/guide.md"), &project_root)?;

// Now safe to write
let target = project_root
    .join(".claude/skills/skill-developer")
    .join("resources/guide.md");
write_file_atomic(&target, content)?;
```

**Example Rejections:**
```rust
// ❌ Rejected - escapes to parent
validate_skill_path("skill-1", Path::new("../../etc/passwd"), &root)
// Error: PathTraversalDetected

// ❌ Rejected - absolute path
validate_skill_path("skill-1", Path::new("/etc/passwd"), &root)
// Error: PathTraversalDetected

// ✅ Allowed - within skill directory
validate_skill_path("skill-1", Path::new("resources/guide.md"), &root)
// Ok(())
```

---

### to_forward_slashes()

**Purpose:** Convert path to use forward slashes (even on Windows).

**Signature:**
```rust
fn to_forward_slashes(path: &Path) -> String
```

**Implementation:**

```
FUNCTION to_forward_slashes(path: &Path) -> String
    // Convert Path to string
    path_str = path.to_string_lossy().to_string();

    // Replace backslashes with forward slashes
    RETURN path_str.replace("\\", "/")
END FUNCTION
```

**Why Needed:**

JSON files must use forward slashes for consistency:
- ✅ `"skill-developer/SKILL.md"`
- ❌ `"skill-developer\\SKILL.md"`

**Example Usage:**
```rust
let rel_path = file.strip_prefix(".claude/skills/")?;
let normalized = to_forward_slashes(&rel_path);
hashes.insert(normalized, hash);
```

---

### canonicalize_path()

**Purpose:** Resolve symbolic links and convert to absolute path.

**Signature:**
```rust
fn canonicalize_path(path: &Path) -> Result<PathBuf, CatalystError>
```

**Implementation:**

```
FUNCTION canonicalize_path(path: &Path) -> Result<PathBuf>
    // Use std::fs::canonicalize
    canonical = fs::canonicalize(path)
        .map_err(|e| CatalystError::InvalidPath {
            path: path.to_path_buf(),
            reason: format!("Cannot canonicalize: {}", e)
        })?;

    // On Windows, canonicalize returns UNC paths (\\?\C:\...)
    // Use dunce crate to convert back to regular paths
    #[cfg(windows)]
    {
        use dunce::simplified;
        RETURN Ok(simplified(&canonical).to_path_buf())
    }

    #[cfg(not(windows))]
    {
        RETURN Ok(canonical)
    }
END FUNCTION
```

**Edge Cases:**

| Edge Case | Handling |
|-----------|----------|
| Symbolic link | Resolves to target |
| Relative path | Converts to absolute |
| Windows UNC path | Simplifies with dunce |
| Path doesn't exist | Error |
| Broken symlink | Error |

**Example Usage:**
```rust
let project_root = canonicalize_path(&current_dir)?;
```

---

### find_binary_path()

**Purpose:** Search for binary in standard locations.

**Signature:**
```rust
fn find_binary_path(binary_name: &str, platform: &Platform) -> Option<PathBuf>
```

**Implementation:**

```
FUNCTION find_binary_path(binary_name: &str, platform: &Platform) -> Option<PathBuf>
    // Add platform-specific extension
    binary_with_ext = binary_name + platform.binary_extension()

    // 1. Try standalone installation first
    home_dir = get_home_dir().ok()?
    standalone_path = home_dir.join(".claude-hooks/bin").join(&binary_with_ext)

    IF standalone_path.exists() THEN
        RETURN Some(standalone_path)
    END IF

    // 2. Try project build (for development)
    // Look for catalyst/target/release/<binary>
    current_dir = env::current_dir().ok()?

    // Walk up directory tree looking for catalyst repo
    FOR each ancestor IN current_dir.ancestors()
        candidate = ancestor
            .join("target/release")
            .join(&binary_with_ext)

        IF candidate.exists() THEN
            RETURN Some(candidate)
        END IF
    END FOR

    // 3. Try PATH
    MATCH which::which(&binary_with_ext)
        CASE Ok(path)
            RETURN Some(path)
        CASE Err(_)
            RETURN None
    END MATCH
END FUNCTION
```

**Search Order:**

1. **Standalone installation**: `~/.claude-hooks/bin/` (recommended)
2. **Project build**: `catalyst/target/release/` (development)
3. **PATH**: System-wide installation

**Example Usage:**
```rust
let binary_path = find_binary_path("skill-activation-prompt", &platform)
    .ok_or(CatalystError::BinariesNotInstalled { ... })?;
```

---

## Template & String Helpers

### load_wrapper_template()

**Purpose:** Load wrapper script template for platform.

**Signature:**
```rust
fn load_wrapper_template(platform: &Platform) -> &'static str
```

**Implementation:**

```
FUNCTION load_wrapper_template(platform: &Platform) -> &'static str
    MATCH platform
        CASE Platform::Linux | Platform::MacOS | Platform::WSL
            RETURN include_str!("../resources/wrapper-template.sh")

        CASE Platform::Windows
            RETURN include_str!("../resources/wrapper-template.ps1")
    END MATCH
END FUNCTION
```

**Template Format (Unix):**

```bash
#!/bin/bash
# Auto-generated wrapper for {{BINARY_NAME}}

# Try standalone installation first
if [ -x "$HOME/.claude-hooks/bin/{{BINARY_NAME}}" ]; then
    cat | "$HOME/.claude-hooks/bin/{{BINARY_NAME}}"
    exit $?
fi

# Fallback: Try PATH
if command -v {{BINARY_NAME}} >/dev/null 2>&1; then
    cat | {{BINARY_NAME}}
    exit $?
fi

echo "Error: {{BINARY_NAME}} not found" >&2
echo "Please install Catalyst binaries: ./install.sh" >&2
exit 1
```

**Template Format (Windows - PowerShell):**

```powershell
# Auto-generated wrapper for {{BINARY_NAME}}

# Try standalone installation first
$standaloneExe = Join-Path $env:USERPROFILE ".claude-hooks\bin\{{BINARY_NAME}}.exe"
if (Test-Path $standaloneExe) {
    $input | & $standaloneExe @args
    exit $LASTEXITCODE
}

# Fallback: Try PATH
if (Get-Command {{BINARY_NAME}}.exe -ErrorAction SilentlyContinue) {
    $input | & {{BINARY_NAME}}.exe @args
    exit $LASTEXITCODE
}

Write-Error "Error: {{BINARY_NAME}} not found"
Write-Error "Please install Catalyst binaries: .\install.ps1"
exit 1
```

**Key Differences:**

| Feature | Unix | Windows |
|---------|------|---------|
| Shebang | `#!/bin/bash` | None (PowerShell doesn't use shebangs) |
| Stdin piping | `cat \|` | `$input \|` |
| Exit code | `exit $?` | `exit $LASTEXITCODE` |
| Args passing | Automatic | `@args` |
| Extension | None | `.exe` |

---

### substitute_template_vars()

**Purpose:** Replace template variables in string.

**Signature:**
```rust
fn substitute_template_vars(template: &str, vars: &HashMap<String, String>) -> String
```

**Implementation:**

```
FUNCTION substitute_template_vars(template: &str, vars: &HashMap<String, String>) -> String
    mut result = template.to_string()

    FOR each (key, value) IN vars
        placeholder = format!("{{{{{}}}}}", key)  // "{{KEY}}"
        result = result.replace(&placeholder, value)
    END FOR

    RETURN result
END FUNCTION
```

**Example Usage:**
```rust
let template = load_wrapper_template(&platform);
let mut vars = HashMap::new();
vars.insert("BINARY_NAME".to_string(), "skill-activation-prompt".to_string());

let wrapper_content = substitute_template_vars(template, &vars);
```

---

## Helper Functions Summary Table

| Function | Purpose | Returns | Platform-Specific |
|----------|---------|---------|------------------|
| `get_home_dir()` | User home directory | `PathBuf` | Yes (env var) |
| `is_executable()` | Check exec permission | `bool` | Yes (Unix checks +x) |
| `set_executable()` | Set exec permission | `Result<()>` | Yes (Unix only) |
| `get_binary_version()` | Binary version | `Option<String>` | No (MVP: always None) |
| `write_file_atomic()` | Atomic file write | `Result<()>` | No (fallback on network FS) |
| `ensure_directory_exists()` | Create directory | `Result<()>` | No |
| `hash_file()` | SHA256 hash | `String` | No |
| `validate_json_file()` | Parse JSON | `Value` | No |
| `is_valid_sha256()` | Validate hash format | `bool` | No |
| `validate_skill_path()` | Path traversal prevention | `Result<()>` | No (security) |
| `to_forward_slashes()` | Normalize path | `String` | No |
| `canonicalize_path()` | Resolve symlinks | `PathBuf` | Yes (Windows UNC) |
| `find_binary_path()` | Locate binary | `Option<PathBuf>` | Yes (extension) |
| `load_wrapper_template()` | Get template | `&str` | Yes (sh vs ps1) |
| `substitute_template_vars()` | Replace vars | `String` | No |

---

## Completion Checklist

- [x] Specify `get_home_dir()` signature and behavior
- [x] Specify `is_executable()` signature (platform-specific behavior)
- [x] Specify `get_binary_version()` signature (returns None for MVP)
- [x] Specify `hash_file()` signature (SHA256, requires sha2 crate)
- [x] Define all return types and error handling
- [x] Specify additional filesystem helpers (write_file_atomic, ensure_directory_exists)
- [x] Specify path manipulation helpers (to_forward_slashes, canonicalize_path, find_binary_path)
- [x] Specify template helpers (load_wrapper_template, substitute_template_vars)
- [x] Document platform-specific variations
- [x] Document edge cases for all functions

---

**End of Helper Function Specifications**

Next: See `catalyst-cli-cross-platform.md` (Task 0.5) for cross-platform and safety specifications.

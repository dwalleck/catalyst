# Catalyst CLI - Cross-Platform & Safety Specifications

**Last Updated:** 2025-01-04
**Status:** Phase 0 - Specifications (Final)
**Related:** catalyst-cli-plan.md, catalyst-cli-tasks.md

---

## Overview

This document specifies critical cross-platform behaviors and safety mechanisms identified during plan review. These specifications address edge cases that could cause failures on specific platforms or in concurrent scenarios.

**Topics Covered:**
1. WSL Detection Logic
2. SQLite Feature Detection
3. Concurrent Init Protection
4. Atomic Write Fallback Strategy
5. PowerShell Wrapper Templates
6. Windows Execution Policy Requirements

---

## Table of Contents

1. [WSL Detection Logic](#wsl-detection-logic)
2. [SQLite Feature Detection](#sqlite-feature-detection)
3. [Concurrent Init Protection](#concurrent-init-protection)
4. [Atomic Write Fallback Strategy](#atomic-write-fallback-strategy)
5. [PowerShell Wrapper Templates](#powershell-wrapper-templates)
6. [Windows Execution Policy Requirements](#windows-execution-policy-requirements)

---

## WSL Detection Logic

**Problem:** WSL (Windows Subsystem for Linux) needs different wrapper scripts than native Windows, but `cfg!(windows)` is true in WSL builds compiled for Windows.

**Solution:** Detect WSL at runtime using environment variable.

### Detection Algorithm

```
FUNCTION detect_platform() -> Platform
    // Priority order:
    // 1. Compile-time Windows check
    // 2. Runtime WSL detection
    // 3. Compile-time OS detection

    // Step 1: Check if compiled for Windows
    #[cfg(windows)]
    {
        RETURN Platform::Windows
    }

    // Step 2: Check for WSL_DISTRO_NAME environment variable
    IF env::var("WSL_DISTRO_NAME").is_ok() THEN
        RETURN Platform::WSL
    END IF

    // Step 3: Check target OS
    #[cfg(target_os = "macos")]
    {
        RETURN Platform::MacOS
    }

    #[cfg(target_os = "linux")]
    {
        RETURN Platform::Linux
    }

    // Fallback (should never happen)
    RETURN Platform::Linux
END FUNCTION
```

### WSL_DISTRO_NAME Environment Variable

**What it is:**
- Set by WSL automatically when running inside WSL
- Contains the distribution name (e.g., "Ubuntu", "Debian", "Alpine")
- Not present in native Linux
- Not present in native Windows

**Example values:**
```bash
# Inside WSL Ubuntu
$ echo $WSL_DISTRO_NAME
Ubuntu

# Inside WSL Debian
$ echo $WSL_DISTRO_NAME
Debian

# Native Linux (not present)
$ echo $WSL_DISTRO_NAME

```

### Platform Method Implementations for WSL

```
impl Platform {
    fn wrapper_extension(&self) -> &str {
        match self {
            Platform::Linux | Platform::MacOS | Platform::WSL => ".sh",
            Platform::Windows => ".ps1",
        }
    }

    fn home_dir_var(&self) -> &str {
        match self {
            Platform::Linux | Platform::MacOS | Platform::WSL => "HOME",
            Platform::Windows => "USERPROFILE",
        }
    }

    fn binary_extension(&self) -> &str {
        match self {
            Platform::Linux | Platform::MacOS | Platform::WSL => "",
            Platform::Windows => ".exe",
        }
    }

    fn has_permissions(&self) -> bool {
        match self {
            Platform::Linux | Platform::MacOS | Platform::WSL => true,
            Platform::Windows => false,
        }
    }
}
```

### Testing WSL Detection

**Manual Test (in WSL):**
```bash
# Set environment variable
export WSL_DISTRO_NAME=Ubuntu

# Run detection
catalyst init

# Should create .sh wrappers, not .ps1
ls .claude/hooks/
# Expected: skill-activation-prompt.sh
```

**Unit Test:**
```rust
#[test]
fn test_wsl_detection() {
    // Save original value
    let original = env::var("WSL_DISTRO_NAME").ok();

    // Set WSL environment variable
    env::set_var("WSL_DISTRO_NAME", "Ubuntu");

    // Detect should return WSL
    assert_eq!(Platform::detect(), Platform::WSL);

    // Restore original
    match original {
        Some(val) => env::set_var("WSL_DISTRO_NAME", val),
        None => env::remove_var("WSL_DISTRO_NAME"),
    }
}
```

### Edge Cases

| Scenario | Detection Result | Behavior |
|----------|-----------------|----------|
| WSL 1 | Platform::WSL | ✅ Correct (.sh wrappers) |
| WSL 2 | Platform::WSL | ✅ Correct (.sh wrappers) |
| Native Linux | Platform::Linux | ✅ Correct (.sh wrappers) |
| Windows Terminal running WSL | Platform::WSL | ✅ Correct (.sh wrappers) |
| Docker on Windows | Platform::Linux | ✅ Correct (no WSL_DISTRO_NAME) |
| Git Bash on Windows | Platform::Windows | ⚠️ Needs .ps1 wrappers |

---

## SQLite Feature Detection

**Problem:** `file-change-tracker` has two variants:
- **Basic**: Built without `--features sqlite`
- **SQLite**: Built with `--features sqlite`

Both variants have the same binary name. We need to detect which is installed.

### Detection Strategies

#### Strategy 1: Version String Detection (Recommended)

**Approach:** Binary outputs variant in `--version`:

```bash
# SQLite variant
$ file-change-tracker --version
file-change-tracker 0.1.0 (sqlite)

# Basic variant
$ file-change-tracker --version
file-change-tracker 0.1.0 (basic)
```

**Implementation:**
```
FUNCTION detect_tracker_variant(binary_path: &Path) -> Result<Option<String>>
    output = Command::new(binary_path)
        .arg("--version")
        .output()?

    IF NOT output.status.success() THEN
        RETURN Ok(None)
    END IF

    version_str = String::from_utf8_lossy(&output.stdout)

    // Parse: "file-change-tracker 0.1.0 (sqlite)"
    IF version_str.contains("(sqlite)") THEN
        RETURN Ok(Some("sqlite".to_string()))
    ELSE IF version_str.contains("(basic)") THEN
        RETURN Ok(Some("basic".to_string()))
    ELSE
        // Old binary without variant info
        RETURN Ok(None)
    END IF
END FUNCTION
```

#### Strategy 2: Database File Detection (Fallback)

**Approach:** SQLite variant creates `.catalyst-tracker.db`:

```
FUNCTION detect_tracker_variant_fallback(project_path: &Path) -> Option<String>
    db_path = project_path.join(".claude/.catalyst-tracker.db")

    IF db_path.exists() THEN
        RETURN Some("sqlite".to_string())
    ELSE
        RETURN Some("basic".to_string())
    END IF
END FUNCTION
```

**Limitation:** Only works after tracker has run once.

### Status Reporting

```
FUNCTION validate_binaries() -> Result<Vec<BinaryStatus>>
    // ... check other binaries ...

    // Check file-change-tracker
    IF let Some(tracker_path) = find_binary_path("file-change-tracker", &platform)
        variant = detect_tracker_variant(&tracker_path)?

        binaries.push(BinaryStatus {
            name: "file-change-tracker".to_string(),
            variant: variant,  // Some("sqlite") or Some("basic") or None
            found: true,
            path: Some(tracker_path),
            version: None,  // MVP
        })
    ELSE
        binaries.push(BinaryStatus {
            name: "file-change-tracker".to_string(),
            variant: None,
            found: false,
            path: None,
            version: None,
        })
    END IF

    RETURN Ok(binaries)
END FUNCTION
```

### Error Messages

**If SQLite required but basic installed:**
```
❌ file-change-tracker is basic variant, but project needs SQLite support

To fix:
  cd catalyst
  ./install.sh --sqlite    # Linux/macOS
  .\install.ps1 -Sqlite    # Windows
```

**If basic sufficient:**
```
✅ file-change-tracker (basic variant)
ℹ️  Upgrade to SQLite variant for persistent tracking:
    cd catalyst && ./install.sh --sqlite
```

### Coordination with install.sh

**install.sh should build correct variant:**

```bash
# Default: basic variant
cargo build --release --bin file-change-tracker

# With --sqlite flag
cargo build --release --bin file-change-tracker --features sqlite
```

**Binary naming stays the same** - variant is internal.

---

## Concurrent Init Protection

**Problem:** Multiple `catalyst init` processes running simultaneously could corrupt files or create inconsistent state.

**Solution:** Lock file mechanism with PID tracking.

### Lock File Format

**Location:** `.catalyst.lock`

**Content:**
```json
{
  "pid": 12345,
  "started_at": "2025-01-04T10:30:00Z",
  "command": "catalyst init --interactive"
}
```

### Lock Acquisition Algorithm

```
FUNCTION acquire_init_lock(project_path: &Path) -> Result<LockGuard>
    lock_path = project_path.join(".catalyst.lock")

    // 1. Check if lock exists
    IF lock_path.exists() THEN
        // Read lock file
        lock_data = read_lock_file(&lock_path)?

        // 2. Check if process still running
        IF is_process_running(lock_data.pid) THEN
            // Another init is running
            RETURN Err(CatalystError::InitInProgress {
                pid: lock_data.pid,
                lock_path: lock_path
            })
        ELSE
            // Stale lock (process died)
            warn!("Removing stale lock file from PID {}", lock_data.pid)
            fs::remove_file(&lock_path)?
        END IF
    END IF

    // 3. Create lock file with current PID
    lock_data = LockData {
        pid: std::process::id(),
        started_at: Utc::now(),
        command: env::args().collect::<Vec<_>>().join(" ")
    }

    write_file_atomic(&lock_path, serde_json::to_string_pretty(&lock_data)?)?

    // 4. Return guard that removes lock on drop
    RETURN Ok(LockGuard {
        path: lock_path
    })
END FUNCTION
```

### Lock Guard (RAII Pattern)

```
STRUCT LockGuard {
    path: PathBuf
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Remove lock file when guard is dropped
        let _ = fs::remove_file(&self.path);
    }
}
```

**Usage:**
```rust
fn run_init(config: InitConfig) -> Result<InitReport> {
    // Acquire lock (fails if another init running)
    let _lock = acquire_init_lock(&project_path)?;

    // Lock is held until function returns
    // Automatically released even if error occurs
    create_directory_structure()?;
    install_skills(&config.skills)?;
    // ...

    // Lock released here (guard dropped)
    Ok(report)
}
```

### Process Detection

**Unix/Linux/macOS:**
```
FUNCTION is_process_running(pid: u32) -> bool
    // Use kill(pid, 0) to check if process exists
    // (doesn't actually send signal)

    unsafe {
        libc::kill(pid as i32, 0) == 0
    }
END FUNCTION
```

**Windows:**
```
FUNCTION is_process_running(pid: u32) -> bool
    use windows::Win32::System::Threading::*;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid);

        IF handle.is_null() THEN
            RETURN false  // Process doesn't exist
        END IF

        let mut exit_code: u32 = 0;
        GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);

        RETURN exit_code == STILL_ACTIVE
    }
END FUNCTION
```

**Fallback (Cross-Platform):**
```
FUNCTION is_process_running(pid: u32) -> bool
    // Use sysinfo crate for cross-platform process detection
    use sysinfo::{System, SystemExt, ProcessExt};

    let mut system = System::new_all();
    system.refresh_processes();

    RETURN system.process(Pid::from(pid)).is_some()
END FUNCTION
```

**Implementation Recommendation (From Code Review):**

For **MVP simplicity**, use the `sysinfo` crate implementation for ALL platforms instead of three separate implementations:

**Rationale:**
- ✅ **Simpler**: One implementation vs three
- ✅ **Maintainable**: No unsafe code, no platform-specific APIs
- ✅ **Portable**: Works on Linux/macOS/Windows/WSL
- ✅ **Sufficient**: Lock file check is advisory, not critical path
- ⚠️ **Dependency**: Adds sysinfo crate (~150KB binary size)

**Future Optimization**: If performance becomes an issue (unlikely for init), can optimize to platform-specific implementations in later phases.

**Decision Point**: Phase 1 implementation should start with sysinfo only, optimize later if needed.

### Edge Cases

| Scenario | Handling |
|----------|----------|
| Process killed (SIGKILL) | Stale lock detected, removed |
| Power failure | Stale lock on reboot, removed |
| Lock file corrupt | Error, suggest manual removal |
| Lock file deleted mid-init | Second init proceeds (not ideal but safe) |
| Same PID reused | Rare; lock removed if process name differs |
| Network filesystem | Lock file may not work (warn user) |

### Error Message

```
❌ Another catalyst init is already running (PID: 12345)
   Started at: 2025-01-04 10:30:00
   Command: catalyst init --interactive

If no other init is running, remove the lock file:
  rm .catalyst.lock
```

---

## Atomic Write Fallback Strategy

**Problem:** Atomic file writes (temp file + rename) fail on some filesystems:
- Network filesystems (NFS, SMB)
- Docker volumes
- Some virtual filesystems

**Symptom:** `EXDEV` error (cross-device link) when renaming temp file.

**Solution:** Fallback to regular write with user warning.

### Implementation (see helper functions)

Already specified in `catalyst-cli-helpers.md` under `write_file_atomic()`.

### Key Points

1. **Try atomic first**: Always attempt `NamedTempFile::persist()`
2. **Detect cross-device error**: Check for `EXDEV` (error code 18 on Unix)
3. **Fallback gracefully**: Use regular `fs::write()` if atomic fails
4. **Warn user**: Inform about non-atomic write

### Warning Message

```
⚠️  Atomic file write not supported on this filesystem
    Wrote: .claude/settings.json
    Note: File may be corrupted if process crashes during write

This typically happens on:
  - Network filesystems (NFS, SMB)
  - Docker volumes
  - Some cloud storage mounts

Recommendation: Copy .claude/ to local filesystem when possible
```

### Testing

**Simulate network FS failure:**
```rust
#[test]
fn test_atomic_write_fallback() {
    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();

    // Mock EXDEV error (hard to simulate real network FS)
    // Test that fallback_write is called
    // Verify warning is logged
}
```

**Manual test on Docker volume:**
```bash
# Create Docker volume
docker volume create catalyst-test

# Run init inside container
docker run -v catalyst-test:/workspace catalyst
cd /workspace
catalyst init

# Should see fallback warning
```

---

## PowerShell Wrapper Templates

**Problem:** Initial plan included Unix shebang in PowerShell templates, which is invalid.

**Fix:** PowerShell scripts must NOT have shebangs.

### Correct PowerShell Template

```powershell
# Auto-generated wrapper for {{BINARY_NAME}}
# DO NOT include shebang (#!/usr/bin/env pwsh) - PowerShell doesn't use them

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

### Key Differences from Bash

| Feature | Bash | PowerShell |
|---------|------|------------|
| **Shebang** | `#!/bin/bash` | **NONE** (no shebang) |
| **Stdin** | `cat \|` | `$input \|` |
| **Exit code** | `exit $?` | `exit $LASTEXITCODE` |
| **Args** | Automatic | `@args` (splatting) |
| **Extension** | No .exe | `.exe` required |
| **Command check** | `command -v` | `Get-Command -ErrorAction SilentlyContinue` |
| **Path join** | `$HOME/.claude-hooks/bin` | `Join-Path $env:USERPROFILE ".claude-hooks\bin"` |

### Why No Shebang?

**On Unix:**
```bash
#!/bin/bash
# Tells OS to use /bin/bash to execute script
```

**On Windows:**
- PowerShell is invoked by file extension (`.ps1`)
- No shebang concept exists
- Including `#!` is treated as a comment, but:
  - ❌ Violates Windows conventions
  - ❌ Confuses some tools
  - ❌ Not portable

### Args Splatting (`@args`)

**Problem:** PowerShell doesn't automatically pass arguments to piped commands.

**Wrong:**
```powershell
$input | & $exe  # Arguments NOT passed
```

**Correct:**
```powershell
$input | & $exe @args  # Arguments passed via splatting
```

**Example:**
```powershell
# User runs: skill-activation-prompt.ps1 --debug
# $args = @("--debug")
# @args splats array as separate arguments
```

### Template Validation

```
FUNCTION validate_powershell_template(template: &str) -> Result<()>
    // 1. Check no shebang
    IF template.starts_with("#!") THEN
        RETURN Err("PowerShell templates must not have shebangs")
    END IF

    // 2. Check uses @args for splatting
    IF template.contains("& $") AND NOT template.contains("@args") THEN
        WARN "PowerShell wrapper should use @args for argument passing"
    END IF

    // 3. Check uses $LASTEXITCODE
    IF template.contains("exit") AND NOT template.contains("$LASTEXITCODE") THEN
        WARN "PowerShell wrapper should check $LASTEXITCODE"
    END IF

    RETURN Ok(())
END FUNCTION
```

---

## Windows Execution Policy Requirements

**Problem:** PowerShell's execution policy may block `.ps1` scripts by default.

### Execution Policies

| Policy | Description | Default On |
|--------|-------------|------------|
| `Restricted` | No scripts allowed | Some Windows editions |
| `AllSigned` | Only signed scripts | Enterprise |
| `RemoteSigned` | Local scripts OK, remote must be signed | Most common |
| `Unrestricted` | All scripts allowed | Developer machines |
| `Bypass` | No restrictions | Rare |

### Checking Current Policy

```powershell
Get-ExecutionPolicy -Scope CurrentUser
```

### Recommended User Action

**Option 1: Set policy for current user (no admin required)**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Option 2: Run once to unblock (per script)**
```powershell
Unblock-File .claude\hooks\skill-activation-prompt.ps1
```

**Option 3: Bypass for single command**
```powershell
powershell -ExecutionPolicy Bypass -File .claude\hooks\skill-activation-prompt.ps1
```

### Detection in Status Command

```
FUNCTION check_execution_policy() -> Result<ExecutionPolicyStatus>
    #[cfg(windows)]
    {
        // Run: Get-ExecutionPolicy -Scope CurrentUser
        output = Command::new("powershell")
            .args(&["-Command", "Get-ExecutionPolicy -Scope CurrentUser"])
            .output()?

        policy = String::from_utf8_lossy(&output.stdout).trim().to_string()

        MATCH policy.as_str()
            CASE "Restricted" | "AllSigned"
                RETURN Ok(ExecutionPolicyStatus::Blocked)

            CASE "RemoteSigned" | "Unrestricted" | "Bypass"
                RETURN Ok(ExecutionPolicyStatus::Allowed)

            CASE default
                RETURN Ok(ExecutionPolicyStatus::Unknown)
        END MATCH
    }

    #[cfg(not(windows))]
    {
        RETURN Ok(ExecutionPolicyStatus::NotApplicable)
    }
END FUNCTION
```

### User Messaging

**During init (Windows):**
```
✅ Catalyst initialized successfully!

⚠️  Windows PowerShell Execution Policy Notice
Your execution policy is: Restricted

Hooks will not run until you update the policy:
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

Or unblock hook files:
  Unblock-File .claude\hooks\*.ps1
```

**During status (Windows):**
```
⚠️  PowerShell execution policy may block hooks
Current policy: Restricted (blocks unsigned scripts)

To fix:
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Documentation

**Add to generated README or help output:**

```markdown
## Windows Setup

After running `catalyst init`, you may need to update PowerShell's execution policy:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

This allows locally-created scripts (like hook wrappers) to run.

Alternative: Unblock individual files:
```powershell
Get-ChildItem .claude\hooks\*.ps1 | Unblock-File
```
```

---

## Cross-Platform Testing Matrix

### Required Test Scenarios

| Platform | Test | Pass Criteria |
|----------|------|---------------|
| **Linux (Ubuntu)** | Full init flow | ✅ Creates .sh wrappers |
| **macOS** | Full init flow | ✅ Creates .sh wrappers |
| **Windows** | Full init flow | ✅ Creates .ps1 wrappers |
| **WSL (Ubuntu)** | Platform detection | ✅ Detects as WSL, creates .sh |
| **Docker (Linux)** | Platform detection | ✅ Detects as Linux (no WSL_DISTRO_NAME) |
| **Network FS** | Atomic write | ✅ Falls back, warns user |
| **Concurrent init** | Lock protection | ❌ Second init fails with helpful error |
| **SQLite variant** | Binary detection | ✅ Detects and reports variant |
| **Basic variant** | Binary detection | ✅ Detects and reports variant |

### CI Configuration

**.github/workflows/ci.yml:**
```yaml
jobs:
  test-linux:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --features sqlite

  test-macos:
    runs-on: macos-latest
    steps:
      - run: cargo test --features sqlite

  test-windows:
    runs-on: windows-latest
    steps:
      - run: cargo test --features sqlite
      - run: powershell -Command "Get-ExecutionPolicy"

  test-wsl:
    runs-on: ubuntu-latest
    steps:
      - run: export WSL_DISTRO_NAME=Ubuntu && cargo test
```

### Additional Test Recommendations (From Code Review)

Beyond the standard test scenarios, these specific tests ensure robustness:

#### 1. Concurrent Init Test
**Purpose**: Verify lock file prevents race conditions

```rust
#[test]
fn test_concurrent_init_protection() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Spawn two init processes simultaneously
    let handle1 = thread::spawn(|| {
        run_init(&temp_dir, InitConfig::default())
    });

    let handle2 = thread::spawn(|| {
        run_init(&temp_dir, InitConfig::default())
    });

    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();

    // One should succeed, one should fail with InitInProgress
    assert!(
        (result1.is_ok() && result2.is_err()) ||
        (result1.is_err() && result2.is_ok())
    );
}
```

#### 2. Atomic Write Fallback Test
**Purpose**: Verify graceful degradation on network filesystems

```rust
#[test]
fn test_atomic_write_fallback() {
    // Mock EXDEV error by using different mount points
    // (Hard to simulate in unit test - integration test needed)

    // Alternative: Test that fallback_write works
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");

    // Force fallback by simulating cross-device error
    let result = write_file_atomic(&file_path, b"test content");

    assert!(result.is_ok());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "test content");
}
```

#### 3. PowerShell Template Validation Test
**Purpose**: Ensure PowerShell wrappers have no shebang and use @args

```rust
#[test]
fn test_powershell_template_no_shebang() {
    let template = load_wrapper_template(&Platform::Windows);

    // ✅ No shebang
    assert!(!template.starts_with("#!"));

    // ✅ Uses @args for argument splatting
    assert!(template.contains("@args"));

    // ✅ Uses $LASTEXITCODE
    assert!(template.contains("$LASTEXITCODE"));

    // ✅ Uses $input for stdin
    assert!(template.contains("$input"));
}
```

#### 4. Schema Validation Tests
**Purpose**: Ensure all example files are valid

```rust
#[test]
fn test_all_schema_examples_valid() {
    // Test settings.json examples
    let settings_unix = include_str!("../../docs/schemas/settings.json.example");
    assert!(serde_json::from_str::<Value>(settings_unix).is_ok());

    let settings_windows = include_str!("../../docs/schemas/settings-windows.json.example");
    assert!(serde_json::from_str::<Value>(settings_windows).is_ok());

    // Test skill-rules.json example
    let skill_rules = include_str!("../../docs/schemas/skill-rules.json.example");
    assert!(serde_json::from_str::<Value>(skill_rules).is_ok());

    // Test .catalyst-hashes.json example
    let hashes = include_str!("../../docs/schemas/.catalyst-hashes.json.example");
    let parsed: HashMap<String, String> = serde_json::from_str(hashes).unwrap();

    // All hashes should be valid SHA256
    for hash in parsed.values() {
        assert!(is_valid_sha256(hash));
    }

    // Test .catalyst-version example
    let version = include_str!("../../docs/schemas/.catalyst-version.example").trim();
    assert!(version.matches('.').count() == 2); // Semver: X.Y.Z
}
```

#### 5. Skill ID Validation Test
**Purpose**: Ensure security constraints enforced

```rust
#[test]
fn test_skill_id_validation() {
    // ✅ Valid IDs
    assert!(validate_skill_id("skill-developer").is_ok());
    assert!(validate_skill_id("backend-dev-guidelines").is_ok());
    assert!(validate_skill_id("my-skill-123").is_ok());

    // ❌ Invalid IDs (security)
    assert!(validate_skill_id("../malicious").is_err());
    assert!(validate_skill_id("/etc/passwd").is_err());
    assert!(validate_skill_id("skill; rm -rf /").is_err());
    assert!(validate_skill_id("Skill-Name").is_err()); // Uppercase
    assert!(validate_skill_id("skill_name").is_err()); // Underscore
    assert!(validate_skill_id("-skill").is_err()); // Starts with hyphen
    assert!(validate_skill_id("skill-").is_err()); // Ends with hyphen
    assert!(validate_skill_id("").is_err()); // Empty
}
```

#### 6. Permission Handling Tests
**Purpose**: Verify behavior on read-only filesystems

```rust
#[test]
#[cfg(unix)]
fn test_init_on_readonly_filesystem() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Make directory read-only
    let mut permissions = fs::metadata(temp_dir.path()).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(temp_dir.path(), permissions).unwrap();

    // Init should fail with PermissionDenied
    let result = run_init(temp_dir.path(), InitConfig::default());
    assert!(matches!(result, Err(CatalystError::PermissionDenied { .. })));
}
```

**Note**: These tests should be added in Phase 8 (Testing) as part of comprehensive test coverage.

---

## Completion Checklist

- [x] Specify WSL detection logic (check WSL_DISTRO_NAME env var)
- [x] Specify SQLite feature detection logic (both variants)
- [x] Specify concurrent init protection mechanism (.catalyst.lock file)
- [x] Specify atomic write fallback strategy (network FS, Docker volumes)
- [x] Specify PowerShell wrapper template (NO shebang, @args syntax)
- [x] Document Windows execution policy requirements
- [x] Define cross-platform testing matrix

---

**End of Cross-Platform & Safety Specifications**

**Phase 0 (Specifications) is now COMPLETE!**

All specifications are ready for implementation to begin in Phase 1.

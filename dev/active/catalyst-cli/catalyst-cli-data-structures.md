# Catalyst CLI - Data Structures Specification

**Last Updated:** 2025-01-04
**Status:** Phase 0 - Specifications
**Related:** catalyst-cli-plan.md, catalyst-cli-tasks.md, catalyst-cli-dependencies.md

---

## Overview

This document specifies ALL data structures (structs and enums) required for the Catalyst CLI implementation. This is a **specification-only** document - no Rust code is written yet.

Each structure includes:
- Purpose and usage
- All fields with types
- Validation rules
- Serialization requirements
- Example instances

---

## Table of Contents

1. [Core Configuration Structures](#core-configuration-structures)
2. [Report Structures](#report-structures)
3. [Status Structures](#status-structures)
4. [Platform Abstraction](#platform-abstraction)
5. [Error Types](#error-types)
6. [Supporting Types](#supporting-types)

---

## Core Configuration Structures

### InitConfig

**Purpose:** Captures user's choices for what to install during `catalyst init`.

**Fields:**

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `install_hooks` | `bool` | Whether to install skill activation hooks | `true` |
| `install_tracker` | `bool` | Whether to install file-change-tracker hook | `true` |
| `skills` | `Vec<String>` | List of skill IDs to install | `vec!["skill-developer"]` |
| `force` | `bool` | Overwrite existing files | `false` |

**Validation Rules:**
- `skills` must not be empty
- Each skill ID in `skills` must exist in embedded resources
- If `install_hooks` is `false`, warn user that skills won't auto-activate

**Example Instances:**

```rust
// Default configuration
InitConfig {
    install_hooks: true,
    install_tracker: true,
    skills: vec!["skill-developer".to_string()],
    force: false,
}

// Interactive mode - user selected multiple skills
InitConfig {
    install_hooks: true,
    install_tracker: true,
    skills: vec![
        "skill-developer".to_string(),
        "backend-dev-guidelines".to_string(),
        "frontend-dev-guidelines".to_string(),
    ],
    force: false,
}

// Force reinstall all
InitConfig {
    install_hooks: true,
    install_tracker: true,
    skills: vec!["skill-developer".to_string()],
    force: true,  // Overwrites existing files
}

// Minimal install (no hooks, just skill)
InitConfig {
    install_hooks: false,
    install_tracker: false,
    skills: vec!["skill-developer".to_string()],
    force: false,
}
```

**Serialization:** Not serialized (command-line only).

---

## Report Structures

### InitReport

**Purpose:** Summarizes what was created/installed during `catalyst init`.

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `created_dirs` | `Vec<PathBuf>` | Directories created (`.claude/`, `.claude/hooks/`, etc.) |
| `installed_hooks` | `Vec<String>` | Hook wrapper scripts created |
| `installed_skills` | `Vec<String>` | Skill IDs successfully installed |
| `settings_created` | `bool` | Whether `settings.json` was created |
| `skill_rules_created` | `bool` | Whether `skill-rules.json` was created |
| `version_written` | `bool` | Whether `.catalyst-version` was written |

**Validation Rules:**
- `installed_skills` count must match `InitConfig.skills` count
- If `settings_created` is `true`, file must exist and be valid JSON
- If `skill_rules_created` is `true`, file must exist and be valid JSON

**Example Instance:**

```rust
InitReport {
    created_dirs: vec![
        PathBuf::from(".claude"),
        PathBuf::from(".claude/hooks"),
        PathBuf::from(".claude/skills"),
    ],
    installed_hooks: vec![
        "skill-activation-prompt.sh".to_string(),
        "file-change-tracker.sh".to_string(),
    ],
    installed_skills: vec![
        "skill-developer".to_string(),
        "backend-dev-guidelines".to_string(),
    ],
    settings_created: true,
    skill_rules_created: true,
    version_written: true,
}
```

**Serialization:** Not serialized (display only).

**Display Format:**
```
✅ Catalyst initialized successfully!

Created:
  ✓ .claude/
  ✓ .claude/hooks/
  ✓ .claude/skills/

Installed hooks:
  ✓ UserPromptSubmit → skill-activation-prompt.sh
  ✓ PostToolUse → file-change-tracker.sh

Installed skills:
  ✓ skill-developer
  ✓ backend-dev-guidelines
```

---

### UpdateReport

**Purpose:** Summarizes what changed during `catalyst update`.

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `old_version` | `String` | Version before update (from `.catalyst-version`) |
| `new_version` | `String` | Version after update (current binary version) |
| `wrappers_updated` | `Vec<String>` | Hook wrappers recreated |
| `skills_updated` | `Vec<String>` | Skills updated (hash matched) |
| `skills_skipped` | `Vec<String>` | Skills skipped (hash mismatch = user modified) |

**Validation Rules:**
- `old_version` must be valid semver
- `new_version` must be valid semver
- `skills_updated` + `skills_skipped` = all installed skills

**Example Instances:**

```rust
// Already up to date
UpdateReport {
    old_version: "0.1.0".to_string(),
    new_version: "0.1.0".to_string(),
    wrappers_updated: vec![],
    skills_updated: vec![],
    skills_skipped: vec![],
}

// Successful update
UpdateReport {
    old_version: "0.1.0".to_string(),
    new_version: "0.2.0".to_string(),
    wrappers_updated: vec![
        "skill-activation-prompt.sh".to_string(),
        "file-change-tracker.sh".to_string(),
    ],
    skills_updated: vec![
        "skill-developer".to_string(),
        "backend-dev-guidelines".to_string(),
    ],
    skills_skipped: vec![],
}

// Partial update (user modified a skill)
UpdateReport {
    old_version: "0.1.0".to_string(),
    new_version: "0.2.0".to_string(),
    wrappers_updated: vec![
        "skill-activation-prompt.sh".to_string(),
    ],
    skills_updated: vec![
        "skill-developer".to_string(),
    ],
    skills_skipped: vec![
        "backend-dev-guidelines".to_string(),  // User customized
    ],
}
```

**Display Format:**
```
✅ Updated to version 0.2.0

Wrappers updated:
  ✓ skill-activation-prompt.sh
  ✓ file-change-tracker.sh

Skills updated:
  ✓ skill-developer

⚠️  Skipped (modified locally):
  - backend-dev-guidelines
    Use --force to overwrite
```

---

### FixReport

**Purpose:** Summarizes auto-fix results from `catalyst status --fix`.

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `fixed` | `Vec<String>` | Issues successfully fixed |
| `failed` | `Vec<(String, String)>` | Issues that couldn't be fixed: (issue, reason) |

**Validation Rules:**
- At least one of `fixed` or `failed` must be non-empty
- Each `failed` entry must have a helpful reason

**Example Instances:**

```rust
// All issues fixed
FixReport {
    fixed: vec![
        "Recreated skill-activation-prompt.sh".to_string(),
        "Set executable permission on file-change-tracker.sh".to_string(),
    ],
    failed: vec![],
}

// Partial fix
FixReport {
    fixed: vec![
        "Set executable permission on file-change-tracker.sh".to_string(),
    ],
    failed: vec![
        (
            "Missing binary: skill-activation-prompt".to_string(),
            "Run ./install.sh in catalyst repository".to_string(),
        ),
    ],
}

// Nothing could be fixed
FixReport {
    fixed: vec![],
    failed: vec![
        (
            "Missing binaries".to_string(),
            "Run ./install.sh to install Catalyst binaries".to_string(),
        ),
    ],
}
```

**Display Format:**
```
✅ Fixed 2 issues:
  ✓ Recreated skill-activation-prompt.sh
  ✓ Set executable permission on file-change-tracker.sh

❌ Could not fix 1 issue:
  ✗ Missing binary: skill-activation-prompt
    → Run ./install.sh in catalyst repository
```

---

## Status Structures

### StatusReport

**Purpose:** Complete diagnostic report from `catalyst status`.

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `overall` | `HealthStatus` | Overall health: Healthy/Warning/Error |
| `binaries` | `Vec<BinaryStatus>` | Status of each required binary |
| `hooks` | `Vec<HookStatus>` | Status of each configured hook |
| `skills` | `Vec<SkillStatus>` | Status of each installed skill |
| `issues` | `Vec<Issue>` | List of problems found |

**Validation Rules:**
- `overall` must be consistent with `issues`:
  - `Healthy` if `issues` is empty
  - `Warning` if all issues are severity Warning
  - `Error` if any issue is severity Error

**Example Instance:**

```rust
StatusReport {
    overall: HealthStatus::Healthy,
    binaries: vec![
        BinaryStatus {
            name: "skill-activation-prompt".to_string(),
            variant: None,
            found: true,
            path: Some(PathBuf::from("/home/user/.claude-hooks/bin/skill-activation-prompt")),
            version: None,
        },
        BinaryStatus {
            name: "file-change-tracker".to_string(),
            variant: Some("sqlite".to_string()),
            found: true,
            path: Some(PathBuf::from("/home/user/.claude-hooks/bin/file-change-tracker")),
            version: None,
        },
    ],
    hooks: vec![
        HookStatus {
            hook_type: "UserPromptSubmit".to_string(),
            wrapper_path: PathBuf::from(".claude/hooks/skill-activation-prompt.sh"),
            wrapper_exists: true,
            executable: true,
            binary_accessible: true,
        },
    ],
    skills: vec![
        SkillStatus {
            id: "skill-developer".to_string(),
            installed: true,
            has_skill_md: true,
            in_rules: true,
        },
    ],
    issues: vec![],
}
```

**Display Format:** See plan.md page 522 for full example.

---

### BinaryStatus

**Purpose:** Status of a single required binary (skill-activation-prompt, file-change-tracker, file-analyzer).

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Binary name (without `.exe`) |
| `variant` | `Option<String>` | Variant if applicable ("sqlite" or "basic") |
| `found` | `bool` | Whether binary was found |
| `path` | `Option<PathBuf>` | Full path to binary if found |
| `version` | `Option<String>` | Binary version (None for MVP) |

**Validation Rules:**
- If `found` is `true`, `path` must be `Some`
- If `found` is `false`, `path` must be `None`
- `name` must be one of: `skill-activation-prompt`, `file-change-tracker`, `file-analyzer`
- For `file-change-tracker`, `variant` should be detected (sqlite vs basic)

**Example Instances:**

```rust
// Found binary
BinaryStatus {
    name: "skill-activation-prompt".to_string(),
    variant: None,
    found: true,
    path: Some(PathBuf::from("/home/user/.claude-hooks/bin/skill-activation-prompt")),
    version: None,  // MVP: version detection not implemented
}

// Found SQLite variant
BinaryStatus {
    name: "file-change-tracker".to_string(),
    variant: Some("sqlite".to_string()),
    found: true,
    path: Some(PathBuf::from("/home/user/.claude-hooks/bin/file-change-tracker")),
    version: None,
}

// Missing binary
BinaryStatus {
    name: "file-analyzer".to_string(),
    variant: None,
    found: false,
    path: None,
    version: None,
}
```

---

### HookStatus

**Purpose:** Status of a single configured hook (UserPromptSubmit or PostToolUse).

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `hook_type` | `String` | "UserPromptSubmit" or "PostToolUse" |
| `wrapper_path` | `PathBuf` | Path to wrapper script |
| `wrapper_exists` | `bool` | Whether wrapper file exists |
| `executable` | `bool` | Whether wrapper is executable (Unix only) |
| `binary_accessible` | `bool` | Whether wrapper can find the binary |

**Validation Rules:**
- `hook_type` must be valid hook type from settings.json
- If `wrapper_exists` is `false`, `executable` and `binary_accessible` are irrelevant
- On Windows, `executable` is always `true` (not applicable)

**Example Instances:**

```rust
// Healthy hook
HookStatus {
    hook_type: "UserPromptSubmit".to_string(),
    wrapper_path: PathBuf::from(".claude/hooks/skill-activation-prompt.sh"),
    wrapper_exists: true,
    executable: true,
    binary_accessible: true,
}

// Missing wrapper
HookStatus {
    hook_type: "PostToolUse".to_string(),
    wrapper_path: PathBuf::from(".claude/hooks/file-change-tracker.sh"),
    wrapper_exists: false,
    executable: false,
    binary_accessible: false,
}

// Wrapper exists but not executable
HookStatus {
    hook_type: "UserPromptSubmit".to_string(),
    wrapper_path: PathBuf::from(".claude/hooks/skill-activation-prompt.sh"),
    wrapper_exists: true,
    executable: false,  // ❌ Missing +x
    binary_accessible: true,
}
```

---

### SkillStatus

**Purpose:** Status of a single installed skill.

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Skill ID (directory name) |
| `installed` | `bool` | Whether skill directory exists |
| `has_skill_md` | `bool` | Whether `SKILL.md` exists |
| `in_rules` | `bool` | Whether skill is in `skill-rules.json` |

**Validation Rules:**
- If `installed` is `false`, other fields are irrelevant
- If `installed` is `true` but `has_skill_md` is `false`, skill is incomplete

**Example Instances:**

```rust
// Healthy skill
SkillStatus {
    id: "skill-developer".to_string(),
    installed: true,
    has_skill_md: true,
    in_rules: true,
}

// Incomplete skill (missing SKILL.md)
SkillStatus {
    id: "backend-dev-guidelines".to_string(),
    installed: true,
    has_skill_md: false,  // ❌ Missing SKILL.md
    in_rules: true,
}

// Not in rules (won't activate)
SkillStatus {
    id: "custom-skill".to_string(),
    installed: true,
    has_skill_md: true,
    in_rules: false,  // ❌ Not in skill-rules.json
}
```

---

### Issue

**Purpose:** Represents a single problem found during status check.

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `severity` | `IssueSeverity` | Error, Warning, or Info |
| `message` | `String` | Human-readable description |
| `suggestion` | `Option<String>` | How to fix (if known) |
| `auto_fixable` | `bool` | Whether `--fix` can resolve this |

**Validation Rules:**
- `message` must be clear and specific
- If `auto_fixable` is `true`, `suggestion` should be `Some`
- `severity` determines overall status

**Example Instances:**

```rust
// Auto-fixable error
Issue {
    severity: IssueSeverity::Error,
    message: "Wrapper script not executable: skill-activation-prompt.sh".to_string(),
    suggestion: Some("Run: chmod +x .claude/hooks/skill-activation-prompt.sh".to_string()),
    auto_fixable: true,
}

// Non-fixable error
Issue {
    severity: IssueSeverity::Error,
    message: "Binary not found: skill-activation-prompt".to_string(),
    suggestion: Some("Run: ./install.sh in catalyst repository".to_string()),
    auto_fixable: false,
}

// Warning
Issue {
    severity: IssueSeverity::Warning,
    message: "Skill 'backend-dev-guidelines' modified locally".to_string(),
    suggestion: Some("Update will skip this skill. Use --force to overwrite.".to_string()),
    auto_fixable: false,
}

// Info
Issue {
    severity: IssueSeverity::Info,
    message: "Using SQLite variant of file-change-tracker".to_string(),
    suggestion: None,
    auto_fixable: false,
}
```

---

## Platform Abstraction

### Platform

**Purpose:** Abstract over OS differences (paths, file extensions, environment variables).

**Enum Variants:**

| Variant | Description |
|---------|-------------|
| `Linux` | Linux native (not WSL) |
| `MacOS` | macOS / Darwin |
| `Windows` | Windows native |
| `WSL` | Windows Subsystem for Linux |

**Method Signatures:**

```rust
impl Platform {
    /// Detect current platform
    /// Priority: Check cfg!(windows), then WSL_DISTRO_NAME, then cfg!(target_os)
    fn detect() -> Self;

    /// File extension for wrapper scripts
    /// Returns: ".sh" for Unix/WSL, ".ps1" for Windows
    fn wrapper_extension(&self) -> &str;

    /// Environment variable for home directory
    /// Returns: "HOME" for Unix/WSL/macOS, "USERPROFILE" for Windows
    fn home_dir_var(&self) -> &str;

    /// Binary file extension
    /// Returns: "" for Unix/WSL/macOS, ".exe" for Windows
    fn binary_extension(&self) -> &str;

    /// Whether platform uses Unix-style permissions
    /// Returns: true for Linux/macOS/WSL, false for Windows
    fn has_permissions(&self) -> bool;
}
```

**Detection Algorithm (Pseudocode):**

```
fn detect() -> Platform {
    // 1. Check if Windows at compile time
    if cfg!(windows) {
        return Platform::Windows;
    }

    // 2. Check WSL_DISTRO_NAME environment variable (runtime)
    if env::var("WSL_DISTRO_NAME").is_ok() {
        return Platform::WSL;
    }

    // 3. Check target OS at compile time
    if cfg!(target_os = "macos") {
        return Platform::MacOS;
    } else if cfg!(target_os = "linux") {
        return Platform::Linux;
    }

    // 4. Fallback (should never happen)
    Platform::Linux
}
```

**Example Behavior:**

| Platform | `wrapper_extension()` | `home_dir_var()` | `binary_extension()` | `has_permissions()` |
|----------|----------------------|------------------|---------------------|-------------------|
| Linux | ".sh" | "HOME" | "" | true |
| MacOS | ".sh" | "HOME" | "" | true |
| Windows | ".ps1" | "USERPROFILE" | ".exe" | false |
| WSL | ".sh" | "HOME" | "" | true |

---

## Error Types

### CatalystError

**Purpose:** All error types that can occur in the CLI.

**Enum Variants:**

| Variant | Fields | Description |
|---------|--------|-------------|
| `BinariesNotInstalled` | `missing: Vec<String>`, `install_path: String` | Required binaries not found |
| `InitInProgress` | `pid: u32`, `lock_path: PathBuf` | Another init is running (lock file exists) |
| `InvalidPath` | `path: PathBuf`, `reason: String` | Path is invalid or inaccessible |
| `IoError` | `source: std::io::Error`, `context: String` | Filesystem operation failed |
| `JsonError` | `source: serde_json::Error`, `file: PathBuf` | JSON parsing/generation failed |
| `SkillNotFound` | `skill_id: String`, `available: Vec<String>` | Requested skill not in embedded resources |
| `SkillHashMismatch` | `skill_id: String` | Skill modified locally (hash mismatch) |
| `PathTraversalDetected` | `skill_id: String`, `attempted_path: PathBuf`, `reason: String` | Skill file path escapes `.claude/` directory (security) |
| `NoHomeDirectory` | - | Cannot determine home directory |
| `PermissionDenied` | `path: PathBuf`, `operation: String` | Insufficient permissions |
| `AlreadyInitialized` | `catalyst_version: String` | .catalyst-version exists, use --force |
| `NotInitialized` | - | No .catalyst-version found (for update/status) |
| `AtomicWriteFailed` | `path: PathBuf`, `fallback_used: bool` | Atomic write failed, fallback may have been used |

**Error Messages (Specifications):**

```rust
// BinariesNotInstalled
"Catalyst binaries not installed at {install_path}
Missing: {missing.join(", ")}

Please install binaries first:
  cd catalyst
  ./install.sh         # Linux/macOS
  ./install.ps1        # Windows

{if missing includes file-change-tracker:
Note: Use ./install.sh --sqlite if you need SQLite tracking
}"

// InitInProgress
"Another catalyst init is already running (PID: {pid})
Lock file: {lock_path}

If no other init is running, remove the lock file:
  rm {lock_path}"

// InvalidPath
"Invalid path: {path}
Reason: {reason}"

// IoError
"{context}
Error: {source}"

// JsonError
"Failed to parse JSON file: {file}
Error: {source}

Please check the file syntax."

// SkillNotFound
"Skill not found: {skill_id}

Available skills:
{available.iter().map(|s| format!("  - {}", s)).join("\n")}"

// SkillHashMismatch
"Skill '{skill_id}' has been modified locally.
Update will skip this skill to preserve your changes.

Use --force to overwrite with latest version."

// PathTraversalDetected
"Path traversal detected in skill: {skill_id}
Attempted path: {attempted_path}
Reason: {reason}

Skills must only write to: .claude/skills/{skill_id}/
This may indicate a compromised or malicious skill.

Security: Skill installation aborted."

// NoHomeDirectory
"Cannot determine home directory.
Please set HOME (Unix) or USERPROFILE (Windows) environment variable."

// PermissionDenied
"Permission denied: {operation} on {path}

Try:
  chmod +w {path}    # Unix/macOS
  (or run as administrator on Windows)"

// AlreadyInitialized
"Catalyst already initialized (version {catalyst_version}).

To reinstall:
  catalyst init --force"

// NotInitialized
"Catalyst not initialized in this directory.

Run:
  catalyst init"

// AtomicWriteFailed
"Failed to write file atomically: {path}
{if fallback_used:
⚠️  Used fallback write (not atomic). File may be corrupted on crash.
}{else:
File was not written.
}

Reason: This filesystem may not support atomic operations (network FS, Docker volume).
"
```

**Usage Examples:**

```rust
// Example 1: Missing binaries
return Err(CatalystError::BinariesNotInstalled {
    missing: vec!["skill-activation-prompt".to_string()],
    install_path: "~/.claude-hooks/bin/".to_string(),
});

// Example 2: Concurrent init
return Err(CatalystError::InitInProgress {
    pid: 12345,
    lock_path: PathBuf::from(".catalyst.lock"),
});

// Example 3: Skill not found
return Err(CatalystError::SkillNotFound {
    skill_id: "invalid-skill".to_string(),
    available: vec![
        "skill-developer".to_string(),
        "backend-dev-guidelines".to_string(),
    ],
});
```

---

## Supporting Types

### HealthStatus

**Purpose:** Overall health status for `StatusReport`.

**Enum Variants:**

| Variant | Description | Condition |
|---------|-------------|-----------|
| `Healthy` | All checks pass | No issues |
| `Warning` | Minor issues | All issues are Warning severity |
| `Error` | Critical issues | Any issue is Error severity |

**Display:**

| Status | Icon | Color |
|--------|------|-------|
| `Healthy` | ✅ | Green |
| `Warning` | ⚠️ | Yellow |
| `Error` | ❌ | Red |

---

### IssueSeverity

**Purpose:** Severity level for `Issue`.

**Enum Variants:**

| Variant | Description | Color |
|---------|-------------|-------|
| `Error` | Critical problem (blocks functionality) | Red |
| `Warning` | Minor problem (reduced functionality) | Yellow |
| `Info` | Informational message | Blue |

**Impact on Overall Status:**
- Any `Error` → Overall status is `Error`
- Only `Warning`s → Overall status is `Warning`
- Only `Info` → Overall status is `Healthy`

---

## Serialization Specifications

### Structures That Are Serialized

| Structure | Format | File | Purpose |
|-----------|--------|------|---------|
| *(None in this phase)* | - | - | All structures are runtime-only |

### Structures That Parse External JSON

| Structure | Source File | Parser |
|-----------|-------------|--------|
| *(See JSON Schemas doc)* | `settings.json`, `skill-rules.json` | TBD in Task 0.2 |

**Note:** While these structs don't serialize themselves, they generate JSON files. See `catalyst-cli-json-schemas.md` (Task 0.2) for the JSON specifications.

---

## Type Aliases & Constants

### Constants

```rust
// Binary names (without extension)
const BINARY_SKILL_ACTIVATION: &str = "skill-activation-prompt";
const BINARY_FILE_TRACKER: &str = "file-change-tracker";
const BINARY_FILE_ANALYZER: &str = "file-analyzer";

// File names
const FILE_SETTINGS: &str = "settings.json";
const FILE_SKILL_RULES: &str = "skill-rules.json";
const FILE_VERSION: &str = ".catalyst-version";
const FILE_HASHES: &str = ".catalyst-hashes.json";
const FILE_LOCK: &str = ".catalyst.lock";

// Directory names
const DIR_CLAUDE: &str = ".claude";
const DIR_HOOKS: &str = "hooks";
const DIR_SKILLS: &str = "skills";

// Hook types
const HOOK_USER_PROMPT: &str = "UserPromptSubmit";
const HOOK_POST_TOOL: &str = "PostToolUse";

// Default skills
const DEFAULT_SKILL: &str = "skill-developer";
const ALL_SKILLS: &[&str] = &[
    "skill-developer",
    "backend-dev-guidelines",
    "frontend-dev-guidelines",
    "route-tester",
    "error-tracking",
];

// Installation paths
const STANDALONE_BIN_DIR: &str = ".claude-hooks/bin";  // Relative to home
```

### Type Aliases

```rust
// For clarity in function signatures
type SkillId = String;
type BinaryName = String;
type HookType = String;
type FilePath = PathBuf;
```

---

## Validation Rules Summary

### Cross-Structure Rules

1. **Skill IDs must be consistent:**
   - `InitConfig.skills` must match embedded resources
   - `InitReport.installed_skills` must match `InitConfig.skills`
   - `SkillStatus.id` must match directory name

2. **Status consistency:**
   - `StatusReport.overall` must reflect `StatusReport.issues`
   - If any `Issue.severity` is `Error`, overall must be `Error`

3. **Path consistency:**
   - All `PathBuf` fields must be relative to project root
   - Binary paths must be absolute (in `~/.claude-hooks/bin/`)

4. **Version consistency:**
   - `UpdateReport.old_version` must match `.catalyst-version` content
   - `UpdateReport.new_version` must match `env!("CARGO_PKG_VERSION")`

---

## Completion Checklist

- [x] Define `InitConfig` struct
- [x] Define `InitReport` struct
- [x] Define `UpdateReport` struct
- [x] Define `FixReport` struct
- [x] Define `StatusReport` struct
- [x] Define `BinaryStatus` struct
- [x] Define `HookStatus` struct
- [x] Define `SkillStatus` struct
- [x] Define `Issue` struct
- [x] Define `Platform` enum with 4 variants (Linux, MacOS, Windows, WSL)
- [x] Define Platform method signatures
- [x] Define `CatalystError` enum with all variants
- [x] Define `HealthStatus` enum
- [x] Define `IssueSeverity` enum
- [x] Document validation rules
- [x] Provide example instances for all structures
- [x] Specify error messages
- [x] Define constants and type aliases
- [x] Document display formats

---

**End of Data Structures Specification**

Next: See `catalyst-cli-json-schemas.md` (Task 0.2) for JSON file specifications.

# Catalyst CLI - Context & Decisions

**Last Updated:** 2025-01-03 (After plan-reviewer fixes)
**Status:** Ready for Implementation
**Related Plan:** catalyst-cli-plan.md
**Task Tracker:** catalyst-cli-tasks.md

---

## Current State

### Existing Binaries

```
catalyst-cli/src/bin/
├── settings_manager.rs        # Will be renamed → catalyst.rs
├── post_tool_use_tracker_sqlite.rs  # Will be renamed → file_change_tracker.rs
├── skill_activation_prompt.rs # No changes
└── file_analyzer.rs           # No changes
```

### Current User Experience

**Manual 3-Step Process:**
1. Run `./install.sh` to build binaries (~45 seconds)
2. Manually create wrapper scripts with heredoc syntax
3. Manually edit settings.json with correct paths

**Problems:**
- 15-30 minute setup time
- 40-50% error rate (heredoc syntax, JSON editing, permissions)
- 600+ lines of documentation needed
- No validation or diagnostics

### Technology Stack

- **Language:** Rust (for performance and zero dependencies)
- **CLI Framework:** clap 4.x with derive macros
- **Error Handling:** thiserror for custom errors
- **File I/O:** Atomic writes with tempfile crate
- **Hashing:** SHA256 via sha2 crate
- **Interactive UI:** dialoguer + indicatif
- **Embedded Resources:** include_dir macro
- **Platforms:** Linux, macOS, Windows

---

## Key Architecture Decisions

### Decision 1: Unified CLI Binary

**Question:** Keep separate binaries or create unified catalyst CLI?

**Decision:** Create unified `catalyst` CLI with subcommands

**Rationale:**
- Better UX (single command to remember)
- Easier discovery (`catalyst --help` shows all operations)
- Industry standard (git, docker, kubectl all use subcommands)
- Backward compatible (settings commands preserved)

**Impact:**
- Rename `settings-manager` → `catalyst`
- Rename `post-tool-use-tracker-sqlite` → `file-change-tracker`
- Other binaries (skill-activation-prompt, file-analyzer) unchanged

---

### Decision 2: Embed Skills vs Download on Demand

**Question:** Should skills be embedded in binary or downloaded at init?

**Decision:** Embed skills at compile time using `include_dir!` macro

**Rationale:**
- **Pros:**
  - Zero network dependencies
  - Works offline
  - Fast initialization (no downloads)
  - Guaranteed version compatibility
  - Simple implementation

- **Cons:**
  - Larger binary size (~3-4MB vs ~2MB)
  - Skills can't be updated without recompiling

**Mitigation:** Binary size is acceptable for CLI tool. Update command handles skill refreshes.

**Trade-off:** Accepted 1-2MB binary increase for offline reliability.

---

### Decision 3: Language-Agnostic pathPatterns

**Question:** Should init detect project structure and customize pathPatterns automatically?

**Decision:** Use broad, permissive default patterns. User customizes post-init.

**Rationale:**
- **Project detection** complexity:
  - Monorepo vs single-app detection unreliable
  - Many edge cases (mixed languages, custom structures)
  - Detection heuristics fragile and maintenance-heavy

- **Broad patterns** approach:
  - Safe defaults: `["src/**/*", "lib/**/*", "app/**/*", "tests/**/*"]`
  - Works for most projects out-of-box
  - Users customize in skill-rules.json after init
  - Better to activate too often than never

**Impact:**
- No detection logic in Phase 5 (Interactive Mode)
- Add clear customization instructions in init summary
- Document pathPatterns customization in README and docs

**Deferred Feature:** Smart project detection moved to "Future Enhancements"

---

### Decision 4: Phase 0 is Specification-Only

**Question:** Should Phase 0 write code or just specifications?

**Decision:** Phase 0 is **specification-only** (no code compilation)

**Rationale:**
- Follows Rust best practice: design before implementation
- Allows specification review without code changes
- Prevents "specify twice, implement twice" confusion
- Clear separation between planning and coding

**Impact:**
- Phase 0 output: Struct definitions, schemas, pseudocode algorithms
- Actual implementations happen in phases 1-8
- Dependencies documented in Phase 0, added in Phase 1.1

---

### Decision 5: Atomic File Writes

**Question:** Direct file writes or atomic writes with tempfile?

**Decision:** Use atomic writes for all critical files (settings.json, skill files, wrappers)

**Rationale:**
- Prevents corruption from partial writes
- Safe against crashes mid-write
- Follows Rust best practices (see rust-developer skill)
- Minimal complexity overhead (tempfile crate handles details)

**Implementation:** Use `NamedTempFile::persist()` pattern

---

### Decision 6: Binary Version Detection

**Question:** Should `get_binary_version()` parse binary metadata?

**Decision:** **Return `None` for MVP**. Implement in future enhancement.

**Rationale:**
- Parsing binary metadata is complex (platform-specific formats)
- Low value for MVP (version tracked in .catalyst-version file)
- Can add later without breaking changes

**Impact:** Status command won't show binary versions in v0.1.0

---

### Decision 7: Skill Update Conflicts

**Question:** How to handle 3-way merge when user modified skill AND upstream changed?

**Decision:** **Skip update, warn user, suggest manual merge**

**Rationale:**
- Automated merge is complex and error-prone
- User customizations are valuable (shouldn't be overwritten)
- Manual review ensures intentional changes

**Implementation:**
- Hash-based detection (compare current hash to stored hash)
- If modified: Add to `skills_skipped` list
- Show warning: "⚠️ Skipped skill-developer (modified locally). Use --force to overwrite."
- `--force` flag allows overwrite if user confirms

---

### Decision 8: pathPatterns Validation

**Question:** Should we validate pathPatterns syntax on init?

**Decision:** **No validation for MVP**. Add in Phase 4 status check (optional).

**Rationale:**
- pathPatterns are glob patterns (permissive syntax)
- Invalid patterns fail silently (safe degradation)
- Validation complexity not worth it for MVP

**Future:** Status command could check if patterns match any files

---

### Decision 9: Windows PowerShell Version

**Question:** Support both `powershell.exe` (5.1) and `pwsh` (7+)?

**Decision:** **Use PowerShell Core 7+** for Windows wrapper scripts. Document requirements.

**Rationale:**
- PowerShell Core is cross-platform (Windows/Linux/macOS)
- Modern, actively maintained
- Better shell scripting features
- **Critical:** Windows doesn't support shebangs - scripts invoked directly by extension

**Windows-Specific Requirements:**
```powershell
# Users must set execution policy once:
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned

# Scripts use .ps1 extension (no shebang needed)
# Invoked as: powershell -File script.ps1
```

**Documentation:**
- Installation instructions will note PowerShell 7+ requirement on Windows
- Add execution policy setup to Windows installation guide
- Wrapper templates must NOT include shebang on Windows (line 684 of old plan was wrong)

---

### Decision 10: .catalyst-version in .gitignore

**Question:** Should init add `.catalyst-version` to .gitignore automatically?

**Decision:** **No**. Let user decide. Document that it's safe to commit.

**Rationale:**
- Committing version is useful for teams (everyone knows installation version)
- Automatic .gitignore modification is intrusive
- User can choose based on their workflow

**Documentation:** README will note: "Safe to commit `.catalyst-version` for team consistency"

---

### Decision 11: WSL Detection and Platform Enum Enhancement

**Question:** Should Platform enum distinguish between Linux, macOS, Windows, and WSL?

**Decision:** **Yes**. Extend Platform enum to include WSL as a distinct platform.

**Rationale:**
- `cfg!(windows)` returns `false` in WSL, but WSL has unique characteristics
- WSL can execute both Linux and Windows binaries
- WSL uses Linux-style paths (`/home/`) but can access Windows paths (`/mnt/c/`)
- Different home directory conventions than native Windows
- Wrapper scripts need `.sh` extension (Unix-style) not `.ps1`

**Implementation:**
```rust
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    WSL,    // Windows Subsystem for Linux
}

impl Platform {
    pub fn detect() -> Self {
        if cfg!(windows) {
            Platform::Windows
        } else if std::env::var("WSL_DISTRO_NAME").is_ok() {
            Platform::WSL
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        }
    }

    pub fn wrapper_extension(&self) -> &str {
        match self {
            Platform::Windows => ".ps1",
            Platform::Linux | Platform::MacOS | Platform::WSL => ".sh",
        }
    }

    pub fn home_dir_var(&self) -> &str {
        match self {
            Platform::Windows => "USERPROFILE",
            Platform::Linux | Platform::MacOS | Platform::WSL => "HOME",
        }
    }
}
```

**Impact:** Phase 1.4 implementation must include WSL detection.

---

### Decision 12: SQLite Feature Coordination

**Question:** How should `catalyst init` handle the SQLite feature flag for file-change-tracker?

**Decision:** **Detect both variants** and handle gracefully.

**Rationale:**
- `file-change-tracker` binary requires `--features sqlite` to enable database functionality
- Some users may build without SQLite (lighter dependency footprint)
- `install.sh` supports both: `./install.sh` (no SQLite) and `./install.sh --sqlite`

**Implementation:**
```rust
fn check_file_change_tracker() -> BinaryStatus {
    let home = get_home_dir()?;
    let bin_dir = home.join(".claude-hooks/bin");

    // Try SQLite version first (preferred)
    let sqlite_binary = bin_dir.join("file-change-tracker");
    if sqlite_binary.exists() {
        return BinaryStatus {
            name: "file-change-tracker".to_string(),
            found: true,
            variant: Some("sqlite".to_string()),
            path: Some(sqlite_binary),
        };
    }

    // Fallback to non-SQLite version
    let basic_binary = bin_dir.join("file-change-tracker-basic");
    if basic_binary.exists() {
        return BinaryStatus {
            name: "file-change-tracker".to_string(),
            found: true,
            variant: Some("basic".to_string()),
            path: Some(basic_binary),
        };
    }

    // Not found
    BinaryStatus {
        name: "file-change-tracker".to_string(),
        found: false,
        variant: None,
        path: None,
    }
}
```

**Impact:**
- Binary validation (Phase 4.2) must detect both variants
- Error messages should suggest correct install.sh flags

---

### Decision 13: Concurrent Init Protection

**Question:** How to prevent multiple simultaneous `catalyst init` executions from corrupting state?

**Decision:** **Use filesystem-based advisory locking** with `.catalyst.lock` file.

**Rationale:**
- Multiple concurrent inits could create partial/inconsistent state
- Filesystem locking is cross-platform and doesn't require additional dependencies
- Advisory lock (not mandatory) is sufficient for this use case
- Lock file includes PID for debugging

**Implementation:**
```rust
use std::fs::{File, OpenOptions};
use std::io::Write;

fn acquire_init_lock(claude_dir: &Path) -> Result<File, CatalystError> {
    let lock_path = claude_dir.join(".catalyst.lock");

    // Try to create lock file exclusively
    let mut lock_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                CatalystError::InitInProgress {
                    lock_file: lock_path.clone(),
                }
            } else {
                CatalystError::Io(e)
            }
        })?;

    // Write PID for debugging
    writeln!(lock_file, "{}", std::process::id())?;

    Ok(lock_file)
}

fn release_init_lock(claude_dir: &Path) {
    let lock_path = claude_dir.join(".catalyst.lock");
    let _ = std::fs::remove_file(lock_path);
}
```

**Impact:**
- Phase 2.1 must acquire lock before directory creation
- Lock released on completion or error
- Error variant added to `CatalystError` enum

---

### Decision 14: Atomic Write Fallback Strategy

**Question:** What happens when atomic writes fail on network filesystems or Docker volumes?

**Decision:** **Fallback to regular write** with warning if `NamedTempFile::persist()` fails with specific error codes.

**Rationale:**
- Atomic writes via `persist()` can fail on:
  - Network filesystems (NFS, SMB)
  - Some Docker volume configurations
  - Cross-device boundaries in WSL
- Better to complete init with warning than fail completely
- User warned that file may be corrupted if crash occurs during write

**Implementation:**
```rust
use std::io::ErrorKind;

fn write_file_atomic(path: &Path, content: &[u8]) -> Result<(), CatalystError> {
    let parent = path.parent().ok_or(CatalystError::InvalidPath)?;

    // Try atomic write first
    match NamedTempFile::new_in(parent) {
        Ok(mut temp) => {
            temp.write_all(content)?;
            match temp.persist(path) {
                Ok(_) => Ok(()),
                Err(e) if is_cross_device(&e) => {
                    // Fallback to regular write
                    eprintln!("⚠️  Warning: Atomic write not supported on this filesystem");
                    eprintln!("   Falling back to regular write (less safe)");
                    std::fs::write(path, content)?;
                    Ok(())
                }
                Err(e) => Err(CatalystError::from(e.error)),
            }
        }
        Err(e) => {
            // Temp file creation failed, use regular write
            eprintln!("⚠️  Warning: Temporary file creation failed");
            eprintln!("   Using regular write (less safe)");
            std::fs::write(path, content)?;
            Ok(())
        }
    }
}

fn is_cross_device(e: &PersistError) -> bool {
    matches!(e.error.kind(), ErrorKind::Other)
        && e.error.raw_os_error() == Some(18) // EXDEV on Unix
}
```

**Impact:** Phase 2 file creation must use this pattern with fallback.

---

## Key Files & Locations

### Files to be Created

```
catalyst-cli/
├── src/
│   ├── bin/
│   │   ├── catalyst.rs           # NEW: Main CLI (renamed)
│   │   └── file_change_tracker.rs  # NEW: Renamed binary
│   ├── commands/                  # NEW: Command modules
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── status.rs
│   │   ├── update.rs
│   │   └── settings.rs
│   └── resources/                 # NEW: Templates
│       ├── wrapper-template.sh
│       └── wrapper-template.ps1
└── Cargo.toml                     # Update dependencies

.github/workflows/
└── ci.yml                         # NEW: Cross-platform CI

docs/
├── catalyst-cli.md                # NEW: CLI reference
└── schemas/                       # NEW: JSON examples
    ├── settings.json
    ├── skill-rules.json
    └── catalyst-hashes.json
```

### Files Generated by `catalyst init`

```
project/.claude/
├── settings.json                  # Hook configurations
├── hooks/
│   ├── skill-activation-prompt.sh  # Wrapper
│   └── file-change-tracker.sh      # Wrapper
└── skills/
    ├── skill-rules.json           # Activation rules
    ├── .catalyst-version          # Version tracking
    ├── .catalyst-hashes.json      # Modification detection
    └── [skill-id]/                # Installed skills
        ├── SKILL.md
        └── resources/
```

---

## Dependencies

### New Crate Dependencies

Added in Phase 1.1:

```toml
[dependencies]
# EXISTING - Already used in types.rs:469 (CatalystHashes::new())
# CRITICAL FIX: Currently feature-gated, must be standard dependency
chrono = { workspace = true }

# NEW - Skill hashing (Phase 3.4)
sha2 = "0.10"

# NEW - Interactive mode (Phase 5)
dialoguer = { version = "0.11", features = ["completion"] }
indicatif = "0.17"

# NEW - Embedded skills (Phase 3.1)
include_dir = "0.7"

# NEW - Better cross-platform support
dirs = "5.0"    # Cross-platform home/config directory detection
dunce = "1.0"   # Windows path canonicalization (UNC paths)

# Testing (Phase 8)
[dev-dependencies]
tempfile = "3.14"
```

**Critical Note:** `chrono` is already in use at `catalyst-cli/src/types.rs:469` in `CatalystHashes::new()`. It must be changed from an optional (feature-gated) dependency to a standard dependency in Phase 1.1 to avoid compilation errors.

### External Dependencies

- **Rust toolchain:** 1.70+ (for workspace dependencies)
- **PowerShell Core 7+:** Windows users only
- **Git:** For repository operations (optional, for update command future enhancement)

---

## Data Structures

### Core Configuration Types

```rust
// What user wants to install
struct InitConfig {
    install_hooks: bool,
    install_tracker: bool,
    skills: Vec<String>,
    force: bool,
}

// What was actually installed
struct InitReport {
    created_dirs: Vec<String>,
    installed_hooks: Vec<String>,
    installed_skills: Vec<String>,
    settings_created: bool,
}

// Platform detection (see Decision 11)
enum Platform {
    Linux,
    MacOS,
    Windows,
    WSL,    // Windows Subsystem for Linux
}

// Binary status with SQLite variant detection (see Decision 12)
struct BinaryStatus {
    name: String,
    found: bool,
    variant: Option<String>,  // "sqlite" or "basic" for file-change-tracker
    path: Option<PathBuf>,
}

// Error handling with new variants
#[derive(Error, Debug)]
enum CatalystError {
    #[error("Catalyst binaries not installed at {install_path}\nMissing: {}\nRun: cd {repo_path} && ./install.sh{}",
        missing.join(", "),
        if missing.contains(&"file-change-tracker".to_string()) { " --sqlite" } else { "" })]
    BinariesNotInstalled {
        missing: Vec<String>,
        install_path: String,
        repo_path: String,  // Added for better error message
    },

    #[error("Skill not found: {0}\nAvailable: skill-developer, backend-dev-guidelines, frontend-dev-guidelines, route-tester, error-tracking")]
    SkillNotFound(String),

    #[error(".claude directory already exists\nUse --force to reinitialize or 'catalyst status' to check current setup")]
    AlreadyInitialized,

    #[error("Catalyst init already in progress\nLock file: {}\nIf no other init is running, remove the lock file manually", lock_file.display())]
    InitInProgress { lock_file: PathBuf },  // NEW (Decision 13)

    #[error("Invalid path: {0}")]
    InvalidPath,  // NEW (Decision 14)

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

### JSON Schemas

**settings.json:**
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [{
          "type": "command",
          "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
        }]
      }
    ],
    "PostToolUse": [
      {
        "matchers": ["Edit", "MultiEdit", "Write", "NotebookEdit"],
        "hooks": [{
          "type": "command",
          "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/file-change-tracker.sh"
        }]
      }
    ]
  }
}
```

**skill-rules.json:**
```json
{
  "version": "1.0",
  "skills": {
    "skill-developer": {
      "type": "UserPromptSubmit",
      "enforcement": "suggest",
      "priority": "high",
      "keywords": ["skill", "create skill"],
      "intentPatterns": ["wants to create a skill"],
      "pathPatterns": ["src/**/*", "lib/**/*", "app/**/*", "tests/**/*"],
      "enabled": true
    }
  }
}
```

---

## Critical Patterns

### Atomic File Writes

```rust
use tempfile::NamedTempFile;

fn write_settings(path: &Path, content: &str) -> Result<(), CatalystError> {
    let parent = path.parent().ok_or(...)?;
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.persist(path)?;
    Ok(())
}
```

### Platform Detection

```rust
impl Platform {
    fn detect() -> Self {
        if cfg!(windows) {
            Platform::Windows
        } else {
            Platform::Unix
        }
    }

    fn wrapper_extension(&self) -> &str {
        match self {
            Platform::Unix => ".sh",
            Platform::Windows => ".ps1",
        }
    }
}
```

### Skill Hash Tracking

```rust
use sha2::{Sha256, Digest};

fn hash_file(path: &Path) -> Result<String, CatalystError> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
```

---

## Risk Mitigation

### Risk 1: Platform-Specific Bugs

**Mitigation:**
- CI testing on Linux, macOS, Windows
- Platform abstraction layer (Platform enum)
- Conditional compilation where needed (`#[cfg(unix)]`)
- Templates for both Unix (.sh) and Windows (.ps1)

### Risk 2: Binary Size Bloat

**Assessment:** Skills total ~500KB text. Binary size ~3-4MB (acceptable).

**Mitigation:** If size becomes issue, future enhancement can download skills on demand.

### Risk 3: Embedded Resources Out of Sync

**Mitigation:**
- Skills embedded from `.claude/skills/` at compile time
- CI test ensures skills directory exists
- Build fails early if skills missing

### Risk 4: Breaking Existing Installations

**Assessment:** Low risk (no current users)

**Mitigation:**
- No migration needed for v0.1.0
- `--force` flag allows re-init if needed
- Status command diagnoses and fixes issues

---

## Timeline & Effort Estimates

**Total Duration:** 5-6 weeks (22 days implementation + 4 days buffer)

**Updated after plan-reviewer feedback**: Phase 0 expanded by 1 day to address critical issues (WSL detection, SQLite coordination, concurrent protection, atomic write fallbacks, Windows PowerShell fixes).

**Phase Breakdown:**
- Phase 0: Specifications (3 days) - M **[EXPANDED +1 day]**
  - Added Task 0.5: Cross-platform & safety specifications
- Phase 1: Foundation (3 days) - M
- Phase 2: Core Setup (3 days) - M
- Phase 3: Skills (2 days) - M
- Phase 4: Validation (2 days) - L
- Phase 5: Interactive (1 day) - S
- Phase 6: Update (2 days) - M
- Phase 7: Polish (2 days) - S
- Phase 8: Testing (3 days) - L
- Phase 9: Docs (1 day) - S

**Critical Path:** Phase 0 → 1 → 2 → 3 → 4 (13 days) **[+1 day]**

---

## Success Metrics

### Quantitative

- Setup time: 15-30 min → 30-60 seconds (30x faster)
- Error rate: 40-50% → <5%
- User steps: 3 manual → 1 automated
- Documentation: 600+ lines → 100 lines quick start

### Qualitative

- ✅ First-time users can set up without reading docs
- ✅ Issues automatically detected and fixable
- ✅ Works on all platforms without platform-specific instructions
- ✅ Updates preserve user customizations

---

## Future Enhancements (Post-MVP)

### High Priority

- `catalyst add-skill <skill-id>` - Install skills after init
- `catalyst remove-skill <skill-id>` - Uninstall skills
- `catalyst doctor` - Deep diagnostics with repair suggestions

### Medium Priority

- Smart project structure detection (deferred from Phase 5)
- Language-specific pathPattern presets
- `catalyst upgrade` - In-place binary upgrades from GitHub releases

### Low Priority

- Custom skill support (install from URLs)
- Skill marketplace
- Telemetry (opt-in)

---

**End of Context Document**

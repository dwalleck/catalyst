# Catalyst CLI Implementation Plan

**Last Updated:** 2025-01-03 (After plan-reviewer fixes)
**Status:** Ready for Implementation
**Estimated Duration:** 5-6 weeks (22 days + 4 days buffer)
**Goal:** Transform Catalyst from manual 3-step installation into single-command setup with `catalyst init`

**Related Files:**
- **Context:** catalyst-cli-context.md (decisions, current state, architecture)
- **Tasks:** catalyst-cli-tasks.md (actionable checklist)

---

## Executive Summary

### The Problem

Catalyst currently requires a manual 3-step installation process:

1. **Install binaries** (~45 seconds): Run `./install.sh`
2. **Create wrapper scripts** (manual, error-prone): Write heredoc scripts with correct paths
3. **Edit settings.json** (manual JSON editing): Add hook configurations

**Result:**
- 15-30 minute setup time for first-time users
- 40-50% error rate (heredoc syntax, JSON structure, file permissions)
- 600+ lines of documentation needed to guide users through process
- No validation - users don't know if setup worked

### The Solution

Create a unified `catalyst` CLI that automates the entire setup:

```bash
# Before: 3 manual steps, 15-30 minutes
cd catalyst && ./install.sh
cd project/.claude/hooks && cat > script.sh << 'EOF' ...
vi .claude/settings.json

# After: 1 command, 30-60 seconds
catalyst init
```

**Key Features:**
- **Single command** setup with validation
- **Interactive mode** for guided setup
- **Status command** for diagnostics (`catalyst status`)
- **Auto-fix** capability (`catalyst status --fix`)
- **Update command** preserving user customizations
- **Cross-platform** (Linux, macOS, Windows)

---

## Proposed Future State

### User Experience

```bash
# First-time setup (one-time, 45 seconds)
cd catalyst
./install.sh

# Initialize in any project (30 seconds)
cd my-project
catalyst init

# Or with options
catalyst init --interactive        # Guided setup
catalyst init --backend --frontend # Specific skills
catalyst init --all                # All skills

# Validate setup
catalyst status

# Fix issues automatically
catalyst status --fix

# Update to latest
catalyst update
```

### What Gets Created

```
project/.claude/
├── settings.json              # Hook configurations (auto-generated)
├── hooks/
│   ├── skill-activation-prompt.sh    # Wrappers (auto-generated)
│   └── file-change-tracker.sh
└── skills/
    ├── skill-rules.json       # Activation rules (auto-generated)
    ├── .catalyst-version      # Version tracking
    ├── .catalyst-hashes.json  # Modification detection
    └── skill-developer/       # Installed skills (embedded)
        ├── SKILL.md
        └── resources/
```

---

## Architecture Overview

### Binary Reorganization

**Rename for Clarity:**
- `settings-manager` → `catalyst` (unified CLI)
- `post-tool-use-tracker-sqlite` → `file-change-tracker` (clearer purpose)

**Command Structure:**
```
catalyst
├── init [options]           # Setup automation
├── status [--fix]           # Validation & diagnostics
├── update [--force]         # Maintenance
└── settings                 # Existing commands (backward compatible)
    ├── read
    ├── validate
    └── ... (preserved)
```

### Core Components

**1. Embedded Skills**
Skills bundled in binary using `include_dir!` macro:
- Zero network dependencies (works offline)
- Fast initialization (no downloads)
- Guaranteed version compatibility
- Binary size: ~3-4MB (acceptable)

**2. Platform Abstraction**
Unified handling for Unix and Windows:
```rust
enum Platform { Unix, Windows }
impl Platform {
    fn detect() -> Self { ... }
    fn wrapper_extension(&self) -> &str { ".sh" or ".ps1" }
    fn home_dir_var(&self) -> &str { "HOME" or "USERPROFILE" }
}
```

**3. Atomic File Operations**
Safe writes using `tempfile` crate:
- Prevents corruption from partial writes
- Safe against crashes mid-operation
- Follows Rust best practices

**4. Hash-Based Change Detection**
Track user modifications to skills:
```
.catalyst-hashes.json stores SHA256 of each file
→ Update command compares current vs stored hashes
→ Skip modified files, update unmodified ones
→ --force flag overrides if needed
```

---

## Implementation Phases

### Phase 0: Foundation & Specifications (2 days)

**Purpose:** Define all data structures, schemas, and algorithms BEFORE coding

**Deliverables:**
- All struct definitions documented (InitConfig, StatusReport, etc.)
- JSON schemas specified (settings.json, skill-rules.json)
- Algorithms in pseudocode (status determination, auto-fix decision tree)
- Helper function signatures (get_home_dir, hash_file, is_executable)
- Dependencies listed

**Note:** This phase is **specification-only**. No Rust code written.

---

### Phase 1: Rename & Foundation (3 days)

**Goals:**
- Rename binaries for clarity
- Add new dependencies to Cargo.toml
- Restructure CLI with subcommands (init, status, update, settings)
- Implement platform detection
- Implement binary validation check

**Key Milestone:** `catalyst --help` shows new command structure

---

### Phase 2: Directory & File Creation (3 days)

**Goals:**
- Create `.claude/` directory structure
- Generate wrapper scripts from templates (Unix .sh, Windows .ps1)
- Create settings.json with hook configurations
- Set proper file permissions (executable on Unix)

**Key Milestone:** `catalyst init` creates all necessary files

---

### Phase 3: Skill Installation (2 days)

**Goals:**
- Embed skills at compile time (`include_dir!`)
- Install skills from embedded resources to target directory
- Generate skill-rules.json with activation configurations
- Implement hash tracking (.catalyst-hashes.json)

**Key Milestone:** Skills auto-activate after init

---

### Phase 4: Validation & Status (2 days)

**Goals:**
- Implement status command (binary check, hook check, skill check)
- Report issues with severity levels (Error/Warning/Info)
- Implement auto-fix logic (recreate wrappers, set permissions)
- Format output with colors and icons

**Key Milestone:** `catalyst status --fix` repairs common issues

---

### Phase 5: Interactive Mode (1 day)

**Goals:**
- Add interactive prompts (directory, hooks, tracker, skills)
- Multi-select skill chooser
- Show summary before proceeding
- Add progress indicators for skill installation

**Key Milestone:** `catalyst init --interactive` guides users through setup

---

### Phase 6: Update Command (2 days)

**Goals:**
- Track installation version (.catalyst-version file)
- Update wrapper scripts
- Update skills (hash-based detection of modifications)
- Skip modified skills, update unmodified ones

**Key Milestone:** `catalyst update` refreshes installation preserving customizations

---

### Phase 7: Polish & UX (2 days)

**Goals:**
- Improve error messages (helpful, actionable)
- Add colored output (green success, red error, yellow warning)
- Create init summary report
- Enhance help text

**Key Milestone:** Professional, polished user experience

---

### Phase 8: Testing (3 days)

**Goals:**
- Write unit tests (directory creation, wrappers, platform detection, hashing)
- Write integration tests (full init flow, status, update, auto-fix)
- Set up cross-platform CI (Linux, macOS, Windows)

**Key Milestone:** All tests pass on all platforms

---

### Phase 9: Documentation (1 day)

**Goals:**
- Update README.md with Quick Start
- Create docs/catalyst-cli.md (complete CLI reference)
- Document pathPatterns customization
- Update existing docs with new binary names

**Key Milestone:** Users can get started in <5 minutes from documentation

---

## Success Criteria

### Must Have (MVP)

- ✅ `catalyst init` creates working setup in 30-60 seconds
- ✅ `catalyst status` validates installation and reports issues
- ✅ `catalyst status --fix` repairs common issues automatically
- ✅ `catalyst update` refreshes installation without breaking customizations
- ✅ Works on Linux, macOS, and Windows
- ✅ Skills auto-activate after init
- ✅ Error rate <5% (down from 40-50%)

### Nice to Have (Future)

- Interactive mode with smart defaults
- Binary version detection
- Skill marketplace / custom skill support
- `catalyst doctor` deep diagnostics

---

## Risk Assessment

### High Impact Risks

**1. Platform-Specific Bugs**
*Likelihood:* Medium
*Mitigation:*
- CI testing on all 3 platforms
- Platform abstraction layer
- Template-based wrapper generation

**2. Breaking Existing Installations**
*Likelihood:* Low (no current users)
*Mitigation:*
- No migration needed for v0.1.0
- `--force` flag for re-init
- Status command for diagnostics

### Medium Impact Risks

**3. Binary Size Bloat**
*Likelihood:* High (embedding skills)
*Impact Assessment:* 3-4MB is acceptable for CLI
*Mitigation:* If needed, future enhancement can download on demand

**4. Embedded Resources Out of Sync**
*Likelihood:* Low
*Mitigation:*
- Skills embedded from source at compile time
- CI test ensures skills directory exists

---

## Dependencies

### External

- Rust 1.70+ (workspace dependencies support)
- PowerShell Core 7+ (Windows only)

### New Crates

```toml
sha2 = "0.10"                     # Skill hashing
dialoguer = "0.11"                # Interactive prompts
indicatif = "0.17"                # Progress bars
include_dir = "0.7"               # Embed skills

[dev-dependencies]
tempfile = "3.14"                 # Test isolation
```

---

## Timeline

**Updated after plan-reviewer feedback**: Additional specifications for WSL, SQLite coordination, concurrent protection, atomic write fallbacks, and Windows PowerShell fixes add ~1 day to Phase 0.

### Week 1: Foundation
- Days 1-3: **Phase 0 (Specifications)** - EXPANDED from 2 to 3 days
  - Added Task 0.5: Cross-platform & safety specifications
  - WSL detection, SQLite coordination, concurrent protection, atomic write fallbacks
- Days 4-5: Phase 1 (Rename, CLI structure, platform detection with WSL)

### Week 2: Core Setup
- Days 1-3: Phase 2 (Directories with lock files, wrappers with fallbacks, settings.json)
- Days 4-5: Phase 3 (Skill embedding, installation, hashing)

### Week 3: Validation & Interactive
- Days 1-2: Phase 4 (Status command with SQLite detection, validation, auto-fix)
- Day 3: Phase 5 (Interactive mode)
- Days 4-5: Phase 6 (Update command)

### Week 4: Polish & Testing
- Days 1-2: Phase 7 (Error messages, colors, UX)
- Days 3-5: Phase 8 (Unit tests with WSL/concurrency/fallback coverage, integration tests, CI)

### Week 5: Documentation & Buffer
- Day 1: Phase 9 (Documentation with Windows execution policy notes)
- Days 2-5: Buffer for bug fixes and polish

**Total: 22 days implementation + 4 days buffer = 26 days across 5-6 weeks**

**Critical fixes addressed:**
- ✅ Chrono dependency conflict resolved
- ✅ Windows PowerShell wrapper templates fixed (no shebangs)
- ✅ WSL platform detection added
- ✅ SQLite feature coordination specified
- ✅ Concurrent init protection added
- ✅ Atomic write fallback strategy defined

---

## Key Design Principles

### 1. **Fail Fast with Helpful Errors**

```rust
#[error("Catalyst binaries not installed at {install_path}\nMissing: {}\nRun: ./install.sh", missing.join(", "))]
BinariesNotInstalled {
    missing: Vec<String>,
    install_path: String,
}
```

Every error includes:
- What went wrong
- Context (paths, missing items)
- How to fix it

### 2. **Idempotent Operations**

Running `catalyst init` twice is safe:
- Check before create (skip if exists)
- `--force` flag for intentional overwrite
- Clear messaging about what's skipped

### 3. **Progressive Disclosure**

Start simple, add complexity as needed:
- `catalyst init` - sensible defaults
- `catalyst init --interactive` - guided setup
- `catalyst init --backend --frontend` - specific skills
- `catalyst init --all` - everything

### 4. **Validate Early, Validate Often**

- Binary check before init
- JSON validation after generation
- Status command for post-init verification
- Auto-fix for common issues

### 5. **Cross-Platform First**

Never assume platform:
- Platform detection abstraction
- Conditional compilation where needed
- Template-based generation
- CI testing on all platforms

---

## Future Enhancements (Post-MVP)

### High Priority

- `catalyst add-skill <skill-id>` - Add skills after init
- `catalyst remove-skill <skill-id>` - Remove unwanted skills
- `catalyst doctor` - Deep diagnostics with repair suggestions
- **Unit tests for interactive mode** - Test `run_interactive_init()` with mocked inputs
  ```rust
  #[test]
  fn test_user_cancels_at_directory_prompt() {
      // Mock dialoguer to return false
      // Assert Ok(None) returned
  }
  ```

### Medium Priority

- Smart project structure detection (monorepo vs single-app)
- Language-specific pathPattern presets
- `catalyst upgrade` - In-place binary upgrades from GitHub releases
- **Configuration profiles** - Save and reuse init configs with presets
  ```bash
  catalyst init --interactive --save-as my-preset
  catalyst init --preset my-preset
  ```
- **Dynamic separator width** - Make terminal UI responsive to terminal width
  ```rust
  // Instead of const SEPARATOR_WIDTH: usize = 60;
  let separator_width = terminal_size().map(|(w, _)| w).unwrap_or(60);
  ```
- **Skill preview in interactive mode** - Show detailed info on request
  ```
  Press 'i' for more info on highlighted skill
  ```
- **Custom progress bar themes** - Allow styling via environment variable
  ```bash
  CATALYST_THEME=minimal catalyst init --interactive
  ```

### Low Priority

- Custom skill support (install from URLs or local paths)
- Skill marketplace (browse and install community skills)
- Telemetry (opt-in usage analytics)

---

## Communication Strategy

### Init Summary

After successful init, show:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Catalyst initialized successfully!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

Next steps:
  1. Customize .claude/skills/skill-rules.json pathPatterns for your project
  2. Review .claude/settings.json
  3. Try editing a file - skills should activate automatically
  4. Run 'catalyst status' to validate setup

Documentation: https://github.com/dwalleck/catalyst
```

### Status Output

```
✅ Catalyst Status: HEALTHY

Binaries:
  ✓ skill-activation-prompt (found)
  ✓ file-change-tracker (found)
  ✓ file-analyzer (found)

Hooks:
  ✓ UserPromptSubmit → skill-activation-prompt.sh
  ✓ PostToolUse → file-change-tracker.sh

Skills:
  ✓ skill-developer (installed)
  ✓ backend-dev-guidelines (installed)

Issues: None

All systems operational!
```

---

**End of Strategic Plan**

See **catalyst-cli-tasks.md** for actionable checklist.
See **catalyst-cli-context.md** for detailed decisions and architecture.

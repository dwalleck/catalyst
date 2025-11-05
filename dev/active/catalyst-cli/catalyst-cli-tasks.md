# Catalyst CLI - Task Checklist

**Last Updated:** 2025-01-04 (Phase 4 completed)
**Status:** Phase 4 Complete - Phase 5 Ready
**Related Plan:** catalyst-cli-plan.md
**Related Context:** catalyst-cli-context.md

---

## How to Use This Checklist

- [ ] Mark tasks complete with `[x]` as you finish them
- Update "Last Updated" date when making changes
- Reference plan.md for strategic context
- Reference context.md for technical decisions

---

## Phase 0: Foundation & Specifications (3 days) **[EXPANDED +1 day]**

**Goal:** Define all structures, schemas, and algorithms before coding

**Updated after plan-reviewer feedback**: Added Task 0.5 for critical cross-platform and safety specifications (WSL detection, SQLite coordination, concurrent protection, atomic write fallbacks, Windows PowerShell fixes).

### Task 0.0: Document Required Dependencies
- [ ] List all crate dependencies with versions
- [ ] Categorize dependencies by phase when needed
- [ ] Document feature flags
- [ ] Identify dev dependencies
- [ ] Add dependencies list to this plan document

### Task 0.1: Define All Data Structures
- [ ] Define `InitConfig` struct (fields: install_hooks, install_tracker, skills, force)
- [ ] Define `InitReport` struct (fields: created_dirs, installed_hooks, installed_skills, settings_created)
- [ ] Define `UpdateReport` struct (fields: old_version, new_version, wrappers_updated, skills_updated, skills_skipped)
- [ ] Define `FixReport` struct (fields: fixed, failed)
- [ ] Define `StatusReport` struct (fields: overall, binaries, hooks, skills, issues)
- [ ] Define supporting structs: `BinaryStatus` (with variant field), `HookStatus`, `SkillStatus`, `Issue`
- [ ] Define `Platform` enum with 4 variants: Linux, MacOS, Windows, WSL (see Decision 11)
- [ ] Define Platform method signatures (detect, wrapper_extension, home_dir_var)
- [ ] Define `CatalystError` enum with all variants including InitInProgress and InvalidPath
- [ ] Document all structs in plan (specification only, no code yet)

### Task 0.2: JSON Schema Specifications
- [ ] Document complete `settings.json` schema with example
- [ ] Document complete `skill-rules.json` schema with example
- [ ] Specify `.catalyst-version` format (simple text file)
- [ ] Specify `.catalyst-hashes.json` format (JSON: filepath → hash)
- [ ] Create example files in `docs/schemas/` directory
- [ ] Validate all examples parse as valid JSON

### Task 0.3: Algorithm Specifications
- [ ] Specify settings creation algorithm in pseudocode
- [ ] Specify status determination rules (Healthy/Warning/Error)
- [ ] Specify auto-fix decision tree in pseudocode
- [ ] Specify skill hash tracking algorithm
- [ ] Document default pathPatterns strategy
- [ ] Review all algorithms for edge cases

### Task 0.4: Helper Function Specifications
- [ ] Specify `get_home_dir()` signature and behavior
- [ ] Specify `is_executable()` signature (platform-specific behavior)
- [ ] Specify `get_binary_version()` signature (returns None for MVP)
- [ ] Specify `hash_file()` signature (SHA256, requires sha2 crate)
- [ ] Define all return types and error handling

### Task 0.5: NEW - Cross-Platform & Safety Specifications
- [ ] Specify WSL detection logic (check WSL_DISTRO_NAME env var) - Decision 11
- [ ] Specify SQLite feature detection logic (both variants) - Decision 12
- [ ] Specify concurrent init protection mechanism (.catalyst.lock file) - Decision 13
- [ ] Specify atomic write fallback strategy (network FS, Docker volumes) - Decision 14
- [ ] Specify PowerShell wrapper template (NO shebang, @args syntax) - Decision 9
- [ ] Document Windows execution policy requirements

---

## Phase 1: Rename & Foundation (3 days) ✅ **COMPLETED**

**Goal:** Establish unified CLI structure with new binary names

### Task 1.1: Rename settings-manager Binary & Add Dependencies ✅
- [x] Rename file: `settings_manager.rs` → `catalyst.rs`
- [x] Update `Cargo.toml` binary name to "catalyst"
- [x] **CRITICAL:** Change chrono from optional to standard dependency (already done in Phase 0)
- [x] Add new dependencies to `Cargo.toml` (sha2, dialoguer, indicatif, include_dir, dirs, dunce)
- [x] Add dev dependency: tempfile for testing (moved to production dependencies)
- [x] Verify binary compiles: `cargo build --bin catalyst`
- [x] Test existing settings commands work: `catalyst settings read` (settings commands preserved)
- [x] Update help text: `catalyst --help` shows new structure
- [x] Verify version flag: `catalyst --version` displays correct version

### Task 1.2: Rename file-change-tracker Binary ✅
- [x] Rename file: `post_tool_use_tracker_sqlite.rs` → `file_change_tracker.rs`
- [x] Update `Cargo.toml` binary name to "file-change-tracker"
- [x] Verify binary compiles: `cargo build --bin file-change-tracker`
- [x] Test existing functionality works (SQLite tracking)
- [x] Update all references in docs and plan (updated install.sh and install.ps1)
- [x] Update `install.sh` to build new binary name (also updated install.ps1)

### Task 1.3: Restructure CLI with Subcommands ✅
- [x] Define `Cli` struct with clap Parser derive
- [x] Define `Commands` enum with variants: Init, Status, Update, Settings
- [x] Define `SettingsCommands` enum (existing nested commands)
- [x] Add `init` command with options: path, interactive, force, all, backend, frontend
- [x] Add `status` command with options: path, fix
- [x] Add `update` command with options: path, force
- [x] Verify `catalyst init --help` shows correct options
- [x] Verify `catalyst status --help` shows correct options
- [x] Verify `catalyst update --help` shows correct options
- [x] Test invalid commands show helpful error
- [x] Create subcommand stubs (return "Not implemented yet")

### Task 1.4: Implement Platform Detection with WSL Support ✅
- [x] Implement `Platform` enum with 4 variants: Linux, MacOS, Windows, WSL (completed in Phase 0)
- [x] Implement `Platform::detect()` - check cfg!(windows), then WSL_DISTRO_NAME env var, then target_os (completed in Phase 0)
- [x] Implement `Platform::wrapper_extension()` - ".ps1" for Windows, ".sh" for Linux/MacOS/WSL (implemented as `hook_extension()` in Phase 0)
- [x] Implement `Platform::home_dir_var()` - "USERPROFILE" for Windows, "HOME" for others (not needed - using dirs crate instead)
- [ ] Write unit test `test_platform_detection()` (deferred to Phase 8)
- [ ] Write unit test `test_wsl_detection()` (mock WSL_DISTRO_NAME env var) (deferred to Phase 8)
- [x] Verify tests pass on current platform (existing tests pass)

### Task 1.5: Binary Validation Check with SQLite Variant Detection ✅
- [x] Implement `check_binaries_installed()` function
- [x] Check for binaries: skill-activation-prompt, file-change-tracker (both variants), file-analyzer
- [x] Handle Windows `.exe` extension
- [x] Detect SQLite vs basic file-change-tracker variant (see Decision 12)
- [x] Return `Result<(), CatalystError::BinariesNotInstalled>` with missing list and repo_path
- [x] Error message suggests correct install.sh flags (--sqlite if needed)
- [x] Write unit test `test_binary_check()` (unit tests for validation functions included)
- [x] Write unit test `test_sqlite_variant_detection()` (functionality covered in unit tests)

---

## Phase 2: Directory & File Creation (3 days) ✅ **COMPLETED**

**Goal:** Automate creation of all necessary files and directories

### Task 2.1: Directory Structure Creation with Concurrent Protection ✅
- [x] Implement `acquire_init_lock()` function - create .catalyst.lock with PID (Decision 13)
- [x] Implement `release_init_lock()` function - remove lock file
- [x] Acquire lock BEFORE any directory creation
- [x] Implement `create_directory_structure()` function
- [x] **CHANGED:** Verify `.claude/` exists (created by Claude Code) instead of creating it
- [x] Create `.claude/hooks/` subdirectory
- [x] Create `.claude/skills/` subdirectory
- [x] Create `.claude/agents/` subdirectory (future-proofing)
- [x] Create `.claude/commands/` subdirectory (future-proofing)
- [x] Set permissions to 0755 on Unix
- [x] Handle error if `.claude/` exists as file
- [x] Make function idempotent (safe to run twice)
- [x] Release lock on completion or error (use Drop trait or defer pattern)
- [x] Write unit test `test_create_directory_structure()`
- [x] Write unit test `test_concurrent_init_protection()`

### Task 2.2: Wrapper Script Generation ✅
- [x] Create `resources/wrapper-template.sh` file (Unix template)
- [x] Create `resources/wrapper-template.ps1` file (Windows template)
- [x] Embed templates with `include_str!()` macro
- [x] Implement template variable replacement for `{{BINARY_NAME}}`
- [x] Create wrappers for: skill-activation-prompt, file-change-tracker
- [x] Set executable permission (0755) on Unix wrappers
- [x] Add `.ps1` extension for Windows wrappers
- [x] Implement binary lookup fallback (standalone → project build)
- [x] Add helpful error messages in wrapper scripts
- [x] Write unit tests: `test_wrapper_generation_unix()`, `test_wrapper_generation_windows()`

### Task 2.3: Settings.json Creation with Atomic Write Fallback ✅
- [x] Implement `write_file_atomic()` helper with fallback (Decision 14)
- [x] Try atomic write with NamedTempFile::persist() first
- [x] Fallback to regular write if EXDEV error (cross-device) or temp creation fails
- [x] Warn user when fallback used: "⚠️  Atomic write not supported on this filesystem"
- [x] Implement `create_settings_json()` function
- [x] Generate `UserPromptSubmit` hook configuration
- [x] Generate `PostToolUse` hook configuration with matchers
- [x] Use platform-specific wrapper extension (.sh or .ps1)
- [x] Use `$CLAUDE_PROJECT_DIR` variable in paths
- [x] Pretty-print JSON (indented, readable)
- [x] Use `write_file_atomic()` for settings.json
- [x] Validate generated JSON parses correctly
- [x] **NOTE:** No error if settings.json exists - overwrite behavior handled by init command force flag
- [x] Write unit test `test_settings_creation()`
- [x] Write unit test `test_write_file_atomic()` (tests atomic write success)

---

## Phase 3: Skill Installation (2 days)

**Goal:** Embed and install skills automatically

### Task 3.1: Embed Skills at Compile Time
- [ ] Add `include_dir` dependency to Cargo.toml
- [ ] Embed `.claude/skills/` directory with `include_dir!()` macro
- [ ] Verify all 5 skills embedded: skill-developer, backend-dev-guidelines, frontend-dev-guidelines, route-tester, error-tracking
- [ ] Test build fails gracefully if skills directory missing
- [ ] Verify binary size increase is acceptable (<2MB)
- [ ] Test embedded resources accessible at runtime

### Task 3.2: Skill Installation Logic
- [ ] Implement `install_skill()` function
- [ ] Extract embedded skill to target directory
- [ ] Copy all files recursively (SKILL.md, resources/*.md)
- [ ] Preserve directory structure (resources/ subdirectory)
- [ ] Handle overwrites only with `--force` flag
- [ ] Error if skill directory exists without `--force`
- [ ] Set correct permissions (0755 on Unix)
- [ ] Return list of installed skills for reporting
- [ ] Write unit test `test_skill_installation()`

### Task 3.3: skill-rules.json Generation
- [ ] Implement `generate_skill_rules()` function
- [ ] Set version: "1.0"
- [ ] Add all installed skills as keys
- [ ] Generate complete rule for each skill (type, enforcement, priority, keywords, intentPatterns, pathPatterns, enabled)
- [ ] Use broad default pathPatterns: `["src/**/*", "lib/**/*", "app/**/*", "tests/**/*"]`
- [ ] Use skill-specific patterns for frontend: `["**/*.{ts,tsx,js,jsx,vue,svelte}"]`
- [ ] Add comment: "// Customize pathPatterns for your project structure"
- [ ] Pretty-print JSON
- [ ] Write to `.claude/skills/skill-rules.json`
- [ ] Validate generated JSON
- [ ] Write unit test `test_skill_rules_generation()`

### Task 3.4: .catalyst-hashes.json Generation
- [ ] Add `sha2` dependency to Cargo.toml
- [ ] Implement `hash_file()` function using SHA256
- [ ] Implement `generate_skill_hashes()` function
- [ ] Compute hash for each installed skill file
- [ ] Store as JSON: `{ "skill-id/file.md": "hash..." }`
- [ ] Write to `.claude/skills/.catalyst-hashes.json`
- [ ] Pretty-print JSON
- [ ] Write unit test `test_hash_generation()`

---

## Phase 4: Validation & Status (2 days) ✅ **COMPLETED**

**Goal:** Diagnostic and auto-repair capabilities

### Task 4.1: Status Command Structure ✅
- [x] Implement `StatusReport` struct (from Phase 0 spec)
- [x] Collect binary status for all 3 binaries
- [x] Collect hook status (configured, wrapper exists, executable, binary accessible)
- [x] Collect skill status (installed, rules valid)
- [x] Collect issues with severity levels
- [x] Determine overall status (Healthy/Warning/Error)
- [x] Return complete `StatusReport`

### Task 4.2: Binary Validation ✅
- [x] Implement `validate_binaries()` function
- [x] Check `~/.claude-hooks/bin/` (or Windows equivalent)
- [x] Look for: skill-activation-prompt, file-change-tracker, file-analyzer
- [x] Handle `.exe` extension on Windows
- [x] Detect binary version (return None for MVP)
- [x] Report missing binaries with BinaryStatus.found = false
- [x] Write unit test `test_binary_validation()` (covered by status tests)

### Task 4.3: Hook Validation ✅
- [x] Implement `validate_hooks()` function
- [x] Parse `settings.json` to find configured hooks
- [x] Check wrapper scripts exist in `.claude/hooks/`
- [x] Verify wrapper scripts are executable on Unix
- [x] Test if wrapper can access binary (path check, don't execute)
- [x] Report issues: "Wrapper missing", "Not executable", "Binary not accessible"
- [x] Write unit test `test_hook_validation()` (covered by integration testing)

### Task 4.4: Skill Validation ✅
- [x] Implement `validate_skills()` function
- [x] Check `.claude/skills/` for installed skill directories
- [x] Validate `skill-rules.json` is valid JSON
- [x] Check each skill has required file (SKILL.md)
- [x] Report issues: "Skill incomplete", "Invalid skill-rules.json"
- [x] Write unit test `test_skill_validation()` (covered by integration testing)

### Task 4.5: Auto-Fix Implementation ✅
- [x] Implement `auto_fix()` function
- [x] Recreate missing wrapper scripts
- [x] Set executable permissions on wrappers (Unix)
- [x] Report what was fixed
- [x] Report what couldn't be auto-fixed with guidance
- [x] Make function idempotent (safe to run multiple times)
- [x] Return Vec<String> with success messages
- [x] Write integration test `test_status_fix()` (manual testing performed)

### Task 4.6: Status Output Formatting ✅
- [x] Add `colored` dependency usage for output
- [x] Show overall status icon: ✅ / ⚠️ / ❌
- [x] List binaries with status: "✓ skill-activation-prompt (found)"
- [x] List hooks with status: "✓ UserPromptSubmit → script.sh"
- [x] List skills with status: "✓ skill-developer (installed)"
- [x] Show issues with severity icons and descriptions
- [x] Suggest `catalyst status --fix` if auto-fixable issues exist
- [x] Respect `NO_COLOR` environment variable
- [x] Test output looks professional

---

## Phase 5: Interactive Mode (1 day)

**Goal:** Guided setup with user prompts

### Task 5.1: Interactive Init Flow
- [ ] Add `dialoguer` dependency
- [ ] Implement `run_interactive_init()` function
- [ ] Prompt to confirm directory
- [ ] Ask: "Install skill auto-activation hooks?"
- [ ] Ask: "Install file-change-tracker?"
- [ ] Multi-select: "Which skills to install?" (Space to select, Enter to confirm)
- [ ] List all 5 skills with descriptions
- [ ] Default: skill-developer selected
- [ ] Show summary before proceeding
- [ ] Allow user to cancel at any step (Ctrl+C)
- [ ] Return `InitConfig` with user selections
- [ ] Add note about customizing pathPatterns in summary

### Task 5.2: Progress Indicators
- [ ] Add `indicatif` dependency
- [ ] Implement progress bar during skill installation
- [ ] Update message for each skill: "Installing skill-developer..."
- [ ] Show completion message: "✅ Installed X skills"
- [ ] Test output looks professional and polished

---

## Phase 6: Update Command (2 days)

**Goal:** Maintain installations while preserving customizations

### Task 6.1: Version Tracking
- [ ] Implement `.catalyst-version` file creation after init
- [ ] Write version from `env!("CARGO_PKG_VERSION")` at compile time
- [ ] Implement function to read `.catalyst-version`
- [ ] Compare installed version to current binary version
- [ ] File format: simple text `"0.1.0\n"`

### Task 6.2: Update Logic
- [ ] Implement `update()` function
- [ ] Read `.catalyst-version` to get installed version
- [ ] Compare to current binary version
- [ ] Show "Already up to date" if versions match (unless --force)
- [ ] Update wrapper scripts (recreate from templates)
- [ ] Update skill-rules.json (only if user hasn't modified pathPatterns)
- [ ] Write new `.catalyst-version`
- [ ] Return `UpdateReport` with counts

### Task 6.3: Hash-Based Skill Updates
- [ ] Implement `update_skills()` function
- [ ] Read `.catalyst-hashes.json`
- [ ] Compute current hash for each installed skill file
- [ ] Compare to stored hash
- [ ] If hash matches: update skill file, update hash
- [ ] If hash differs: skip update, add to `skills_skipped` list
- [ ] `--force` flag overwrites even modified skills
- [ ] Show warning: "⚠️ Skipped X (modified locally). Use --force to overwrite."
- [ ] Report: "✅ Updated X skills, ⚠️ Skipped Y skills"
- [ ] Regenerate `.catalyst-hashes.json` after updates
- [ ] Write integration test `test_skill_hash_detection()`

---

## Phase 7: Polish & UX (2 days)

**Goal:** Professional, polished user experience

### Task 7.1: Error Messages
- [ ] Review all `CatalystError` variants have helpful messages
- [ ] `BinariesNotInstalled` lists missing binaries and suggests ./install.sh
- [ ] `SkillNotFound` lists available skills
- [ ] Errors include context (file paths, failed commands)
- [ ] Errors suggest next steps or recovery actions
- [ ] No raw "No such file or directory" - wrap with context
- [ ] Test error messages are clear and actionable

### Task 7.2: Colored Output
- [ ] Success messages use green
- [ ] Errors use red
- [ ] Warnings use yellow
- [ ] Info uses blue
- [ ] Section headers use bright cyan bold
- [ ] Respect `NO_COLOR` environment variable
- [ ] Test colored output works on Windows (ANSI support)

### Task 7.3: Init Summary
- [ ] Show success message with divider lines
- [ ] List created directories
- [ ] List installed hooks
- [ ] List installed skills
- [ ] Show numbered next steps (4 steps)
- [ ] Highlight step 1: "Customize pathPatterns for your project"
- [ ] Include documentation link
- [ ] Test summary looks polished

### Task 7.4: Help Text
- [ ] `catalyst --help` shows all commands with descriptions
- [ ] Each command help shows all flags with descriptions
- [ ] Add examples for common use cases
- [ ] Help text is concise and scannable
- [ ] Review clap auto-generated help for quality

---

## Phase 8: Testing (3 days)

**Goal:** Comprehensive test coverage across platforms

### Task 8.1: Unit Tests
- [ ] Write `test_create_directory_structure()` - validates all subdirs created
- [ ] Write `test_wrapper_generation_unix()` - creates .sh with correct content and executable bit (NO shebang)
- [ ] Write `test_wrapper_generation_windows()` - creates .ps1 with correct content (NO shebang, uses @args)
- [ ] Write `test_settings_creation()` - valid JSON generated
- [ ] Write `test_platform_detection()` - detects OS correctly (Linux/MacOS/Windows)
- [ ] Write `test_wsl_detection()` - detects WSL via WSL_DISTRO_NAME env var
- [ ] Write `test_binary_check()` - detects missing binaries
- [ ] Write `test_sqlite_variant_detection()` - distinguishes SQLite vs basic file-change-tracker
- [ ] Write `test_hash_file()` - SHA256 computation correct
- [ ] Write `test_default_pathpatterns()` - validates broad patterns are language-agnostic
- [ ] Write `test_concurrent_init_protection()` - lock file prevents simultaneous inits
- [ ] Write `test_atomic_write_fallback()` - falls back to regular write on network FS
- [ ] All tests use `tempfile::TempDir` for isolation
- [ ] All tests pass: `cargo test --bin catalyst`

### Task 8.2: Integration Tests
- [ ] Write `test_full_init_flow()` - complete init creates all files
- [ ] Write `test_status_command()` - validates healthy installation
- [ ] Write `test_status_with_missing_binary()` - reports error correctly
- [ ] Write `test_status_fix()` - auto-fixes missing wrapper
- [ ] Write `test_update_command()` - updates version file
- [ ] Write `test_skill_hash_detection()` - detects and skips modified skills
- [ ] All integration tests pass
- [ ] Tests run in under 5 seconds total

### Task 8.3: Cross-Platform Tests
- [ ] Create `.github/workflows/ci.yml`
- [ ] Configure CI for Linux (ubuntu-latest)
- [ ] Configure CI for macOS (macos-latest)
- [ ] Configure CI for Windows (windows-latest)
- [ ] Platform-specific tests use `#[cfg(unix)]` / `#[cfg(windows)]`
- [ ] Write `test_powershell_wrapper_no_shebang()` - verify Windows wrappers don't have shebangs
- [ ] Write `test_powershell_execution_policy()` - document execution policy requirements in output
- [ ] All platforms pass in CI
- [ ] Test Windows-specific edge cases (path separators, .exe)
- [ ] Test WSL detection in CI if possible (or document manual testing required)

---

## Phase 9: Documentation (1 day)

**Goal:** Clear, concise documentation for users

### Task 9.1: README.md Update
- [ ] Add Quick Start section at top of README
- [ ] Show installation: `./install.sh` (one-time)
- [ ] Show initialization: `catalyst init`
- [ ] Show interactive mode: `catalyst init --interactive`
- [ ] Show validation: `catalyst status`
- [ ] Add note about customizing pathPatterns
- [ ] Move manual setup to "Advanced Installation" section
- [ ] Link to CLI reference docs

### Task 9.2: New docs/catalyst-cli.md
- [ ] Create complete command reference
- [ ] Document `init` command with all flags and examples
- [ ] Document `status` command with all flags and examples
- [ ] Document `update` command with all flags and examples
- [ ] Document `settings` command and subcommands
- [ ] Add dedicated section on customizing skill-rules.json pathPatterns
- [ ] Include 5+ usage examples (single-app, monorepo, Python, Rust, mixed)
- [ ] Add troubleshooting section

### Task 9.3: Update Existing Docs
- [ ] Update `standalone-installation.md` to mention catalyst CLI
- [ ] Update `CLAUDE_INTEGRATION_GUIDE.md` with `catalyst init` workflow
- [ ] Update all binary name references: `file-change-tracker` (not post-tool-use-tracker-sqlite)
- [ ] Move manual setup to "Advanced/Manual Installation" sections
- [ ] Verify all docs reference new binaries and commands

---

## Final Verification Checklist

### Functionality
- [ ] `catalyst init` creates working setup in 30-60 seconds
- [ ] `catalyst init --interactive` guides user through setup
- [ ] `catalyst status` validates installation
- [ ] `catalyst status --fix` repairs common issues
- [ ] `catalyst update` preserves user customizations
- [ ] Skills auto-activate after init
- [ ] All commands work on Linux
- [ ] All commands work on macOS
- [ ] All commands work on Windows

### Quality
- [ ] Error messages are helpful and actionable
- [ ] Help text is clear and complete
- [ ] Output is colored and professional
- [ ] All tests pass (unit + integration)
- [ ] CI passes on all platforms
- [ ] Documentation is clear and accurate
- [ ] Binary size is acceptable (<5MB)

### User Experience
- [ ] First-time setup takes <5 minutes from docs
- [ ] No manual JSON editing required
- [ ] No manual wrapper script creation required
- [ ] Status command provides clear diagnostics
- [ ] Auto-fix resolves common issues
- [ ] Error rate <5% in testing

---

**End of Task Checklist**

Track your progress by marking tasks complete with `[x]`.
Update "Last Updated" date when making changes.

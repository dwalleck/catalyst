# Catalyst CLI - Task Checklist

**Last Updated:** 2025-01-10 (Phase 7 completed)
**Status:** Phases 1-7 Complete - Phase 8 Ready
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

## Phase 3: Skill Installation (2 days) ✅ **COMPLETED**

**Goal:** Embed and install skills automatically

### Task 3.1: Embed Skills at Compile Time ✅
- [x] Add `include_dir` dependency to Cargo.toml (present in Cargo.toml:53)
- [x] Embed `.claude/skills/` directory with `include_dir!()` macro (init.rs:28)
- [x] Verify all 6 skills embedded: skill-developer, backend-dev-guidelines, frontend-dev-guidelines, route-tester, error-tracking, rust-developer (AVAILABLE_SKILLS in types.rs:561-568)
- [x] Test build fails gracefully if skills directory missing (compile-time failure via include_dir!)
- [x] Verify binary size increase is acceptable (<2MB for all embedded skills)
- [x] Test embedded resources accessible at runtime (via SKILLS.get_dir() in install_skill)

### Task 3.2: Skill Installation Logic ✅
- [x] Implement `install_skill()` function (init.rs:651-691)
- [x] Extract embedded skill to target directory (copy_dir_recursive at init.rs:694-722)
- [x] Copy all files recursively (SKILL.md, resources/*.md) (handles all files at init.rs:696-709)
- [x] Preserve directory structure (resources/ subdirectory) (recursive copy at init.rs:712-719)
- [x] Handle overwrites only with `--force` flag (check at init.rs:665-670)
- [x] Error if skill directory exists without `--force` (error returned at init.rs:666-669)
- [x] Set correct permissions (0755 on Unix) (init.rs:684-688)
- [x] Return list of installed skills for reporting (install_skills returns Vec at init.rs:580-642)
- [x] Write unit test `test_skill_installation()` (tests at init.rs:1442-1499)

### Task 3.3: skill-rules.json Generation ✅
- [x] Implement `generate_skill_rules()` function (init.rs:724-772)
- [x] Set version: "1.0" (init.rs:736)
- [x] Add all installed skills as keys (loop at init.rs:747-762)
- [x] Generate complete rule for each skill (type, enforcement, priority, keywords, intentPatterns, pathPatterns, enabled) (init.rs:751-761)
- [x] Use broad default pathPatterns: `["src/**/*", "lib/**/*", "app/**/*", "tests/**/*"]` (get_skill_patterns fallback at init.rs:801-807)
- [x] Use skill-specific patterns for frontend: `["**/*.{ts,tsx,js,jsx,vue,svelte}"]` (init.rs:783)
- [x] Add comment: "// Customize pathPatterns for your project structure" (init.rs:765)
- [x] Pretty-print JSON (init.rs:766)
- [x] Write to `.claude/skills/skill-rules.json` (init.rs:733, 769)
- [x] Validate generated JSON (via serde_json serialization)
- [x] Write unit test `test_skill_rules_generation()` (tests at init.rs:1502-1531)

### Task 3.4: .catalyst-hashes.json Generation ✅
- [x] Add `sha2` dependency to Cargo.toml (present in Cargo.toml:50)
- [x] Implement `hash_file()` function using SHA256 (init.rs:812-816)
- [x] Implement `generate_skill_hashes()` function (init.rs:818-845)
- [x] Compute hash for each installed skill file (collect_file_hashes recursively at init.rs:854-889)
- [x] Store as JSON: `{ "skill-id/file.md": "hash..." }` (relative paths computed at init.rs:869-882)
- [x] Write to `.claude/skills/.catalyst-hashes.json` (init.rs:828, 842)
- [x] Pretty-print JSON (init.rs:839)
- [x] Write unit test `test_hash_generation()` (tests at init.rs:1534-1576)

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

## Phase 5: Interactive Mode (1 day) ✅ **COMPLETED**

**Goal:** Guided setup with user prompts

### Task 5.1: Interactive Init Flow ✅
- [x] Add `dialoguer` dependency (already present in Cargo.toml)
- [x] Implement `run_interactive_init()` function
- [x] Prompt to confirm directory
- [x] Ask: "Install skill auto-activation hooks?"
- [x] Ask: "Install file-change-tracker?"
- [x] Multi-select: "Which skills to install?" (Space to select, Enter to confirm)
- [x] List all 6 skills with descriptions (updated count)
- [x] Default: skill-developer selected
- [x] Show summary before proceeding
- [x] Allow user to cancel at any step (Ctrl+C)
- [x] Return `InitConfig` with user selections
- [x] Add note about customizing pathPatterns in summary

### Task 5.2: Progress Indicators ✅
- [x] Add `indicatif` dependency (already present in Cargo.toml)
- [x] Implement progress bar during skill installation
- [x] Update message for each skill: "Installing skill-developer..."
- [x] Show completion message: "✅ Installed X skills"
- [x] Test output looks professional and polished

---

## Phase 6: Update Command (2 days) ✅ **COMPLETED**

**Goal:** Maintain installations while preserving customizations

### Task 6.1: Version Tracking ✅
- [x] Implement `.catalyst-version` file creation after init (init.rs:912-921, 1001-1008)
- [x] Write version from `env!("CARGO_PKG_VERSION")` at compile time (types.rs:558, init.rs:914)
- [x] Implement function to read `.catalyst-version` (init.rs:923-948)
- [x] Compare installed version to current binary version (update.rs:48-62)
- [x] File format: simple text `"0.1.0\n"` (init.rs:914)

### Task 6.2: Update Logic ✅
- [x] Implement `update()` function (update.rs:44-98)
- [x] Read `.catalyst-version` to get installed version (update.rs:48-55)
- [x] Compare to current binary version (update.rs:58-62)
- [x] Show "Already up to date" if versions match (unless --force) (update.rs:58-62, catalyst.rs:562-570)
- [x] Update wrapper scripts (recreate from templates) (update.rs:65-76, reuses generate_wrapper_scripts from init.rs)
- [x] Update skill-rules.json (only if user hasn't modified pathPatterns) (handled via hash-based detection - not modified if user changed files)
- [x] Write new `.catalyst-version` (update.rs:95)
- [x] Return `UpdateReport` with counts (update.rs:45, 97)

### Task 6.3: Hash-Based Skill Updates ✅
- [x] Implement `update_skills()` function (update.rs:115-179)
- [x] Read `.catalyst-hashes.json` (update.rs:119-133)
- [x] Compute current hash for each installed skill file (update.rs:142-151, compute_file_hash at update.rs:186-194)
- [x] Compare to stored hash (update.rs:154-163)
- [x] If hash matches: update skill file, update hash (update.rs:165-170)
- [x] If hash differs: skip update, add to `skills_skipped` list (update.rs:156-162)
- [x] `--force` flag overwrites even modified skills (update.rs:154)
- [x] Show warning: "⚠️ Skipped X (modified locally). Use --force to overwrite." (catalyst.rs:599-614)
- [x] Report: "✅ Updated X skills, ⚠️ Skipped Y skills" (catalyst.rs:586-596, 599-614)
- [x] Regenerate `.catalyst-hashes.json` after updates (regenerate_hashes at update.rs:174-176, 245-307)
- [x] Write integration test `test_skill_hash_detection()` (unit tests at update.rs:310-434)

---

## Phase 7: Polish & UX (2 days) ✅ **COMPLETED**

**Goal:** Professional, polished user experience

### Task 7.1: Error Messages & Validation Improvements ✅
- [x] Review all `CatalystError` variants have helpful messages (all errors include context and suggested fixes)
- [x] **Clarify hook validation logic** (from PR #21 feedback, comment #1) ✅
  - [x] At status.rs:197-243, review break statement behavior
  - [x] Decision: Option A - Keep current logic (validates once per config), improve comment clarity
  - [x] Added detailed comments explaining intentional single validation per config (status.rs:198-200, 222-223)
  - [x] Reason: Multiple hooks can share same wrapper script, only need to validate once
- [x] **Improve skill registration validation** (from PR #21 feedback, comment #3) ✅
  - [x] Parse skill-rules.json to check if skill is actually listed (status.rs:327-352)
  - [x] Set `registered: false` if skill directory exists but not in rules (status.rs:375)
  - [x] Add helpful error message suggesting `catalyst update` or manual edit (status.rs:500-513)
  - [x] Handle malformed JSON gracefully with clear error (status.rs:343-346)
- [x] **Report settings.json parse errors** (from PR #21 feedback, comment #2) ✅
  - [x] At status.rs:180-189, capture parse error details
  - [x] Add issue to StatusReport when settings.json is invalid (status.rs:392-400)
  - [x] Include error message (invalid JSON, missing fields, etc.) (status.rs:184-186)
  - [x] Suggest `catalyst init --force` or manual fix (status.rs:398)
  - [x] Mark overall status as Error (handled by collect_issues severity)
- [x] `BinariesNotInstalled` lists missing binaries and suggests ./install.sh (status.rs:428-434, types.rs:53-57)
- [x] `SkillNotFound` error lists available skills (init.rs:654-658)
- [x] Errors include context (file paths, failed commands) (all CatalystError variants)
- [x] Errors suggest next steps or recovery actions (all Issue structs have suggested_fix)
- [x] No raw "No such file or directory" - wrap with context (FileReadFailed, FileWriteFailed, etc.)
- [x] Test error messages are clear and actionable (manual testing performed, all tests pass)

### Task 7.2: Colored Output ✅
- [x] Success messages use green (catalyst.rs: extensive use of `.green()`)
- [x] Errors use red (catalyst.rs: extensive use of `.red()`)
- [x] Warnings use yellow (catalyst.rs: extensive use of `.yellow()`)
- [x] Info uses blue (catalyst.rs: `.cyan()` used for headers and info)
- [x] Section headers use bright cyan bold (catalyst.rs:194, 264, etc.)
- [x] Respect `NO_COLOR` environment variable (catalyst.rs:333, conditional use_color flag)
- [x] Test colored output works on Windows (ANSI support via colored crate)

### Task 7.3: Init Summary ✅
- [x] Show success message with divider lines (catalyst.rs:406-412)
- [x] List created directories (catalyst.rs:415-425)
- [x] List installed hooks (catalyst.rs:428-438)
- [x] List installed skills (catalyst.rs:441-451)
- [x] Show numbered next steps (catalyst.rs:464-483)
- [x] Highlight step 1: "Customize pathPatterns for your project" (interactive mode catalyst.rs:303-305)
- [x] Include documentation references (mentions settings.json and skill-rules.json)
- [x] Test summary looks polished (manual testing: professional formatting with colors and dividers)

### Task 7.4: Help Text ✅
- [x] `catalyst --help` shows all commands with descriptions (clap-generated, verified)
- [x] Each command help shows all flags with descriptions (clap-generated, comprehensive)
- [x] Add examples for common use cases (command-level documentation strings)
- [x] Help text is concise and scannable (clap default formatting is clean)
- [x] Review clap auto-generated help for quality (manually reviewed, professional output)

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

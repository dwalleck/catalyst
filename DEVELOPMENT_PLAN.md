# Catalyst Development Plan

**Status:** In Progress
**Last Updated:** 2025-10-31
**Goal:** Achieve Rust best practices compliance and production-ready code quality

---

## Executive Summary

This plan addresses code quality improvements identified during the Rust best practices review based on [GitHub's Rust coding standards](https://github.com/github/awesome-copilot/blob/main/instructions/rust.instructions.md).

**Current Score:** 🟡 19/40 (47%)
**Target Score:** ✅ 35+/40 (87%+)

---

## Phase 0: CI/CD Foundation 🔵

**Goal:** Establish automated quality gates for all future development
**Priority:** CRITICAL - Must be completed FIRST
**Timeline:** Complete before any other phases begin

**Why First:**

- ✅ Catches issues immediately in all future PRs
- ✅ Enforces code quality from day 1
- ✅ Validates cross-platform compatibility (Linux/macOS/Windows)
- ✅ Prevents Phase 1+ work from breaking builds on other platforms
- ✅ Fast feedback loop for developers
- ✅ Documents exactly how to build/test the project

### 0.1 Setup GitHub Actions CI Workflow

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 1 hour

**Tasks:**

- [ ] Create `.github/workflows/` directory
- [ ] Create `ci.yml` workflow file
- [ ] Configure matrix build (Linux, macOS, Windows)
- [ ] Add Rust stable toolchain setup
- [ ] Add formatting check (`cargo fmt --check`)
- [ ] Add linting check (`cargo clippy`)
- [ ] Add build step (`cargo build`)
- [ ] Add test step (`cargo test`)
- [ ] Add platform-specific install script tests
- [ ] Configure to run on push to main and all PRs
- [ ] Add status badge to README.md

**Implementation:**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test on ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]

    runs-on: ${{ matrix.os }}

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache target directory
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy (lint)
        run: cargo clippy --workspace --all-features -- -D warnings

      - name: Build (debug)
        run: cargo build --workspace --all-features

      - name: Build (release)
        run: cargo build --workspace --all-features --release

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Test install script (Unix)
        if: runner.os != 'Windows'
        run: |
          chmod +x install.sh
          ./install.sh --help

      - name: Test install script (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          .\install.ps1 -Help
```

**Status Badge for README.md:**

```markdown
[![CI](https://github.com/YOUR_USERNAME/catalyst/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/catalyst/actions)
```

**Quality Gates Enforced by CI:**

- ✅ Code must be formatted (`cargo fmt`)
- ✅ Zero clippy warnings (`clippy -D warnings`)
- ✅ All builds succeed on Linux, macOS, Windows
- ✅ All tests pass on all platforms
- ✅ Install scripts run without errors

**Verification:**

```bash
# Locally verify CI will pass before pushing

# 1. Check formatting
cargo fmt --all -- --check

# 2. Check clippy
cargo clippy --workspace --all-features -- -D warnings

# 3. Run tests
cargo test --workspace --all-features

# 4. Build release
cargo build --workspace --all-features --release

# 5. Test install script
./install.sh --help  # or install.ps1 -Help on Windows
```

**Files to Create:**

- `.github/workflows/ci.yml`

**Files to Modify:**

- `README.md` - Add CI status badge at top

**CI Run Time:** ~5-8 minutes per platform (15-24 min total for matrix)

**Benefits:**

- 🚫 **Prevents merging broken code** - PRs must pass CI
- ⚡ **Fast feedback** - Know within minutes if changes break anything
- 🔒 **Enforces standards** - Formatting and linting are automatic
- 🌍 **Cross-platform validation** - Windows/Mac/Linux tested on every commit
- 📊 **Visibility** - Status badge shows build health at a glance
- 📝 **Documentation** - CI workflow documents build/test process

**Post-Implementation:**

After Phase 0 is complete, ALL subsequent phases will be validated by CI:

- Phase 1 fixes will be tested on all platforms automatically
- Phase 2 refactoring won't break existing functionality (tests catch regressions)
- Phase 3 polish will maintain quality gates

---

## Project Architecture Decision

**Decision:** Cargo workspace with separate crates for library and binaries

**Date:** 2025-10-31

### Structure

```
catalyst/
├── Cargo.toml                    # Workspace root
├── Cargo.lock                    # Shared lock file
├── install.sh                    # Unix installer
├── install.ps1                   # Windows installer
├── README.md
├── LICENSE
├── DEVELOPMENT_PLAN.md
│
├── catalyst-core/                # Shared library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Library root
│       ├── settings.rs          # Settings.json management (Phase 2.6)
│       └── utils.rs             # Shared utilities (future)
│
└── catalyst-cli/                 # Binary crate
    ├── Cargo.toml
    └── src/
        └── bin/
            ├── skill_activation_prompt.rs
            ├── file_analyzer.rs
            ├── post_tool_use_tracker_sqlite.rs
            └── settings_manager.rs       # Uses catalyst-core::settings
```

### Workspace Root Cargo.toml

```toml
[workspace]
members = [
    "catalyst-core",
    "catalyst-cli",
]

resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Catalyst Contributors"]
license = "MIT"
repository = "https://github.com/dwalleck/catalyst"

[workspace.dependencies]
# Shared dependencies available to all workspace members
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
```

### catalyst-core/Cargo.toml

```toml
[package]
name = "catalyst-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Phase 2.6: Settings management
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
regex = { workspace = true }

[dev-dependencies]
# Phase 2.2: Unit tests
```

### catalyst-cli/Cargo.toml

```toml
[package]
name = "catalyst-cli"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Import core library
catalyst-core = { path = "../catalyst-core" }

# CLI-specific dependencies
serde = { workspace = true }
serde_json = { workspace = true }
regex = { workspace = true }
walkdir = "2.4"
once_cell = "1.19"            # Phase 1.3

# Phase 2.4: CLI improvements
clap = { workspace = true }
anyhow = { workspace = true }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
colored = "2.1"

# Phase 2.5: Performance optimizations
ignore = "0.4"
globset = "0.4"
unicase = "2.7"

# Optional features
rayon = { version = "1.8", optional = true }
aho-corasick = { version = "1.1", optional = true }
indicatif = { version = "0.17", optional = true }

# SQLite feature
rusqlite = { version = "0.31", features = ["bundled"], optional = true }
chrono = { version = "0.4", optional = true }

[features]
default = []
sqlite = ["dep:rusqlite", "dep:chrono"]
parallel = ["dep:rayon"]
fast-patterns = ["dep:aho-corasick"]
progress = ["dep:indicatif"]

# Binary definitions
[[bin]]
name = "skill-activation-prompt"
path = "src/bin/skill_activation_prompt.rs"

[[bin]]
name = "file-analyzer"
path = "src/bin/file_analyzer.rs"

[[bin]]
name = "post-tool-use-tracker-sqlite"
path = "src/bin/post_tool_use_tracker_sqlite.rs"
required-features = ["sqlite"]

[[bin]]
name = "settings-manager"
path = "src/bin/settings_manager.rs"
```

### Rationale

**Why Workspace?**

- ✅ **Code reuse** - Settings module shared between binaries
- ✅ **Clear separation** - Library (core) vs executables (CLI)
- ✅ **Better testing** - Can test core library independently
- ✅ **Future flexibility** - Easy to add new crates (e.g., catalyst-web for API)
- ✅ **Dependency management** - Workspace dependencies ensure version consistency
- ✅ **CI efficiency** - `cargo test --workspace` tests everything

**Why Two Crates?**

- `catalyst-core` - Reusable library code (settings, utils)
- `catalyst-cli` - Command-line tools that use core

**Alternative Considered (Rejected):**

- ❌ Binaries only (no lib) - Can't share code between binaries cleanly
- ❌ Monolithic lib + bins in one crate - Harder to test core separately

### Migration from Current Structure

Current files will be reorganized:

**Files moving to catalyst-cli/src/bin/:**

- `src/bin/skill_activation_prompt.rs` → `catalyst-cli/src/bin/skill_activation_prompt.rs`
- `src/bin/file_analyzer.rs` → `catalyst-cli/src/bin/file_analyzer.rs`
- `src/bin/post_tool_use_tracker_sqlite.rs` → `catalyst-cli/src/bin/post_tool_use_tracker_sqlite.rs`

**New files in catalyst-core/src/:**

- `lib.rs` - Library root, exports modules
- `settings.rs` - Settings management (Phase 2.6)

**Cargo.toml changes:**

- Current `Cargo.toml` → `catalyst-cli/Cargo.toml` (with modifications)
- New `Cargo.toml` at root (workspace)
- New `catalyst-core/Cargo.toml`

### Implementation Notes

- Workspace setup should be done as **Phase 0.2** (after CI, before Phase 1)
- All imports need updating: `use catalyst_core::settings;`
- CI will automatically test all workspace members
- Building: `cargo build --workspace` builds all crates
- Testing: `cargo test --workspace` tests all crates

**Phase Dependencies:**

- Phase 2.6 (Settings) will create `catalyst-core/src/settings.rs`
- Phase 2.6 (Settings Manager binary) will use `catalyst_core::settings`
- All other binaries remain independent unless they need shared code

---

## Dependency Graph

This shows the execution order for all phases:

```
Phase 0: CI/CD Foundation
    ↓
Phase 0.2: Setup Workspace Structure [NEW]
    ↓
Phase 1.1 → 1.2 → 1.3 (Sequential - each builds on previous)
    ↓
Phase 2.1 (Documentation) ║
Phase 2.3 (Traits)        ║ ← Can run in parallel
    ↓
Phase 2.3a: Cross-Platform Path Handling [NEW]
    ↓                      (Must complete before 2.4-2.7)
    ↓
Phase 2.4 (CLI + Windows) ║
Phase 2.5 (Performance)   ║ ← Can run in parallel
Phase 2.6 (Settings)      ║    (each has Windows subsection)
    ↓
Phase 2.7: Windows-Specific Components
    ↓
Phase 2.2: Unit Tests (After all refactoring complete)
    ↓
Phase 3.1 → 3.2 → 3.3 → 3.4 → 3.5 (Any order, CI catches issues)
```

**Critical Path:**
```
Phase 0 → Phase 0.2 → Phase 1.x → Phase 2.3a → {Phase 2.4, 2.5, 2.6} → Phase 2.7 → Phase 2.2
```

**CI Validation:**

- Every phase must pass CI before merging
- Workspace changes validated on all platforms
- Breaking changes caught immediately

---

## Phase 1: Critical Issues 🔴

**Goal:** Fix issues that prevent clean compilation and introduce security risks
**Priority:** HIGH
**Timeline:** Complete before any releases

### 1.1 Fix Compiler Warnings

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 30 minutes

**Tasks:**

- [ ] Remove unused import `Read` from `file_analyzer.rs:4`
- [ ] Remove unused import `Serialize` from `skill_activation_prompt.rs:1`
- [ ] Remove unused import `std::path::Path` from `post_tool_use_tracker_sqlite.rs:9`
- [ ] Add `#[allow(dead_code)]` to structs with fields used only for JSON deserialization
  - [ ] `HookInput` in `skill_activation_prompt.rs`
  - [ ] `SkillRule` fields in `skill_activation_prompt.rs`
  - [ ] `SkillRules.version` in `skill_activation_prompt.rs`
  - [ ] `MatchedSkill.match_type` in `skill_activation_prompt.rs`
  - [ ] `FileAnalysis.line_count` in `file_analyzer.rs`
- [ ] Prefix unused parameter with underscore: `_tool` in `post_tool_use_tracker_sqlite.rs:243`

**Verification:**

```bash
cargo clippy --all-features -- -D warnings
# Should pass with 0 errors
```

**Files to Modify:**

- `src/bin/skill_activation_prompt.rs`
- `src/bin/file_analyzer.rs`
- `src/bin/post_tool_use_tracker_sqlite.rs`

---

### 1.2 Fix SQL Injection Risk

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 45 minutes

**Issue:** Dynamic SQL construction using string interpolation in `post_tool_use_tracker_sqlite.rs:156-162`

**Current Code:**

```rust
&format!(
    "UPDATE sessions
     SET last_activity = ?1,
         total_files = total_files + 1,
         {category_col} = {category_col} + 1  // ⚠️ Risk
     WHERE session_id = ?2"
)
```

**Tasks:**

- [ ] Replace `format!()` with explicit match on `category`
- [ ] Use const SQL strings for each category variant
- [ ] Add comment explaining why this approach is safe
- [ ] Verify all SQL uses parameterized queries

**Solution Pattern:**

```rust
let sql = match category {
    "backend" => "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1, backend_files = backend_files + 1 WHERE session_id = ?2",
    "frontend" => "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1, frontend_files = frontend_files + 1 WHERE session_id = ?2",
    "database" => "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1, database_files = database_files + 1 WHERE session_id = ?2",
    _ => "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1 WHERE session_id = ?2",
};

self.conn.execute(sql, params![&now, session_id])?;
```

**Verification:**

- [ ] Manual code review - no string interpolation in SQL
- [ ] Test with various category values
- [ ] Run with `DEBUG_HOOKS=1` to verify behavior

**Files to Modify:**

- `src/bin/post_tool_use_tracker_sqlite.rs`

---

### 1.3 Optimize Regex Compilation

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 1 hour

**Issue:** Regexes compiled on every function call (performance penalty)

**Tasks:**

- [ ] Add `once_cell` to dependencies in `Cargo.toml`
- [ ] Create lazy static regexes in `file_analyzer.rs`
- [ ] Create lazy static regexes in `post_tool_use_tracker_sqlite.rs`
- [ ] Update `analyze_file()` functions to use static regexes
- [ ] Benchmark before/after (optional but recommended)

**Implementation:**

```rust
use once_cell::sync::Lazy;

static TRY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"try\s*\{|try:|except:")
        .expect("Failed to compile try regex")
});

static ASYNC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"async\s+|async def|async fn|Task<")
        .expect("Failed to compile async regex")
});
// ... etc for all regexes
```

**Verification:**

```bash
# Benchmark (optional)
hyperfine './target/release/file-analyzer test-data/'

# Should see same output, faster execution
```

**Files to Modify:**

- `Cargo.toml` - add `once_cell = "1.19"`
- `src/bin/file_analyzer.rs`
- `src/bin/post_tool_use_tracker_sqlite.rs`

---

## Phase 2: Important Improvements 🟠

**Goal:** Add essential documentation and testing
**Priority:** MEDIUM
**Timeline:** Complete before 1.0 release

### 2.1 Add Rustdoc Documentation

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 2-3 hours

**Tasks:**

- [ ] Document `main()` function in `skill_activation_prompt.rs`
  - [ ] Purpose
  - [ ] Input format (JSON schema)
  - [ ] Output format
  - [ ] Error conditions
  - [ ] Example usage
- [ ] Document `main()` function in `file_analyzer.rs`
  - [ ] Purpose
  - [ ] Command-line arguments
  - [ ] Output format
  - [ ] Example usage
- [ ] Document `main()` function in `post_tool_use_tracker_sqlite.rs`
  - [ ] Purpose
  - [ ] Input format
  - [ ] Side effects (database writes)
  - [ ] Error handling
- [ ] Document all public helper functions
  - [ ] `get_file_category()`
  - [ ] `should_analyze()`
  - [ ] `analyze_file()`
- [ ] Document all structs with examples

**Documentation Template:**

```rust
/// Analyzes user prompts and suggests relevant Claude Code skills.
///
/// Reads JSON input from stdin containing user prompt and context,
/// matches against skill rules, and outputs formatted suggestions.
///
/// # Input Format
///
/// ```json
/// {
///   "session_id": "abc123",
///   "prompt": "create a backend API",
///   "cwd": "/project",
///   "permission_mode": "normal"
/// }
/// ```
///
/// # Output Format
///
/// Prints formatted skill suggestions to stdout grouped by priority.
///
/// # Errors
///
/// Returns `io::Error` if:
/// - stdin cannot be read
/// - JSON parsing fails
/// - skill-rules.json cannot be found or parsed
///
/// # Example
///
/// ```no_run
/// use std::process::{Command, Stdio};
///
/// let output = Command::new("skill-activation-prompt")
///     .stdin(Stdio::piped())
///     .output()?;
/// # Ok::<(), std::io::Error>(())
/// ```
fn main() -> io::Result<()> {
```

**Verification:**

```bash
cargo doc --all-features --no-deps --open
# Should see well-formatted documentation
```

**Files to Modify:**

- All files in `src/bin/`

---

### 2.2 Add Unit Tests

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 3-4 hours

**Tasks:**

#### `skill_activation_prompt.rs` Tests

- [ ] Test keyword matching (case-insensitive)
- [ ] Test intent pattern matching (regex)
- [ ] Test priority grouping (critical/high/medium/low)
- [ ] Test empty matches (no output)
- [ ] Test malformed JSON input (error handling)

#### `file_analyzer.rs` Tests

- [ ] Test `get_file_category()` with various paths
  - [ ] Frontend paths
  - [ ] Backend paths
  - [ ] Database paths
  - [ ] Other paths
- [ ] Test `should_analyze()` filters
  - [ ] Valid extensions (.ts, .tsx, .js, etc.)
  - [ ] Test files (should skip)
  - [ ] Config files (should skip)
- [ ] Test regex pattern matching
  - [ ] Async detection
  - [ ] Try/catch detection
  - [ ] Prisma detection
  - [ ] Controller detection
  - [ ] API call detection

#### `post_tool_use_tracker_sqlite.rs` Tests

- [ ] Test database creation
- [ ] Test file modification tracking
- [ ] Test session summary updates
- [ ] Test category counting (backend/frontend/database)
- [ ] Test file analysis integration
- [ ] Test concurrent access (if applicable)

**Test Structure:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_category_detection() {
        assert_eq!(get_file_category("/frontend/App.tsx"), "frontend");
        assert_eq!(get_file_category("/src/controllers/UserController.ts"), "backend");
        assert_eq!(get_file_category("/database/schema.sql"), "database");
        assert_eq!(get_file_category("/src/utils.ts"), "other");
    }

    #[test]
    fn test_should_analyze_filters() {
        assert!(should_analyze("/src/app.ts"));
        assert!(should_analyze("/components/Button.tsx"));
        assert!(!should_analyze("/src/app.test.ts"));
        assert!(!should_analyze("/config.json"));
        assert!(!should_analyze("/README.md"));
    }

    #[test]
    fn test_async_detection() {
        use once_cell::sync::Lazy;

        static ASYNC_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"async\s+").unwrap()
        });

        let content = "async function fetchData() { return data; }";
        assert!(ASYNC_REGEX.is_match(content));

        let sync_content = "function getData() { return data; }";
        assert!(!ASYNC_REGEX.is_match(sync_content));
    }
}
```

**Verification:**

```bash
cargo test --all-features
# All tests should pass

cargo test --all-features -- --nocapture
# See test output
```

**Files to Modify:**

- All files in `src/bin/` (add test modules)

---

### 2.3 Implement Common Traits

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 1 hour

**Tasks:**

- [ ] Add `Clone` to deserializable structs
- [ ] Add `PartialEq` where equality makes sense
- [ ] Add `Eq` + `Hash` for types used in collections
- [ ] Consider `Default` for initialization patterns

**Implementation:**

```rust
// Before
#[derive(Debug, Deserialize)]
struct HookInput { ... }

// After
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(dead_code)]  // JSON deserialization
struct HookInput { ... }

// For comparable types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MatchedSkill {
    name: String,
    priority: String,
}
```

**Checklist:**

- [ ] `HookInput` - Add `Clone`, `PartialEq`
- [ ] `PromptTriggers` - Add `Clone`, `PartialEq`
- [ ] `SkillRule` - Add `Clone`, `PartialEq`
- [ ] `SkillRules` - Add `Clone`, `PartialEq`
- [ ] `MatchedSkill` - Add `Clone`, `PartialEq`, `Eq`, `Hash`
- [ ] `FileAnalysis` - Add `Clone`, `PartialEq`
- [ ] `Stats` - Add `Clone`, `PartialEq`

**Verification:**

```bash
cargo check --all-features
# Should compile without issues
```

**Files to Modify:**

- All files in `catalyst-cli/src/bin/`

---

### 2.3a Cross-Platform Path Handling

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 2-3 hours

**Issue:** Consolidate ALL path handling refactoring for cross-platform compatibility (Windows/Linux/macOS). This was previously split between Phase 2.5 and 2.7, causing duplication.

**Current Problems:**

1. **String concatenation instead of Path methods**

```rust
// ❌ Current (Unix-only)
let rules_path = format!("{project_dir}/.claude/skills/skill-rules.json");

// ✅ Fixed (Cross-platform)
let rules_path = project_dir
    .join(".claude")
    .join("skills")
    .join("skill-rules.json");
```

2. **String contains for path detection**

```rust
// ❌ Current (Unix-only)
if path.contains("/frontend/") || path.contains("/client/") { ... }

// ✅ Fixed (Cross-platform)
fn get_file_category(path: &Path) -> &'static str {
    for component in path.components() {
        match component.as_os_str().to_str() {
            Some("frontend") | Some("client") => return "frontend",
            Some("backend") | Some("server") => return "backend",
            _ => continue,
        }
    }
    "other"
}
```

3. **Hardcoded Unix environment variables**

```rust
// ❌ Current
env::var("HOME")

// ✅ Fixed
fn get_home_dir() -> PathBuf {
    #[cfg(windows)]
    {
        env::var("USERPROFILE")
            .or_else(|_| env::var("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"))
    }

    #[cfg(not(windows))]
    {
        env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }
}
```

---

**Tasks:**

#### Update All Binaries

- [ ] **skill_activation_prompt.rs**
  - [ ] Use `PathBuf` for `CLAUDE_PROJECT_DIR`
  - [ ] Use `path.join()` for rules_path construction
  - [ ] Change function signatures: `&str` → `&Path` where appropriate

- [ ] **file_analyzer.rs**
  - [ ] Change `get_file_category(path: &str)` → `get_file_category(path: &Path)`
  - [ ] Use `path.components()` instead of string contains
  - [ ] Use `path.extension()` instead of string `ends_with()`
  - [ ] Update `should_analyze()` to use Path methods

- [ ] **post_tool_use_tracker_sqlite.rs**
  - [ ] Use Path for file path parameters
  - [ ] Use `path.components()` for category detection
  - [ ] Cross-platform database path construction

#### Add Cross-Platform Helper Functions

- [ ] Create `get_home_dir()` function (handles HOME vs USERPROFILE)
- [ ] Create `get_claude_hooks_dir()` function
- [ ] Create `normalize_path()` if needed (handle both `/` and `\` in input)

#### Test on All Platforms

- [ ] Verify builds on Linux
- [ ] Verify builds on macOS
- [ ] Verify builds on Windows
- [ ] Test with backslash paths (Windows)
- [ ] Test with forward slash paths (Unix)
- [ ] Test with spaces in paths (Program Files)
- [ ] Test with UNC paths (`\\server\share\`)

---

**Implementation Examples:**

```rust
// catalyst-cli/src/bin/skill_activation_prompt.rs

use std::path::{Path, PathBuf};
use std::env;

fn main() -> io::Result<()> {
    // Cross-platform project directory
    let project_dir = env::var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    // Cross-platform path construction
    let rules_path = project_dir
        .join(".claude")
        .join("skills")
        .join("skill-rules.json");

    let rules_content = fs::read_to_string(&rules_path)?;
    // ... rest of code
}
```

```rust
// catalyst-cli/src/bin/file_analyzer.rs

use std::path::{Path, PathBuf};

// Change from &str to &Path
fn get_file_category(path: &Path) -> &'static str {
    // Use path components instead of string contains
    for component in path.components() {
        match component.as_os_str().to_str() {
            Some("frontend") | Some("client") | Some("components") => {
                return "frontend";
            }
            Some("backend") | Some("server") | Some("api") | Some("controllers") => {
                return "backend";
            }
            Some("database") | Some("prisma") | Some("migrations") => {
                return "database";
            }
            _ => continue,
        }
    }
    "other"
}

// Use Path extension method
fn should_analyze(path: &Path) -> bool {
    // Check extension using Path API
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("rs") | Some("cs") => {
            // Skip test files
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                !file_name.contains(".test.") && !file_name.contains(".spec.")
            } else {
                false
            }
        }
        _ => false,
    }
}
```

```rust
// Cross-platform home directory helper

use std::path::PathBuf;
use std::env;

#[cfg(windows)]
fn get_home_dir() -> PathBuf {
    env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"))
}

#[cfg(not(windows))]
fn get_home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn get_claude_hooks_dir() -> PathBuf {
    get_home_dir()
        .join(".claude-hooks")
        .join("bin")
}
```

---

**Verification:**

```bash
# Local testing (all platforms)
cargo build --workspace --all-features
cargo test --workspace --all-features

# Test with Windows-style paths (on Windows)
set CLAUDE_PROJECT_DIR=C:\Users\test\project
.\target\release\skill-activation-prompt.exe < test-input.json

# Test with Unix-style paths (on Linux/macOS)
export CLAUDE_PROJECT_DIR=/home/test/project
./target/release/skill-activation-prompt < test-input.json

# CI will automatically test all platforms
git push  # CI runs on Linux, macOS, Windows
```

**Windows-Specific Testing:**

```powershell
# Test with spaces in path
$env:CLAUDE_PROJECT_DIR = "C:\Program Files\My Project"
.\target\release\skill-activation-prompt.exe < test.json

# Test with UNC path
$env:CLAUDE_PROJECT_DIR = "\\server\share\project"
.\target\release\skill-activation-prompt.exe < test.json

# Test file-analyzer with Windows paths
.\target\release\file-analyzer.exe "C:\Users\test\project"
```

---

**Files to Modify:**

- `catalyst-cli/src/bin/skill_activation_prompt.rs`
- `catalyst-cli/src/bin/file_analyzer.rs`
- `catalyst-cli/src/bin/post_tool_use_tracker_sqlite.rs`

**CI Validation:**

- ✅ Build succeeds on Windows, macOS, Linux
- ✅ Tests pass with different path formats
- ✅ Clippy passes (no path-related warnings)

**Priority:** HIGH - Must be done before Phase 2.4, 2.5, 2.6, 2.7 (blocks Windows work)

**Why Before Other Phases:**

- Phase 2.4 (CLI) will add new path handling - needs this foundation
- Phase 2.5 (Performance) uses ignore/globset with paths - needs Path API
- Phase 2.6 (Settings) will work with settings.json paths - needs this
- Phase 2.7 (Windows) needs all paths already working cross-platform

---

### 2.4 Improve CLI with Modern Crates

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 3-4 hours

**Issue:** Currently using manual argument parsing and basic error handling. Modern CLI crates would improve UX significantly.

**Current Problems:**

- Manual `env::args()` parsing in `file-analyzer.rs`
- No automatic `--help` generation
- Manual DEBUG_HOOKS env var checking
- Converting `serde_json::Error` to `io::Error` manually
- Basic println! without colors or structure

**Recommended Crates:**

#### Must-Have: `clap` (argument parsing)

**Why:** Better than manual `env::args()` parsing

**Benefits:**

- ✅ Automatic `--help` and `--version`
- ✅ Type-safe argument parsing
- ✅ Built-in validation
- ✅ Shell completion generation

#### Must-Have: `anyhow` (error handling)

**Why:** Already using in SQLite, should use everywhere

**Benefits:**

- ✅ `.context()` for helpful error messages
- ✅ No manual error type conversion
- ✅ Stack traces in debug mode

#### Recommended: `tracing` (structured logging)

**Why:** Better than manual `DEBUG_HOOKS` checking

**Benefits:**

- ✅ Structured logging (not just strings)
- ✅ Log levels (debug, info, warn, error)
- ✅ Controlled by `RUST_LOG=debug` env var
- ✅ Can output JSON for parsing

#### Nice-to-Have: `colored` (terminal colors)

**Why:** Visual hierarchy improves readability

**Benefits:**

- ✅ Better visual output
- ✅ Respects `NO_COLOR` env var
- ✅ Cross-platform (Windows/Unix)

#### Nice-to-Have: `indicatif` (progress bars)

**Why:** Feedback for long operations

**Benefits:**

- ✅ Shows progress during large scans
- ✅ Automatically hides when piped
- ✅ Estimated time remaining

**Tasks:**

#### Add Dependencies

- [ ] Add `clap = { version = "4.5", features = ["derive"] }` to Cargo.toml
- [ ] Add `anyhow = "1.0"` to core dependencies (not just SQLite feature)
- [ ] Add `tracing = "0.1"` to dependencies
- [ ] Add `tracing-subscriber = { version = "0.3", features = ["env-filter"] }` to dependencies
- [ ] Add `colored = "2.1"` to dependencies
- [ ] Add `indicatif = { version = "0.17", optional = true }` with feature flag

#### Update `file-analyzer.rs`

- [ ] Replace manual args parsing with `clap::Parser` derive
- [ ] Add `--verbose`, `--format`, `--no-color` flags
- [ ] Change `main()` return type to `anyhow::Result<()>`
- [ ] Add `.context()` to all error handling
- [ ] Replace `println!` with `colored` output
- [ ] Initialize tracing subscriber
- [ ] Replace manual prints with `info!`, `warn!`, `debug!` macros
- [ ] Add optional progress bar for large directories

#### Update `skill-activation-prompt.rs`

- [ ] Change return type from `io::Result<()>` to `anyhow::Result<()>`
- [ ] Add `.context()` for JSON parsing errors
- [ ] Add `.context()` for file reading errors
- [ ] Add colored output for different priority levels
- [ ] Add tracing for debugging

#### Update `post-tool-use-tracker-sqlite.rs`

- [ ] Already using `anyhow::Result` ✅
- [ ] Replace manual `DEBUG_HOOKS` check with `tracing::debug!`
- [ ] Add structured logging with fields

**Implementation Examples:**

```rust
// file-analyzer.rs with clap
use clap::Parser;

#[derive(Parser)]
#[command(name = "file-analyzer")]
#[command(about = "Analyzes files for error-prone patterns", long_about = None)]
struct Args {
    /// Directory to analyze
    directory: PathBuf,

    /// Show detailed output
    #[arg(short, long)]
    verbose: bool,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text", value_parser = ["text", "json"])]
    format: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Analyzing directory: {:?}", args.directory);
    // ...
}
```

```rust
// With anyhow context
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let data: HookInput = serde_json::from_str(&input)
        .context("Failed to parse hook input JSON")?;

    let rules_content = fs::read_to_string(&rules_path)
        .context(format!("Failed to read skill rules from {}", rules_path))?;

    let rules: SkillRules = serde_json::from_str(&rules_content)
        .context("Failed to parse skill rules JSON")?;
}
```

```rust
// With colored output
use colored::*;

println!("{}", "⚠️ CRITICAL SKILLS (REQUIRED):".red().bold());
for skill in critical {
    println!("  → {}", skill.name.yellow());
}
```

```rust
// With tracing instead of manual DEBUG_HOOKS
use tracing::{debug, info, warn};

// Replace this:
if std::env::var("DEBUG_HOOKS").is_ok() {
    eprintln!("[Rust/SQLite] Tracked: {file_path}");
}

// With this:
debug!(file_path = %file_path, category = %category, "Tracked file modification");

// Usage:
// RUST_LOG=debug ./post-tool-use-tracker-sqlite
```

**Updated Cargo.toml:**

```toml
[dependencies]
# Core dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
walkdir = "2.4"
once_cell = "1.19"  # From Phase 1

# CLI improvements (Phase 2.4)
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"  # Use everywhere, not just SQLite
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
colored = "2.1"

# Optional: Progress bars
indicatif = { version = "0.17", optional = true }

# SQLite feature dependencies
rusqlite = { version = "0.31", features = ["bundled"], optional = true }
chrono = { version = "0.4", optional = true }

[features]
default = []
sqlite = ["dep:rusqlite", "dep:chrono"]
progress = ["dep:indicatif"]  # Optional feature for progress bars
```

**Verification:**

```bash
# Test new CLI arguments
./target/release/file-analyzer --help
# Should show auto-generated help

./target/release/file-analyzer /path --verbose
# Should show detailed output

./target/release/file-analyzer /path --format json
# Should output JSON

# Test tracing
RUST_LOG=debug ./target/release/skill-activation-prompt < input.json
# Should show debug logs

# Test colored output
./target/release/file-analyzer /path
# Should show colored output

NO_COLOR=1 ./target/release/file-analyzer /path
# Should show plain output
```

**Files to Modify:**

- `Cargo.toml` - Add new dependencies
- `src/bin/skill_activation_prompt.rs` - Add tracing, anyhow, colored
- `src/bin/file_analyzer.rs` - Add clap, tracing, anyhow, colored
- `src/bin/post_tool_use_tracker_sqlite.rs` - Replace DEBUG_HOOKS with tracing

**Example Output (Before vs After):**

Before:

```
Usage: file-analyzer <directory>

Analyzes files in directory for error-prone patterns
```

After:

```
$ file-analyzer --help
Analyzes files for error-prone patterns

Usage: file-analyzer [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>  Directory to analyze

Options:
  -v, --verbose          Show detailed output
  -f, --format <FORMAT>  Output format (text, json) [default: text]
      --no-color         Suppress colored output
  -h, --help             Print help
  -V, --version          Print version
```

**Benefits:**

- ✅ Professional CLI interface
- ✅ Better error messages with context
- ✅ Structured logging that can be disabled
- ✅ Colored output that respects NO_COLOR
- ✅ JSON output for machine parsing
- ✅ Progress feedback for large operations
- ✅ Automatic --help generation

**Priority:** MEDIUM-HIGH - Significantly improves usability

---

#### 2.4b Windows CLI Support

**Status:** ❌ Not Started
**Effort:** 1 hour

**Issue:** Ensure all CLI improvements work correctly on Windows.

**Tasks:**

- [ ] Test colored output on Windows Terminal and PowerShell
- [ ] Verify clap argument parsing works with Windows paths
- [ ] Test tracing output to Windows Event Log (optional)
- [ ] Ensure progress bars work in PowerShell
- [ ] Test `NO_COLOR` environment variable on Windows
- [ ] Verify `--help` formatting in cmd.exe and PowerShell
- [ ] Test with paths containing spaces (Program Files)

**Windows-Specific Considerations:**

```powershell
# Test colored output
.\target\release\file-analyzer.exe C:\project
# Should show colors in Windows Terminal

# Test NO_COLOR
$env:NO_COLOR = "1"
.\target\release\file-analyzer.exe C:\project
# Should show no colors

# Test with spaces in path
.\target\release\file-analyzer.exe "C:\Program Files\My Project"
# Should handle path correctly

# Test JSON output
.\target\release\file-analyzer.exe C:\project --format json
# Should output valid JSON

# Test tracing on Windows
$env:RUST_LOG = "debug"
.\target\release\skill-activation-prompt.exe < input.json
# Should show debug logs
```

**Known Windows Quirks:**

- Windows Terminal supports ANSI colors, cmd.exe may not (colored crate handles this)
- PowerShell requires backtick (`) for line continuation, not backslash
- Environment variables use `$env:VAR` syntax
- Paths use backslashes but forward slashes also work
- Admin rights may be required for some operations

**CI Validation:**

- ✅ All CLI tools build on windows-latest
- ✅ Help text displays correctly
- ✅ Colored output works (or gracefully degrades)
- ✅ Paths with spaces handled correctly

**Files to Test:**

- All binaries in `catalyst-cli/src/bin/`
- Verify on Windows 10+ with both cmd.exe and PowerShell

---

### 2.5 Performance Optimizations

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 2-3 hours

**Issue:** Current code has performance bottlenecks in string operations, directory traversal, and pattern matching.

**Prerequisites:** Phase 2.3a (Cross-Platform Path Handling) must be complete first.

**Current Problems:**

#### 1. Excessive `to_lowercase()` Allocations

```rust
// skill_activation_prompt.rs:55,73
let prompt = data.prompt.to_lowercase();  // Allocates
let keyword_match = triggers.keywords.iter()
    .any(|kw| prompt.contains(&kw.to_lowercase()));  // Allocates for each keyword!
```

**Problem:** Each `to_lowercase()` allocates a new String. For 100 keywords, that's 100+ allocations.

**Solution:** Use `unicase` crate for zero-allocation case-insensitive comparison.

#### 2. No .gitignore Support

```rust
for entry in WalkDir::new(dir) { ... }
```

**Problem:** Scans `node_modules/`, `.git/`, `target/`, etc. unnecessarily. Can take 10-100x longer than needed.

**Solution:** Use `ignore` crate which respects .gitignore files (same as ripgrep).

#### 3. Inefficient Extension/Pattern Matching

```rust
// Checking 6 extensions one by one (O(n))
path.extension() == Some("ts")
    || path.extension() == Some("tsx")
    || path.extension() == Some("js")
    // ... etc
```

**Problem:** Linear search through extensions for every file.

**Solution:** Use `globset` crate to compile patterns once, match efficiently.

#### 4. Sequential Directory Traversal

```rust
for entry in WalkDir::new(dir) {
    analyze_file(entry.path());  // Sequential, single-threaded
}
```

**Problem:** Large directory scans are sequential, even though each file analysis is independent. Only uses 1 CPU core.

**Solution:** Use `rayon` for parallel processing (4-8x faster on multi-core).

#### 5. Multiple Keyword Matching (Linear)

```rust
// Check each keyword one by one
triggers.keywords.iter()
    .any(|kw| prompt.contains(&kw.to_lowercase()))
```

**Problem:** For N keywords, does N passes through the prompt string.

**Solution:** Use `aho-corasick` for multi-pattern matching (single pass).

---

**Recommended Crates:**

#### Must-Have: `ignore` crate

**Why:** Respects .gitignore, .ignore files (used by ripgrep)

**Benefits:**

- ✅ Skips node_modules, .git, target automatically
- ✅ 10-100x faster for large repos
- ✅ Respects .gitignore patterns
- ✅ Cross-platform path handling

**Before:**

```rust
for entry in WalkDir::new(dir) { ... }
// Scans everything including node_modules/
```

**After:**

```rust
use ignore::WalkBuilder;

for entry in WalkBuilder::new(dir).build() { ... }
// Automatically skips ignored files
```

#### Must-Have: `globset` crate

**Why:** Efficient file extension matching

**Benefits:**

- ✅ Compile patterns once
- ✅ Match multiple patterns efficiently
- ✅ Cross-platform glob support

**Before:**

```rust
path_lower.ends_with(".ts")
    || path_lower.ends_with(".tsx")
    || path_lower.ends_with(".js")
    // ... 6 checks
```

**After:**

```rust
use globset::{Glob, GlobSetBuilder};

// Compile once
static CODE_EXTENSIONS: Lazy<GlobSet> = Lazy::new(|| {
    GlobSetBuilder::new()
        .add(Glob::new("*.ts").unwrap())
        .add(Glob::new("*.tsx").unwrap())
        .add(Glob::new("*.js").unwrap())
        .add(Glob::new("*.jsx").unwrap())
        .add(Glob::new("*.rs").unwrap())
        .add(Glob::new("*.cs").unwrap())
        .build()
        .unwrap()
});

// Use
CODE_EXTENSIONS.is_match(path)
```

#### Recommended: `unicase` crate

**Why:** Case-insensitive comparison without allocation

**Benefits:**

- ✅ No `to_lowercase()` allocations
- ✅ Direct case-insensitive comparison
- ✅ Works with HashSet/HashMap

**Before:**

```rust
let prompt = data.prompt.to_lowercase();  // Allocates
let keyword_match = triggers.keywords.iter()
    .any(|kw| prompt.contains(&kw.to_lowercase()));  // Multiple allocations
```

**After:**

```rust
use unicase::UniCase;

// No allocations
let keyword_match = triggers.keywords.iter()
    .any(|kw| UniCase::new(&data.prompt).contains(&UniCase::new(kw)));
```

#### Nice-to-Have: `rayon` crate

**Why:** Parallel iteration for large directories

**Benefits:**

- ✅ Automatic parallelization
- ✅ 2-8x faster on multi-core systems
- ✅ Easy to use (just change `.iter()` to `.par_iter()`)

**Before:**

```rust
for entry in WalkDir::new(dir) {
    analyze_file(entry.path());  // Sequential
}
```

**After:**

```rust
use rayon::prelude::*;

entries.par_iter().for_each(|entry| {
    analyze_file(entry.path());  // Parallel!
});
```

#### Nice-to-Have: `aho-corasick` crate

**Why:** Multi-pattern string matching (better than multiple contains())

**Benefits:**

- ✅ Match multiple keywords in single pass
- ✅ Much faster than multiple `.contains()` calls
- ✅ Used by ripgrep for speed

**Before:**

```rust
// Checks each keyword one by one
triggers.keywords.iter()
    .any(|kw| prompt.contains(&kw.to_lowercase()))
```

**After:**

```rust
use aho_corasick::AhoCorasick;

static KEYWORD_MATCHER: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasick::new_auto_configured(&["backend", "api", "prisma", ...])
});

// Single pass through prompt
KEYWORD_MATCHER.is_match(&prompt)
```

---

**Tasks:**

#### Add Dependencies to catalyst-cli/Cargo.toml

- [ ] Add `ignore = "0.4"` (must-have - .gitignore support)
- [ ] Add `globset = "0.4"` (must-have - pattern matching)
- [ ] Add `unicase = "2.7"` (recommended - zero-alloc comparison)
- [ ] Add `rayon = "1.8"` (optional - parallel processing)
- [ ] Add `aho-corasick = "1.1"` (optional - multi-pattern matching)

#### Replace WalkDir with ignore crate (file_analyzer.rs)

- [ ] Replace `WalkDir::new(dir)` with `WalkBuilder::new(dir).build()`
- [ ] Test that .gitignore is respected (skips node_modules/)
- [ ] Benchmark: Should be 10x faster on repos with node_modules
- [ ] Add Windows test: Verify respects .gitignore on Windows

#### Use globset for Extension Matching (file_analyzer.rs)

- [ ] Create static `Lazy<GlobSet>` for code file extensions
- [ ] Replace chain of `path.extension() == Some(...)` with `GlobSet::is_match()`
- [ ] Add test file patterns (.test., .spec.) to separate GlobSet
- [ ] Benchmark: Should be O(1) instead of O(n)

#### Optimize Case-Insensitive Matching (skill_activation_prompt.rs)

- [ ] Replace `prompt.to_lowercase()` with `UniCase::new(prompt)`
- [ ] Replace `kw.to_lowercase()` in loop with `UniCase::new(kw)`
- [ ] Benchmark: Should eliminate 100+ allocations
- [ ] Verify case-insensitive matching still works correctly

#### Optional: Add Parallel Processing (file_analyzer.rs)

- [ ] Add `rayon` feature flag to Cargo.toml
- [ ] Change `entries.iter()` to `entries.par_iter()`
- [ ] Test performance on large directories (>1000 files)
- [ ] Expected improvement: 4-8x faster on multi-core systems
- [ ] Add Windows test: Verify parallel processing works on Windows

#### Optional: Multi-Pattern String Matching (skill_activation_prompt.rs)

- [ ] Create static `Lazy<AhoCorasick>` with all keywords
- [ ] Replace `keywords.iter().any(...)` with `KEYWORD_MATCHER.is_match()`
- [ ] Benchmark: Should be single-pass instead of N passes
- [ ] Test with 100+ keywords

---

**Implementation Examples:**

```rust
// 1. Replace WalkDir with ignore crate (respects .gitignore)
use ignore::WalkBuilder;

// Before: Scans everything including node_modules/
for entry in WalkDir::new(dir) { ... }

// After: Automatically skips ignored files
for result in WalkBuilder::new(&args.directory).build() {
    let entry = result?;
    if entry.file_type().is_some_and(|ft| ft.is_file()) {
        analyze_file(entry.path());
    }
}
```

```rust
// 2. Use globset for efficient extension matching
use globset::{Glob, GlobSet, GlobSetBuilder};
use once_cell::sync::Lazy;

static CODE_EXTENSIONS: Lazy<GlobSet> = Lazy::new(|| {
    let mut builder = GlobSetBuilder::new();
    for pattern in &["*.ts", "*.tsx", "*.js", "*.jsx", "*.rs", "*.cs"] {
        builder.add(Glob::new(pattern).unwrap());
    }
    builder.build().unwrap()
});

// O(1) lookup instead of O(n) chain
fn should_analyze(path: &Path) -> bool {
    CODE_EXTENSIONS.is_match(path)
}
```

```rust
// 3. Case-insensitive comparison without allocation
use unicase::UniCase;

// Before: Allocates for every comparison
let keyword_match = triggers.keywords.iter()
    .any(|kw| prompt.to_lowercase().contains(&kw.to_lowercase()));

// After: Zero allocations
let keyword_match = triggers.keywords.iter()
    .any(|kw| UniCase::new(&prompt).as_ref().contains(UniCase::new(kw).as_ref()));
```

```rust
// 4. Parallel directory traversal (optional)
use rayon::prelude::*;

// Before: Sequential (uses 1 core)
for entry in entries {
    analyze_file(entry.path());
}

// After: Parallel (uses all cores)
let results: Vec<_> = entries
    .par_iter()
    .filter_map(|entry| analyze_file(entry.path()).ok())
    .collect();
```

```rust
// 5. Multi-pattern matching in single pass (optional)
use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;

static KEYWORD_MATCHER: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(&["backend", "api", "controller", "service", "route", "prisma"])
        .unwrap()
});

// Single pass through prompt string
fn has_keyword(prompt: &str) -> bool {
    KEYWORD_MATCHER.is_match(prompt)
}
```

---

**Updated Cargo.toml:**

```toml
[dependencies]
# Core dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
once_cell = "1.19"

# String/Path/Pattern optimizations (Phase 2.5)
ignore = "0.4"          # Respects .gitignore (must-have)
globset = "0.4"         # Efficient pattern matching (must-have)
unicase = "2.7"         # Case-insensitive without allocation (recommended)

# CLI improvements
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
colored = "2.1"

# Optional optimizations
rayon = { version = "1.8", optional = true }           # Parallel processing
aho-corasick = { version = "1.1", optional = true }    # Multi-pattern matching
indicatif = { version = "0.17", optional = true }      # Progress bars

# SQLite feature dependencies
rusqlite = { version = "0.31", features = ["bundled"], optional = true }
chrono = { version = "0.4", optional = true }

[features]
default = []
sqlite = ["dep:rusqlite", "dep:chrono"]
parallel = ["dep:rayon"]              # Enable parallel processing
fast-patterns = ["dep:aho-corasick"]  # Enable multi-pattern matching
progress = ["dep:indicatif"]          # Enable progress bars
```

---

**Performance Impact:**

| Optimization | Before | After | Improvement |
|--------------|--------|-------|-------------|
| Directory scan (with node_modules) | ~2000ms | ~200ms | 10x faster |
| Extension matching (1000 files) | ~50ms | ~5ms | 10x faster |
| Keyword matching (100 keywords) | ~100 allocations | 0 allocations | ∞ better |
| Case-insensitive search | Allocates | No allocation | Memory efficient |
| Large directory (10k files) | Sequential | Parallel | 4-8x faster |

---

**Verification:**

```bash
# Test .gitignore support
cd test-repo-with-node-modules
time ./target/release/file-analyzer .
# Should skip node_modules/, be much faster

# Test pattern matching
RUST_LOG=debug ./target/release/file-analyzer /path
# Should show which patterns matched

# Test parallel processing
time cargo run --release --features parallel --bin file-analyzer -- /large/dir
# Compare with non-parallel version

# Memory profiling
valgrind --tool=massif ./target/release/skill-activation-prompt < input.json
# Should show fewer allocations
```

---

**Files to Modify:**

- `catalyst-cli/Cargo.toml` - Add dependencies and optional features
- `catalyst-cli/src/bin/file_analyzer.rs` - ignore crate, globset, rayon
- `catalyst-cli/src/bin/skill_activation_prompt.rs` - unicase, aho-corasick

**CI Validation:**

- ✅ Build succeeds on all platforms (Linux, macOS, Windows)
- ✅ Tests pass with new performance optimizations
- ✅ Benchmarks show expected improvements
- ✅ .gitignore respected on all platforms

**Priority:** MEDIUM - Significant performance improvements, especially for large codebases

**Note:** Phase 2.3a (Cross-Platform Path Handling) must be complete before starting this phase. Path refactoring is a prerequisite for using ignore/globset effectively.

---

### 2.6 Typesafe Settings.json Management

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 2-3 hours

**Issue:** Need to parse, create, and update `.claude/settings.json` in a typesafe way for installation and configuration management.

**Note:** Settings management belongs in `catalyst-core` as shared library code that all binaries can use.

**Current Challenges:**

- Manual JSON manipulation is error-prone
- No validation of hook configurations
- Installation scripts would benefit from programmatic settings updates
- Need to merge configurations without breaking existing settings

**Settings.json Structure to Support:**

```json
{
  "enableAllProjectMcpServers": true,
  "enabledMcpjsonServers": ["mysql", "sequential-thinking"],
  "permissions": {
    "allow": ["Edit:*", "Write:*"],
    "defaultMode": "acceptEdits"
  },
  "hooks": {
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
      }]
    }],
    "PostToolUse": [{
      "matcher": "Edit|MultiEdit|Write",
      "hooks": [{
        "type": "command",
        "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use-tracker.sh"
      }]
    }]
  }
}
```

---

**Recommended Approach:**

Use `serde` with strongly-typed structs to ensure type safety and validation.

**Tasks:**

#### Create Settings Data Structures

- [ ] Create `catalyst-core/src/settings.rs` module (shared library)
- [ ] Define `ClaudeSettings` root struct
- [ ] Define `Permissions` struct
- [ ] Define `Hooks` struct with HashMap for event types
- [ ] Define `HookConfig` struct (matcher + hooks array)
- [ ] Define `Hook` struct (type + command)
- [ ] Add serde derive macros with proper field renaming
- [ ] Export from `catalyst-core/src/lib.rs`

#### Implement Core Operations

- [ ] Implement `ClaudeSettings::read(path)` - Load from file
- [ ] Implement `ClaudeSettings::write(path)` - Save to file
- [ ] Implement `ClaudeSettings::merge(other)` - Merge configurations
- [ ] Implement validation methods
  - [ ] Validate hook commands exist
  - [ ] Validate matcher patterns are valid regex
  - [ ] Validate permission patterns

#### Add CLI Tool (Optional)

- [ ] Create `catalyst-cli/src/bin/settings_manager.rs` binary
- [ ] Add commands: `read`, `validate`, `add-hook`, `remove-hook`, `merge`
- [ ] Add `--dry-run` flag for safety
- [ ] Add pretty-printing with colors

#### Update Installation Scripts

- [ ] Use interactive prompts to guide users through settings setup
- [ ] Preserve existing user settings (merge, don't overwrite)
- [ ] Add backup before modifications
- [ ] Support both bash (install.sh) and PowerShell (install.ps1)

#### CI Validation

- [ ] CI runs `cargo build --bin settings-manager` to ensure it compiles
- [ ] CI runs unit tests for settings module: `cargo test settings`
- [ ] Verify builds on all platforms (Linux, macOS, Windows)

---

**Implementation Examples:**

```rust
// catalyst-core/src/settings.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettings {
    #[serde(default)]
    pub enable_all_project_mcp_servers: bool,

    #[serde(default)]
    pub enabled_mcpjson_servers: Vec<String>,

    #[serde(default)]
    pub permissions: Option<Permissions>,

    #[serde(default)]
    pub hooks: HashMap<String, Vec<HookConfig>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    #[serde(default)]
    pub allow: Vec<String>,

    #[serde(default)]
    pub default_mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,

    pub hooks: Vec<Hook>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hook {
    pub r#type: String,  // "command"
    pub command: String,
}

impl ClaudeSettings {
    /// Read settings from file
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read settings file")?;

        let settings: ClaudeSettings = serde_json::from_str(&content)
            .context("Failed to parse settings JSON")?;

        Ok(settings)
    }

    /// Write settings to file with pretty formatting
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize settings")?;

        fs::write(path.as_ref(), json)
            .context("Failed to write settings file")?;

        Ok(())
    }

    /// Add a hook to the configuration
    pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) {
        self.hooks
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(hook_config);
    }

    /// Remove hooks matching a command pattern
    pub fn remove_hook(&mut self, event: &str, command_pattern: &str) {
        if let Some(configs) = self.hooks.get_mut(event) {
            configs.retain(|config| {
                config.hooks.iter().all(|h| !h.command.contains(command_pattern))
            });
        }
    }

    /// Merge another settings object into this one
    pub fn merge(&mut self, other: ClaudeSettings) {
        // Merge MCP servers
        for server in other.enabled_mcpjson_servers {
            if !self.enabled_mcpjson_servers.contains(&server) {
                self.enabled_mcpjson_servers.push(server);
            }
        }

        // Merge permissions
        if let Some(other_perms) = other.permissions {
            if let Some(ref mut perms) = self.permissions {
                for allow in other_perms.allow {
                    if !perms.allow.contains(&allow) {
                        perms.allow.push(allow);
                    }
                }
            } else {
                self.permissions = Some(other_perms);
            }
        }

        // Merge hooks
        for (event, configs) in other.hooks {
            self.hooks
                .entry(event)
                .or_insert_with(Vec::new)
                .extend(configs);
        }
    }

    /// Validate the settings
    pub fn validate(&self) -> Result<()> {
        // Check hook commands reference valid files
        for (event, configs) in &self.hooks {
            for config in configs {
                // Validate matcher is valid regex if present
                if let Some(ref matcher) = config.matcher {
                    regex::Regex::new(matcher)
                        .context(format!("Invalid matcher regex in {} hook: {}", event, matcher))?;
                }

                // Validate hooks array not empty
                if config.hooks.is_empty() {
                    anyhow::bail!("Empty hooks array in {} event", event);
                }

                for hook in &config.hooks {
                    // Validate hook type
                    if hook.r#type != "command" {
                        anyhow::bail!("Unknown hook type '{}' in {} event", hook.r#type, event);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for ClaudeSettings {
    fn default() -> Self {
        Self {
            enable_all_project_mcp_servers: false,
            enabled_mcpjson_servers: Vec::new(),
            permissions: None,
            hooks: HashMap::new(),
        }
    }
}
```

**CLI Tool Example:**

```rust
// catalyst-cli/src/bin/settings_manager.rs

use clap::{Parser, Subcommand};
use anyhow::Result;
use catalyst_core::settings::*;

#[derive(Parser)]
#[command(name = "settings-manager")]
#[command(about = "Manage Claude Code settings.json files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read and display settings
    Read {
        /// Path to settings.json
        #[arg(default_value = ".claude/settings.json")]
        path: String,
    },

    /// Validate settings file
    Validate {
        /// Path to settings.json
        #[arg(default_value = ".claude/settings.json")]
        path: String,
    },

    /// Add a hook to settings
    AddHook {
        /// Path to settings.json
        #[arg(short, long, default_value = ".claude/settings.json")]
        path: String,

        /// Hook event (UserPromptSubmit, PostToolUse, Stop)
        #[arg(short, long)]
        event: String,

        /// Hook command
        #[arg(short, long)]
        command: String,

        /// Optional matcher pattern
        #[arg(short, long)]
        matcher: Option<String>,

        /// Dry run (don't write changes)
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove hooks matching pattern
    RemoveHook {
        /// Path to settings.json
        #[arg(short, long, default_value = ".claude/settings.json")]
        path: String,

        /// Hook event
        #[arg(short, long)]
        event: String,

        /// Command pattern to match
        #[arg(short, long)]
        pattern: String,

        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },

    /// Merge two settings files
    Merge {
        /// Base settings file
        base: String,

        /// Settings to merge in
        merge: String,

        /// Output file (defaults to base)
        #[arg(short, long)]
        output: Option<String>,

        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Read { path } => {
            let settings = ClaudeSettings::read(&path)?;
            let json = serde_json::to_string_pretty(&settings)?;
            println!("{}", json);
        }

        Commands::Validate { path } => {
            let settings = ClaudeSettings::read(&path)?;
            settings.validate()?;
            println!("✅ Settings file is valid");
        }

        Commands::AddHook { path, event, command, matcher, dry_run } => {
            let mut settings = ClaudeSettings::read(&path)
                .unwrap_or_default();

            let hook_config = HookConfig {
                matcher,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command,
                }],
            };

            settings.add_hook(&event, hook_config);

            if dry_run {
                println!("🔍 Dry run - would write:");
                println!("{}", serde_json::to_string_pretty(&settings)?);
            } else {
                settings.write(&path)?;
                println!("✅ Hook added to {}", path);
            }
        }

        Commands::RemoveHook { path, event, pattern, dry_run } => {
            let mut settings = ClaudeSettings::read(&path)?;
            settings.remove_hook(&event, &pattern);

            if dry_run {
                println!("🔍 Dry run - would write:");
                println!("{}", serde_json::to_string_pretty(&settings)?);
            } else {
                settings.write(&path)?;
                println!("✅ Hooks removed from {}", path);
            }
        }

        Commands::Merge { base, merge, output, dry_run } => {
            let mut base_settings = ClaudeSettings::read(&base)?;
            let merge_settings = ClaudeSettings::read(&merge)?;

            base_settings.merge(merge_settings);

            let output_path = output.as_deref().unwrap_or(&base);

            if dry_run {
                println!("🔍 Dry run - would write to {}:", output_path);
                println!("{}", serde_json::to_string_pretty(&base_settings)?);
            } else {
                base_settings.write(output_path)?;
                println!("✅ Settings merged to {}", output_path);
            }
        }
    }

    Ok(())
}
```

**Usage Examples:**

```bash
# Read settings
./settings-manager read .claude/settings.json

# Validate settings
./settings-manager validate .claude/settings.json

# Add a hook
./settings-manager add-hook \
  --event UserPromptSubmit \
  --command '$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh'

# Add a PostToolUse hook with matcher
./settings-manager add-hook \
  --event PostToolUse \
  --command '$CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use-tracker.sh' \
  --matcher 'Edit|Write|MultiEdit'

# Remove hooks matching pattern
./settings-manager remove-hook \
  --event UserPromptSubmit \
  --pattern 'skill-activation'

# Merge settings (dry run first)
./settings-manager merge base.json new-hooks.json --dry-run
./settings-manager merge base.json new-hooks.json --output merged.json

# Validate before writing
./settings-manager validate merged.json
```

**Updated install.sh with Interactive Prompts:**

```bash
#!/bin/bash
# install.sh

# ... build code ...

SETTINGS_FILE="$HOME/.claude/settings.json"

# Interactive prompt for settings configuration
echo "📝 Settings Configuration"
echo ""

if [ -f "$SETTINGS_FILE" ]; then
    echo "Found existing settings.json at $SETTINGS_FILE"
    read -p "Do you want to update it with Catalyst hooks? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping settings.json configuration."
        exit 0
    fi

    # Backup existing settings
    cp "$SETTINGS_FILE" "$SETTINGS_FILE.backup"
    echo "✅ Backup saved to $SETTINGS_FILE.backup"
else
    echo "No settings.json found. Creating new one..."
    mkdir -p "$(dirname "$SETTINGS_FILE")"
    cat > "$SETTINGS_FILE" << 'EOF'
{
  "hooks": {}
}
EOF
fi

# Show what will be added
echo ""
echo "The following hooks will be configured:"
echo "  • UserPromptSubmit: skill-activation-prompt.sh"
echo "  • PostToolUse: post-tool-use-tracker.sh"
echo ""

# Use settings-manager for the actual merge (optional tool)
if [ -f "./target/release/settings-manager" ]; then
    ./target/release/settings-manager add-hook \
      --path "$SETTINGS_FILE" \
      --event UserPromptSubmit \
      --command '$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh'

    ./target/release/settings-manager add-hook \
      --path "$SETTINGS_FILE" \
      --event PostToolUse \
      --command '$CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use-tracker.sh' \
      --matcher 'Edit|Write|MultiEdit'

    ./target/release/settings-manager validate "$SETTINGS_FILE"
else
    echo "⚠️  settings-manager not built. Please manually add hooks to $SETTINGS_FILE"
    echo "See the documentation for hook configuration examples."
fi

echo "✅ Settings configuration complete"
```

---

**Benefits:**

- ✅ **Type Safety** - Compile-time validation of structure
- ✅ **Validation** - Runtime validation of hook configurations
- ✅ **Merge Support** - Safely merge configurations without losing data
- ✅ **CLI Tool** - Easy configuration management
- ✅ **Installation** - Programmatic hook setup
- ✅ **Backup Safety** - Always backup before modifications
- ✅ **Dry Run** - Preview changes before applying

**Testing:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_settings() {
        let json = r#"{
            "enableAllProjectMcpServers": true,
            "hooks": {
                "UserPromptSubmit": [{
                    "hooks": [{
                        "type": "command",
                        "command": "test.sh"
                    }]
                }]
            }
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert!(settings.enable_all_project_mcp_servers);
        assert_eq!(settings.hooks.len(), 1);
    }

    #[test]
    fn test_add_hook() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook("UserPromptSubmit", HookConfig {
            matcher: None,
            hooks: vec![Hook {
                r#type: "command".to_string(),
                command: "test.sh".to_string(),
            }],
        });

        assert_eq!(settings.hooks.len(), 1);
    }

    #[test]
    fn test_merge_settings() {
        let mut base = ClaudeSettings::default();
        base.enabled_mcpjson_servers.push("mysql".to_string());

        let mut other = ClaudeSettings::default();
        other.enabled_mcpjson_servers.push("playwright".to_string());

        base.merge(other);

        assert_eq!(base.enabled_mcpjson_servers.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook("UserPromptSubmit", HookConfig {
            matcher: Some("[invalid regex".to_string()),
            hooks: vec![Hook {
                r#type: "command".to_string(),
                command: "test.sh".to_string(),
            }],
        });

        assert!(settings.validate().is_err());
    }
}
```

---

**Verification:**

```bash
# Build with settings manager
cargo build --release --bin settings-manager

# Test reading existing settings
./target/release/settings-manager read .claude/settings.json

# Test validation
./target/release/settings-manager validate .claude/settings.json

# Test adding hook (dry run)
./target/release/settings-manager add-hook \
  --event UserPromptSubmit \
  --command 'test.sh' \
  --dry-run

# Run unit tests
cargo test settings
```

---

**Files to Create:**

- `catalyst-core/src/settings.rs` - Core data structures and operations (shared library)
- `catalyst-cli/src/bin/settings_manager.rs` - CLI tool (optional)

**Files to Modify:**

- `catalyst-core/Cargo.toml` - Add dependencies (serde, serde_json, anyhow, regex)
- `catalyst-core/src/lib.rs` - Add `pub mod settings;` export
- `catalyst-cli/Cargo.toml` - Add settings-manager binary, depend on catalyst-core
- `install.sh` - Interactive prompts for settings configuration
- `install.ps1` - PowerShell version with interactive prompts (see Phase 2.6b)

**Dependencies Already Have:**

- ✅ `serde` with derive feature
- ✅ `serde_json`
- ✅ `anyhow` (from Phase 2.4)
- ✅ `clap` (from Phase 2.4)
- ✅ `regex` (already in dependencies)

**Priority:** MEDIUM-HIGH - Essential for proper installation and configuration management

---

#### 2.6b Windows-Specific Settings Management

**Windows-Specific Testing:**

```powershell
# Build settings-manager on Windows
cargo build --release --bin settings-manager

# Test reading settings
.\target\release\settings-manager.exe read "$env:USERPROFILE\.claude\settings.json"

# Test validation
.\target\release\settings-manager.exe validate "$env:USERPROFILE\.claude\settings.json"

# Test adding hooks on Windows
.\target\release\settings-manager.exe add-hook `
  --event UserPromptSubmit `
  --command '$CLAUDE_PROJECT_DIR\.claude\hooks\skill-activation-prompt.sh' `
  --dry-run

# Test with PowerShell paths
.\target\release\settings-manager.exe add-hook `
  --path "C:\Users\username\.claude\settings.json" `
  --event PostToolUse `
  --command '$CLAUDE_PROJECT_DIR\.claude\hooks\post-tool-use-tracker.sh' `
  --matcher 'Edit|Write|MultiEdit'
```

**PowerShell install.ps1 Script:**

```powershell
# install.ps1
# Interactive installation script for Windows

# ... build code ...

$SettingsFile = "$env:USERPROFILE\.claude\settings.json"

Write-Host "📝 Settings Configuration" -ForegroundColor Cyan
Write-Host ""

if (Test-Path $SettingsFile) {
    Write-Host "Found existing settings.json at $SettingsFile"
    $response = Read-Host "Do you want to update it with Catalyst hooks? [y/N]"

    if ($response -notmatch '^[Yy]$') {
        Write-Host "Skipping settings.json configuration."
        exit 0
    }

    # Backup existing settings
    Copy-Item $SettingsFile "$SettingsFile.backup"
    Write-Host "✅ Backup saved to $SettingsFile.backup" -ForegroundColor Green
} else {
    Write-Host "No settings.json found. Creating new one..."
    $null = New-Item -ItemType Directory -Force -Path (Split-Path $SettingsFile)

    @{
        hooks = @{}
    } | ConvertTo-Json | Set-Content $SettingsFile
}

# Show what will be added
Write-Host ""
Write-Host "The following hooks will be configured:"
Write-Host "  • UserPromptSubmit: skill-activation-prompt.sh"
Write-Host "  • PostToolUse: post-tool-use-tracker.sh"
Write-Host ""

# Use settings-manager for the actual merge
$settingsManager = ".\target\release\settings-manager.exe"

if (Test-Path $settingsManager) {
    & $settingsManager add-hook `
        --path $SettingsFile `
        --event UserPromptSubmit `
        --command '$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh'

    & $settingsManager add-hook `
        --path $SettingsFile `
        --event PostToolUse `
        --command '$CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use-tracker.sh' `
        --matcher 'Edit|Write|MultiEdit'

    & $settingsManager validate $SettingsFile
} else {
    Write-Host "⚠️  settings-manager.exe not built. Please manually add hooks." -ForegroundColor Yellow
    Write-Host "See the documentation for hook configuration examples."
}

Write-Host "✅ Settings configuration complete" -ForegroundColor Green
```

**Windows Path Handling in Settings:**

The settings module must handle both forward slashes (Unix) and backslashes (Windows) in hook commands. The `PathBuf` type from Phase 2.3a handles this automatically.

```rust
// Cross-platform hook command paths
// Both of these work on Windows:
"$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
"$CLAUDE_PROJECT_DIR\\.claude\\hooks\\skill-activation-prompt.sh"

// PathBuf normalizes both to the platform-appropriate separator
```

**CI Validation for Windows:**

- [ ] Build settings-manager on Windows runner
- [ ] Test reading/writing settings.json on Windows paths
- [ ] Verify PowerShell install.ps1 script works
- [ ] Test with spaces in Windows paths (e.g., "C:\Program Files\...")

---

### 2.7 Windows-Specific Components

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 2-3 hours

**Issue:** Need PowerShell scripts, install.ps1, and Windows-specific documentation to complete Windows support.

**Note:** This phase focuses ONLY on Windows-specific files not covered by parallel Windows subsections (2.3b, 2.4b, 2.5b, 2.6b):
- Cross-platform path handling → See Phase 2.3a and 2.3b
- CLI improvements on Windows → See Phase 2.4b
- Settings management on Windows → See Phase 2.6b
- CI/CD for Windows → See Phase 0

**What This Phase Covers:**

- PowerShell hook wrapper scripts
- Main install.ps1 installation script
- Documentation updates for Windows users
- Git configuration for line endings
- Final cross-platform testing validation

---

**Required Changes:**

#### 1. Create PowerShell Hook Wrappers

**Tasks:**

- [ ] Create `.claude/hooks/skill-activation-prompt.ps1`
- [ ] Create `.claude/hooks/post-tool-use-tracker.ps1`
- [ ] Create `.claude/hooks/tsc-check.ps1` (if applicable for TypeScript projects)
- [ ] Create `.claude/hooks/trigger-build-resolver.ps1` (if applicable)
- [ ] Ensure wrappers call binaries from `$env:USERPROFILE\.claude-hooks\bin`
- [ ] Add fallback to project-local binaries if standalone not found

**Example: skill-activation-prompt.ps1**

```powershell
#!/usr/bin/env pwsh
# .claude/hooks/skill-activation-prompt.ps1

# Read from stdin and pipe to Rust binary
$input | & "$env:USERPROFILE\.claude-hooks\bin\skill-activation-prompt.exe"

# Fallback to project-local binary if standalone not found
if ($LASTEXITCODE -eq 1 -and -not (Test-Path "$env:USERPROFILE\.claude-hooks\bin\skill-activation-prompt.exe")) {
    if (Test-Path "$env:CLAUDE_PROJECT_DIR\target\release\skill-activation-prompt.exe") {
        $input | & "$env:CLAUDE_PROJECT_DIR\target\release\skill-activation-prompt.exe"
    }
}
```

**Example: post-tool-use-tracker.ps1**

```powershell
#!/usr/bin/env pwsh
# .claude/hooks/post-tool-use-tracker.ps1

$input | & "$env:USERPROFILE\.claude-hooks\bin\post-tool-use-tracker-sqlite.exe"

if ($LASTEXITCODE -eq 1 -and -not (Test-Path "$env:USERPROFILE\.claude-hooks\bin\post-tool-use-tracker-sqlite.exe")) {
    if (Test-Path "$env:CLAUDE_PROJECT_DIR\target\release\post-tool-use-tracker-sqlite.exe") {
        $input | & "$env:CLAUDE_PROJECT_DIR\target\release\post-tool-use-tracker-sqlite.exe"
    }
}
```

---

#### 2. Create Main install.ps1 Script

**Tasks:**

- [ ] Create `install.ps1` (PowerShell equivalent of install.sh)
- [ ] Handle Rust installation check (rustup)
- [ ] Build with cargo
- [ ] Create `~/.claude-hooks/bin/` directory
- [ ] Copy binaries to user directory
- [ ] Handle optional --sqlite flag
- [ ] Add to PATH or provide instructions

**Example: install.ps1**

```powershell
#!/usr/bin/env pwsh
# install.ps1 - Windows installation script

param(
    [switch]$Sqlite,
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: ./install.ps1 [-Sqlite] [-Help]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Sqlite    Build with SQLite support"
    Write-Host "  -Help      Show this help message"
    exit 0
}

# Check for Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust not found. Installing rustup..."
    Write-Host "Visit: https://rustup.rs/"
    Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
    .\rustup-init.exe -y
    Remove-Item rustup-init.exe
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

Write-Host "🔨 Building Catalyst hooks..."

# Build
$features = if ($Sqlite) { "--features sqlite" } else { "" }
$buildCmd = "cargo build --release $features"
Invoke-Expression $buildCmd

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed"
    exit 1
}

# Create installation directory
$installDir = "$env:USERPROFILE\.claude-hooks\bin"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Copy binaries
Write-Host "📦 Installing to $installDir..."
Copy-Item "target\release\skill-activation-prompt.exe" $installDir -Force
Copy-Item "target\release\file-analyzer.exe" $installDir -Force

if ($Sqlite) {
    Copy-Item "target\release\post-tool-use-tracker-sqlite.exe" $installDir -Force
}

# Copy source for reference
$srcDir = "$env:USERPROFILE\.claude-hooks\src"
New-Item -ItemType Directory -Force -Path $srcDir | Out-Null
Copy-Item -Recurse -Force "src\*" $srcDir

Write-Host "✅ Installation complete!"
Write-Host ""
Write-Host "Binaries installed to:"
Write-Host "  $installDir"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Copy hook wrappers to your project:"
Write-Host "     cp .claude/hooks/*.ps1 your-project/.claude/hooks/"
Write-Host "  2. Update your project's .claude/settings.json"
Write-Host "  3. Restart Claude Code"
```

---

#### 3. Git Configuration for Line Endings

**Tasks:**

- [ ] Create `.gitattributes` file
- [ ] Ensure PowerShell scripts use CRLF on Windows
- [ ] Ensure bash scripts use LF on all platforms
- [ ] Ensure Rust code uses LF on all platforms

**Example: .gitattributes**

```gitattributes
# Default to LF
* text=auto eol=lf

# PowerShell scripts need CRLF for Windows compatibility
*.ps1 text eol=crlf

# Bash scripts use LF
*.sh text eol=lf

# Rust source files
*.rs text eol=lf
*.toml text eol=lf

# Markdown and docs
*.md text eol=lf

# JSON files
*.json text eol=lf
```

---

#### 4. Documentation Updates

**Tasks:**

- [ ] Add Windows installation section to README.md
- [ ] Add PowerShell examples to docs/standalone-installation.md
- [ ] Add Windows troubleshooting guide
- [ ] Update CLAUDE.md with Windows integration instructions

**README.md Windows Section:**

```markdown
## Installation on Windows

### Prerequisites
- PowerShell 5.1+ (included in Windows 10+)
- [Rust](https://rustup.rs/) (installer will prompt if missing)

### Install
```powershell
# From PowerShell
.\install.ps1

# With SQLite support
.\install.ps1 -Sqlite
```

### Setup in Your Project

```powershell
# Copy hook wrappers
Copy-Item .claude/hooks/*.ps1 your-project/.claude/hooks/

# Update settings.json
.\target\release\settings-manager.exe add-hook `
  --event UserPromptSubmit `
  --command '$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.ps1'
```

```

---

#### 5. Cross-Platform Testing Validation

**Tasks:**

- [ ] Verify all binaries compile on Windows (done in Phase 0 CI)
- [ ] Test install.ps1 script end-to-end
- [ ] Test PowerShell hook wrappers from Claude Code
- [ ] Verify documentation is accurate for Windows users
- [ ] Test paths with spaces (e.g., "C:\Program Files\...")
- [ ] Validate final cross-platform testing matrix (see below)

**Cross-Platform Testing Matrix:**

| Feature | Linux | macOS | Windows | Notes |
|---------|-------|-------|---------|-------|
| Build (cargo) | ✅ | ✅ | ✅ | Validated in Phase 0 CI |
| Install script | ✅ | ✅ | ✅ | bash vs PowerShell |
| Hook execution | ✅ | ✅ | ✅ | .sh vs .ps1 |
| Path handling | ✅ | ✅ | ✅ | Phase 2.3a (PathBuf) |
| SQLite | ✅ | ✅ | ✅ | Bundled feature |
| Settings manager | ✅ | ✅ | ✅ | Phase 2.6/2.6b |
| CLI improvements | ✅ | ✅ | ✅ | Phase 2.4/2.4b |

---

**Summary - Files to Create:**

- [ ] `install.ps1` - Main Windows installer
- [ ] `.claude/hooks/skill-activation-prompt.ps1` - Hook wrapper
- [ ] `.claude/hooks/post-tool-use-tracker.ps1` - Hook wrapper
- [ ] `.gitattributes` - Line ending configuration
- [ ] README.md updates - Windows installation section
- [ ] docs/standalone-installation.md updates - PowerShell examples
- [ ] CLAUDE.md updates - Windows integration instructions

**Note:** Most cross-platform code changes are handled in Phases 2.3a, 2.4b, 2.5b, and 2.6b. This phase only creates Windows-specific scripts and documentation.

---

**Verification Checklist:**

```powershell
# On Windows machine:

# 1. Install script works
.\install.ps1 -Sqlite

# 2. Binaries installed correctly
Test-Path "$env:USERPROFILE\.claude-hooks\bin\skill-activation-prompt.exe"
Test-Path "$env:USERPROFILE\.claude-hooks\bin\post-tool-use-tracker-sqlite.exe"

# 3. Hook wrappers exist
Test-Path .claude\hooks\skill-activation-prompt.ps1
Test-Path .claude\hooks\post-tool-use-tracker.ps1

# 4. Hooks execute via wrapper
Get-Content test-input.json | .\.claude\hooks\skill-activation-prompt.ps1

# 5. Documentation complete
# - README.md has Windows section
# - docs/standalone-installation.md has PowerShell examples
# - CLAUDE.md mentions Windows support
```

**Priority:** MEDIUM - Completes Windows support started in parallel subsections (2.3b, 2.4b, 2.5b, 2.6b)

**Dependencies:**
- Phase 0 (CI must validate Windows builds)
- Phase 2.3a (Path handling foundation)
- Phase 2.4 and 2.4b (CLI improvements on all platforms)
- Phase 2.6 and 2.6b (Settings management with PowerShell examples)

---

## Phase 3: Polish & Nice-to-Haves 🟢

**Goal:** Professional-grade code quality
**Priority:** LOW
**Timeline:** Before crates.io publication

### 3.1 Complete Cargo.toml Metadata

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 15 minutes

**Tasks:**

- [ ] Update repository URL (remove placeholder)
- [ ] Add homepage URL
- [ ] Add readme field
- [ ] Add keywords (5 max)
- [ ] Add categories (5 max)
- [ ] Add documentation URL (docs.rs)
- [ ] Verify license field is correct

**Implementation:**

```toml
[package]
name = "catalyst"
version = "0.1.0"
edition = "2021"
authors = ["Catalyst Contributors"]
description = "High-performance Claude Code hooks for skill auto-activation"
license = "MIT"
repository = "https://github.com/yourorg/catalyst"
homepage = "https://github.com/yourorg/catalyst"
documentation = "https://docs.rs/catalyst"
readme = "README.md"
keywords = ["claude-code", "hooks", "automation", "ai", "productivity"]
categories = ["development-tools", "command-line-utilities"]
```

**Verification:**

```bash
cargo publish --dry-run
# Should pass validation
```

**Files to Modify:**

- `Cargo.toml`

---

### 3.2 Improve Error Types

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 1-2 hours

**Tasks:**

- [ ] Add `thiserror` dependency
- [ ] Create custom error type for `skill_activation_prompt`
- [ ] Create custom error type for `file_analyzer`
- [ ] Update error handling to use custom types
- [ ] Ensure error messages are helpful and actionable

**Implementation:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum SkillActivationError {
    #[error("Failed to read stdin: {0}")]
    StdinRead(#[from] std::io::Error),

    #[error("Invalid JSON input: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Skill rules file not found at {path}")]
    RulesNotFound { path: String },

    #[error("Failed to parse skill rules: {0}")]
    RulesInvalid(String),
}

fn main() -> Result<(), SkillActivationError> {
    // ...
}
```

**Verification:**

```bash
# Test error messages
echo "invalid json" | ./target/release/skill-activation-prompt
# Should show clear error message
```

**Files to Modify:**

- `Cargo.toml` - add `thiserror = "1.0"`
- `src/bin/skill_activation_prompt.rs`
- `src/bin/file_analyzer.rs`

---

### 3.3 Code Formatting & Linting

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 15 minutes

**Tasks:**

- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-features --fix --allow-dirty`
- [ ] Review and apply clippy suggestions
- [ ] Set up pre-commit hook for formatting (optional)

**Verification:**

```bash
cargo fmt --all -- --check
# Should show "No changes needed"

cargo clippy --all-features
# Should show 0 warnings
```

---

### 3.4 Integration Tests

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 2-3 hours

**Tasks:**

- [ ] Create `tests/` directory
- [ ] Write integration test for `skill-activation-prompt`
  - [ ] Test with sample skill-rules.json
  - [ ] Verify output format
  - [ ] Test error cases
- [ ] Write integration test for `file-analyzer`
  - [ ] Create test directory structure
  - [ ] Verify statistics output
- [ ] Write integration test for `post-tool-use-tracker-sqlite`
  - [ ] Test database creation
  - [ ] Verify data persistence

**Structure:**

```
tests/
├── integration_test.rs
├── fixtures/
│   ├── sample-skill-rules.json
│   └── test-files/
│       ├── frontend/
│       └── backend/
```

**Example:**

```rust
// tests/integration_test.rs
use std::process::Command;

#[test]
fn test_skill_activation_with_backend_prompt() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "skill-activation-prompt"])
        .input(r#"{"session_id":"test","prompt":"create backend API"}"#)
        .output()
        .expect("Failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend-dev-guidelines"));
}
```

**Verification:**

```bash
cargo test --test integration_test
```

---

### 3.5 Performance Benchmarks

**Status:** ❌ Not Started
**Assignee:** TBD
**Effort:** 1-2 hours

**Tasks:**

- [ ] Create `benches/` directory
- [ ] Add `criterion` to dev-dependencies
- [ ] Write benchmark for skill activation
- [ ] Write benchmark for file analysis
- [ ] Establish baseline metrics
- [ ] Document performance characteristics

**Implementation:**

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "skill_activation"
harness = false
```

```rust
// benches/skill_activation.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn skill_activation_benchmark(c: &mut Criterion) {
    c.bench_function("skill activation", |b| {
        b.iter(|| {
            // Benchmark code
        });
    });
}

criterion_group!(benches, skill_activation_benchmark);
criterion_main!(benches);
```

**Verification:**

```bash
cargo bench
# Should complete and generate report
```

---

## Quality Gates

**Note:** Phase 0 CI (GitHub Actions) automatically enforces baseline quality standards on every commit. See Phase 0 for CI configuration details.

### Phase 0 CI Enforcement (Automatic)

Phase 0 CI validates EVERY commit/PR:

- ✅ Zero compiler warnings (`cargo build` must succeed cleanly)
- ✅ Zero clippy warnings (`cargo clippy -D warnings`)
- ✅ All tests pass (`cargo test --all-features`)
- ✅ Code is formatted (`cargo fmt --check`)
- ✅ Builds succeed on Linux, macOS, Windows
- ✅ Install scripts run without errors

**Result:** These quality gates are ENFORCED automatically - you cannot merge code that violates them.

---

### Before Any 0.x Release

- [ ] All Phase 1 tasks complete
- [ ] Phase 0 CI passing on main branch (enforced automatically)
- [ ] No `unwrap()` or `expect()` in production code paths (use `?` operator or proper error handling)
- [ ] Zero `TODO` comments in committed code
- [ ] All binaries compile with `--release` flag
- [ ] Binaries run successfully on all three platforms (verified by CI)

### Before 1.0 Release

- [ ] All Phase 1 and Phase 2 tasks complete
- [ ] Phase 0 CI passing (enforced automatically)
- [ ] Documentation coverage ≥80% (run `cargo doc --workspace --no-deps`)
- [ ] Test coverage ≥70% (run `cargo tarpaulin` or equivalent)
- [ ] All public APIs have documentation with examples
- [ ] No public APIs marked `#[doc(hidden)]` without justification
- [ ] Performance benchmarks document baseline (if added in Phase 3)
- [ ] Binary size ≤3MB per binary
- [ ] Startup time ≤5ms for all binaries

### Before crates.io Publication

- [ ] All phases (0, 1, 2, 3) complete
- [ ] Phase 0 CI passing with zero warnings (enforced automatically)
- [ ] `cargo publish --dry-run` passes for all crates
- [ ] README.md is comprehensive with examples
- [ ] CHANGELOG.md exists and follows [Keep a Changelog](https://keepachangelog.com/)
- [ ] LICENSE file exists (MIT or chosen license)
- [ ] Cargo.toml metadata complete (see Phase 3.1)
- [ ] GitHub repository URL is correct (no placeholders)
- [ ] Documentation published to docs.rs successfully

---

## Metrics Tracking

| Metric | Current | Phase 1 Target | Phase 2 Target | Phase 3 Target |
|--------|---------|---------------|---------------|---------------|
| Compiler Warnings | 7 | 0 | 0 | 0 |
| Clippy Warnings | Unknown | 0 | 0 | 0 |
| Test Coverage | 0% | 0% | 70% | 80% |
| Doc Coverage | 0% | 10% | 80% | 95% |
| Best Practices Score | 19/40 | 28/40 | 35/40 | 38/40 |

---

## Dependencies to Add

### Phase 1

```toml
once_cell = "1.19"  # For lazy static regexes
```

### Phase 2 (Modern CLI & Performance)

```toml
# CLI Improvements (Phase 2.4)
clap = { version = "4.5", features = ["derive"] }  # Argument parsing (must-have)
anyhow = "1.0"  # Error handling with context (must-have)
tracing = "0.1"  # Structured logging (must-have)
tracing-subscriber = { version = "0.3", features = ["env-filter"] }  # Logging control
colored = "2.1"  # Terminal colors (recommended)

# String/Path/Pattern Optimizations (Phase 2.5)
ignore = "0.4"   # Respects .gitignore (must-have)
globset = "0.4"  # Efficient pattern matching (must-have)
unicase = "2.7"  # Case-insensitive without allocation (recommended)

# Optional performance features
rayon = { version = "1.8", optional = true }        # Parallel processing
aho-corasick = { version = "1.1", optional = true } # Multi-pattern matching
indicatif = { version = "0.17", optional = true }   # Progress bars

# New feature flags
[features]
default = []
sqlite = ["dep:rusqlite", "dep:chrono"]
parallel = ["dep:rayon"]              # Enable parallel directory traversal
fast-patterns = ["dep:aho-corasick"]  # Enable multi-pattern keyword matching
progress = ["dep:indicatif"]          # Enable progress bars
```

### Phase 3

```toml
thiserror = "1.0"   # For better error types (if not using anyhow)

[dev-dependencies]
criterion = "0.5"   # For benchmarking
```

---

## Notes

- All changes should maintain backward compatibility unless version is bumped
- Performance must not regress (verify with benchmarks if added)
- Binary size should stay under 3MB per binary
- Startup time target: <5ms for all binaries

---

## References

- [GitHub Rust Best Practices](https://github.com/github/awesome-copilot/blob/main/instructions/rust.instructions.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

**Last Review:** 2025-10-31
**Next Review:** After Phase 1 completion

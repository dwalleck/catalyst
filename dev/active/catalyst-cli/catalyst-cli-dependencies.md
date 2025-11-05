# Catalyst CLI - Dependencies Specification

**Last Updated:** 2025-01-04
**Status:** Phase 0 - Specifications
**Related:** catalyst-cli-plan.md, catalyst-cli-tasks.md

---

## Overview

This document specifies all dependencies required for the Catalyst CLI implementation across all phases. Dependencies are categorized by purpose and phase, with feature flags clearly documented.

---

## Dependency Categories

### 1. Core Dependencies (Already Present)

These dependencies are already in the workspace and will be used throughout:

| Dependency | Version | Purpose | Phase Used |
|------------|---------|---------|------------|
| `serde` | 1.0 | JSON/TOML serialization | All |
| `serde_json` | 1.0 | Settings.json generation | Phase 2 |
| `toml` | 0.8 | Cargo.toml parsing | Phase 1 |
| `regex` | 1.10 | Pattern matching | Phase 4 |
| `anyhow` | 1.0 | Error context | All |
| `thiserror` | 1.0 | Custom error types | Phase 1 |
| `once_cell` | 1.19 | Lazy statics | Various |
| `walkdir` | 2.4 | Directory traversal | Phase 3 |
| `clap` | 4.5 | CLI parsing | Phase 1 |
| `tracing` | 0.1 | Logging | All |
| `tracing-subscriber` | 0.3 | Log configuration | All |
| `colored` | 2.1 | Terminal colors | Phase 7 |

### 2. NEW Dependencies (To Be Added)

These dependencies must be added to `Cargo.toml` for the CLI implementation:

| Dependency | Version | Purpose | Phase Used | Notes |
|------------|---------|---------|------------|-------|
| `sha2` | 0.10 | SHA256 hashing for skill change detection | Phase 3 | For `.catalyst-hashes.json` |
| `dialoguer` | 0.11 | Interactive prompts | Phase 5 | Multi-select, confirmations |
| `indicatif` | 0.17 | Progress bars | Phase 5 | Skill installation progress |
| `include_dir` | 0.7 | Embed skills at compile time | Phase 3 | Zero-copy file embedding |
| `dirs` | 5.0 | Cross-platform home directory | Phase 1 | `~/.claude-hooks/bin/` detection |
| `dunce` | 1.0 | Canonicalize paths (Windows UNC fix) | Phase 2 | Handles `\\?\` paths |
| `tempfile` | 3.14 | Atomic file writes | Phase 2 | Used in `write_file_atomic()` |

### 3. Platform-Specific Dependencies

| Dependency | Version | Platform | Purpose | Phase |
|------------|---------|----------|---------|-------|
| `dirs` | 5.0 | All | Home directory (`HOME` vs `USERPROFILE`) | Phase 1 |
| `dunce` | 1.0 | Windows | Convert `\\?\` UNC paths to regular paths | Phase 2 |

### 4. Optional Feature Dependencies (Already Present)

| Dependency | Version | Feature Flag | Purpose | Status |
|------------|---------|--------------|---------|--------|
| `rusqlite` | 0.31 | `sqlite` | SQLite database for tracker | Used by existing binary |
| `chrono` | 0.4 | `sqlite` | **⚠️ CRITICAL FIX NEEDED** | See Issue #1 below |
| `rayon` | 1.8 | `parallel` | Parallel file processing | Future optimization |
| `aho-corasick` | 1.1 | `fast-patterns` | Multi-pattern matching | Future optimization |

### 5. Dev Dependencies

| Dependency | Version | Purpose | Phase Used |
|------------|---------|---------|------------|
| `cargo-husky` | 1.0 | Pre-commit hooks | All (auto-installs) |

**Note:** `tempfile` is now a production dependency (used in `write_file_atomic()`), not just dev.

---

## Critical Issues & Fixes

### Issue #1: Chrono Dependency Conflict

**Problem:** `chrono` is currently marked as `optional` (only available with `sqlite` feature), but it's already used in `catalyst-cli/src/types.rs:469` without the feature flag.

**Evidence:**
```rust
// catalyst-cli/src/types.rs:469
pub struct FileChange {
    pub timestamp: chrono::DateTime<chrono::Utc>,  // ❌ Fails without sqlite feature
    // ...
}
```

**Current State:**
```toml
# catalyst-cli/Cargo.toml
chrono = { workspace = true, optional = true }

[features]
sqlite = ["dep:rusqlite", "dep:chrono"]
```

**Fix Required (Phase 1, Task 1.1):**
```toml
# catalyst-cli/Cargo.toml
chrono = { workspace = true }  # ✅ Remove optional = true

[features]
sqlite = ["dep:rusqlite"]  # ✅ Remove dep:chrono
```

**Impact:** Without this fix, building without `--features sqlite` will fail compilation.

---

## Feature Flags

### Current Features

```toml
[features]
default = []
sqlite = ["dep:rusqlite", "dep:chrono"]  # ⚠️ WILL CHANGE to just rusqlite
parallel = ["dep:rayon"]
fast-patterns = ["dep:aho-corasick"]
```

### After Phase 1 Fix

```toml
[features]
default = []
sqlite = ["dep:rusqlite"]  # ✅ chrono always available
parallel = ["dep:rayon"]
fast-patterns = ["dep:aho-corasick"]
```

### Feature Usage by Phase

| Phase | Features Used | Reason |
|-------|---------------|--------|
| Phase 1-7 | None (default build) | Core CLI functionality |
| Phase 8 | All features in CI | Test all variants |
| Production | `sqlite` (optional) | User choice for tracker |

---

## Dependency Addition Plan

### Phase 1 (Task 1.1): Critical Fixes

**Changes to `catalyst-cli/Cargo.toml`:**

```toml
[dependencies]
# ... existing dependencies ...

# ✅ FIX: Remove optional from chrono (already used in types.rs)
chrono = { workspace = true }

# ✅ ADD: New dependencies for CLI implementation
sha2 = "0.10"
dialoguer = "0.11"
indicatif = "0.17"
include_dir = "0.7"
dirs = "5.0"
dunce = "1.0"
tempfile = "3.14"

[features]
default = []
sqlite = ["dep:rusqlite"]  # ✅ Remove dep:chrono
parallel = ["dep:rayon"]
fast-patterns = ["dep:aho-corasick"]
```

**Changes to workspace `Cargo.toml`:**

```toml
[workspace.dependencies]
# ... existing dependencies ...

# ✅ ADD: New dependencies
sha2 = "0.10"
dialoguer = "0.11"
indicatif = "0.17"
include_dir = "0.7"
dirs = "5.0"
dunce = "1.0"
tempfile = "3.14"
```

### Verification Commands

```bash
# 1. Verify all dependencies resolve
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "catalyst-cli") | .dependencies'

# 2. Test build without features (should work after chrono fix)
cargo build --bin catalyst

# 3. Test build with sqlite feature
cargo build --bin catalyst --features sqlite

# 4. Check dependency tree
cargo tree -p catalyst-cli

# 5. Verify no duplicate versions
cargo tree -d
```

---

## Dependency Justifications

### Why `sha2` over alternatives?

- **Standard choice:** De facto standard for SHA256 in Rust
- **Zero-copy:** Efficient for large files
- **Well-maintained:** Part of RustCrypto project
- **Size:** Minimal binary size impact (~50KB)

### Why `dialoguer` over alternatives?

- **Rich features:** Multi-select, confirmations, input validation
- **Cross-platform:** Works on Windows/Unix
- **Well-tested:** 500K+ downloads/month
- **Clean API:** Intuitive builder pattern

### Why `indicatif` over alternatives?

- **Best-in-class:** Industry standard for progress bars
- **Flexible:** Supports spinners, bars, multi-progress
- **Terminal-aware:** Respects CI/non-TTY environments
- **Integrates with dialoguer:** Compatible styles

### Why `include_dir` over `include_bytes!`?

- **Directory support:** Embeds entire directory trees
- **Metadata preserved:** File names, structure, permissions
- **Compile-time:** Zero runtime cost
- **Type-safe:** Compile fails if files missing

### Why `dirs` over manual env vars?

- **Cross-platform:** Handles `HOME` vs `USERPROFILE` vs `XDG_*`
- **Edge cases:** Handles missing home dir gracefully
- **Standard:** Used by cargo, rustup, etc.
- **Tiny:** 20KB binary size impact

### Why `dunce` for Windows?

- **UNC path fix:** Converts `\\?\C:\...` to `C:\...`
- **Windows-specific:** No-op on Unix (zero cost)
- **Essential:** Windows canonicalize() produces UNC paths
- **Tiny:** 10KB binary size impact

---

## Binary Size Impact

### Current Binary Size (settings-manager)

```bash
$ ls -lh target/release/settings-manager
-rwxr-xr-x  1 user  staff   1.8M  catalyst/target/release/settings-manager
```

### Estimated Size After All Dependencies

| Component | Size Impact |
|-----------|-------------|
| Current binary | 1.8 MB |
| Embedded skills (~5 skills × 100KB) | +0.5 MB |
| `sha2` | +0.05 MB |
| `dialoguer` + `indicatif` | +0.2 MB |
| `include_dir` | +0.05 MB |
| `dirs` + `dunce` + `tempfile` | +0.04 MB |
| **Estimated Total** | **~2.6 MB** |

**Conclusion:** Well within acceptable range (<5MB target from plan).

---

## License Compatibility

All dependencies are MIT or MIT/Apache-2.0 dual-licensed, compatible with this project's MIT license:

| Dependency | License | Compatible |
|------------|---------|------------|
| `sha2` | MIT/Apache-2.0 | ✅ |
| `dialoguer` | MIT | ✅ |
| `indicatif` | MIT | ✅ |
| `include_dir` | MIT | ✅ |
| `dirs` | MIT/Apache-2.0 | ✅ |
| `dunce` | MIT/Apache-2.0 | ✅ |
| `tempfile` | MIT/Apache-2.0 | ✅ |

---

## Minimum Rust Version (MSRV)

**Workspace MSRV:** 2021 edition (Rust 1.56+)

**Dependency MSRV Requirements:**

| Dependency | MSRV | Notes |
|------------|------|-------|
| `sha2` 0.10 | 1.56+ | ✅ Compatible |
| `dialoguer` 0.11 | 1.63+ | ⚠️ Requires bump if needed |
| `indicatif` 0.17 | 1.63+ | ⚠️ Requires bump if needed |
| `include_dir` 0.7 | 1.56+ | ✅ Compatible |
| `dirs` 5.0 | 1.56+ | ✅ Compatible |
| `dunce` 1.0 | 1.56+ | ✅ Compatible |
| `tempfile` 3.14 | 1.63+ | ⚠️ Requires bump if needed |

**Recommendation:** Bump workspace MSRV to 1.63+ (Rust 2022 Q4) to support `dialoguer`, `indicatif`, and `tempfile`.

### MSRV Decision Required in Phase 1

**Current State:**
- Workspace specifies edition = "2021" (minimum Rust 1.56+)
- Three new dependencies require Rust 1.63+: dialoguer, indicatif, tempfile

**Decision Point (Phase 1, Task 1.1):**

Option 1: **Bump to 1.63+ (Recommended)**
- ✅ Allows all dependencies
- ✅ Still relatively conservative (Oct 2022 release)
- ✅ Matches modern Rust practices
- ⚠️ May impact users on older systems

Option 2: **Stay at 1.56+**
- ❌ Must find alternatives to dialoguer/indicatif
- ❌ Removes interactive mode (Phase 5)
- ❌ Reduces UX quality

**Recommendation**: Bump to 1.63+ in Phase 1 and document in README.

**CI Requirement**: Add MSRV check to CI:
```yaml
# .github/workflows/ci.yml
jobs:
  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: dtolnay/rust-toolchain@1.63
      - run: cargo check --all-features
```

---

## Completion Checklist

- [x] List all crate dependencies with versions
- [x] Categorize dependencies by phase when needed
- [x] Document feature flags
- [x] Identify dev dependencies
- [x] Add dependencies list to this plan document
- [x] Identify critical issues (chrono conflict)
- [x] Justify each new dependency
- [x] Estimate binary size impact
- [x] Verify license compatibility
- [x] Check MSRV requirements

---

**End of Dependencies Specification**

See `catalyst-cli-plan.md` for strategic context.
See `catalyst-cli-tasks.md` for implementation checklist.

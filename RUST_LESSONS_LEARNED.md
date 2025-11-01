# Rust Lessons Learned

This document captures common Rust mistakes and their solutions discovered during code reviews. Reference this before submitting PRs to avoid these issues.

## Table of Contents
1. [Redundant Single-Component Imports](#redundant-single-component-imports)
2. [Uninitialized Tracing Subscribers](#uninitialized-tracing-subscribers)
3. [Unsafe unwrap() on Path Operations](#unsafe-unwrap-on-path-operations)
4. [Duplicated Logic](#duplicated-logic)

---

## Redundant Single-Component Imports

### Problem
Clippy warns about redundant single-component path imports (`use serde_json;`) when you're using fully qualified paths. If you write `serde_json::json!`, you don't need `use serde_json;` - the crate is already available through `Cargo.toml`.

### Example - file_analyzer.rs (Phase 2.4)
```rust
// ❌ WRONG - Redundant import with fully qualified paths
use serde_json;  // Clippy: this import is redundant

fn print_json_results(stats: &Stats, elapsed: Duration) {
    let json = serde_json::json!({     // Using fully qualified path
        "total_files": stats.total_files,
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());  // Fully qualified
}
```

**Clippy error:** `clippy::single_component_path_imports` - "this import is redundant"

### Solution Options

**Option 1: Use fully qualified paths (no import needed)**
```rust
// ✅ CORRECT - No import, use fully qualified paths
fn print_json_results(stats: &Stats, elapsed: Duration) {
    let json = serde_json::json!({
        "total_files": stats.total_files,
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
```

**Option 2: Import specific items and use unqualified**
```rust
// ✅ ALSO CORRECT - Import specific items
use serde_json::json;

fn print_json_results(stats: &Stats, elapsed: Duration) {
    let json = json!({  // Now unqualified
        "total_files": stats.total_files,
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
```

### Rule
**Use fully qualified paths (no import) OR import specific items (unqualified use). Never use single-component imports like `use serde_json;`**

### Common Cases

```rust
// ❌ WRONG - Redundant imports
use tracing_subscriber;
tracing_subscriber::fmt().init();

use serde_json;
serde_json::json!({"key": "value"})

// ✅ CORRECT - Fully qualified (no import)
tracing_subscriber::fmt().init();
serde_json::json!({"key": "value"})

// ✅ ALSO CORRECT - Import specific items
use tracing_subscriber::{fmt, EnvFilter};
fmt().with_env_filter(EnvFilter::new("info")).init();
```

---

## Uninitialized Tracing Subscribers

### Problem
Using `tracing::debug!`, `info!`, `warn!` etc. without initializing a subscriber means logs won't appear, even with `RUST_LOG=debug`.

### Example - skill_activation_prompt.rs & post_tool_use_tracker_sqlite.rs (Phase 2.4)
```rust
// ❌ WRONG - No subscriber initialization
use tracing::debug;

fn main() -> Result<()> {
    debug!("This will never appear!");  // Silent failure
    // ... rest of code
}
```

**Issue:** Debug logs appear to work in development (other parts of codebase might initialize subscriber) but fail in production/standalone use.

### Solution
```rust
// ✅ CORRECT - Initialize subscriber in main()
use tracing::debug;
use tracing_subscriber;

fn main() -> Result<()> {
    // Initialize tracing subscriber (respects RUST_LOG env var)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    debug!("This will appear with RUST_LOG=debug");
    // ... rest of code
}
```

### Rule
**Every binary that uses tracing MUST initialize a subscriber in `main()`. Libraries should NOT initialize subscribers (let the binary decide).**

### Dependencies Required
```toml
[dependencies]
tracing = { workspace = true }
tracing-subscriber = { workspace = true, features = ["env-filter"] }
```

---

## Unsafe unwrap() on Path Operations

### Problem
`Path::file_name()` returns `Option<&OsStr>` and can be `None` for paths ending with `..` or `/`. Using `.unwrap()` directly can panic.

### Example - file_analyzer.rs (Phase 2.4)
```rust
// ❌ WRONG - Unsafe unwrap
if analysis.has_async && !analysis.has_try_catch {
    let file_name = path.file_name().unwrap().to_string_lossy();
    println!("⚠️  {} - Async without try/catch", file_name);
}
```

**Panic scenarios:**
- Path is root directory `/` or `C:\`
- Path ends with `..` (e.g., `foo/..`)
- Path is empty

### Solution
```rust
// ✅ CORRECT - Defensive with fallback
if analysis.has_async && !analysis.has_try_catch {
    // Safe: We know this is a file from walkdir, so file_name() won't be None
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| path.display().to_string().into());

    println!("⚠️  {} - Async without try/catch", file_name);
}
```

### Alternative Solutions
```rust
// Option 1: Use if-let (safest, handles None gracefully)
if let Some(name) = path.file_name() {
    println!("⚠️  {} - Async without try/catch", name.to_string_lossy());
}

// Option 2: Use match
match path.file_name() {
    Some(name) => println!("⚠️  {} - Async without try/catch", name.to_string_lossy()),
    None => eprintln!("Warning: Could not get filename for {:?}", path),
}

// Option 3: Use unwrap_or_else with fallback (shown above)
let file_name = path
    .file_name()
    .map(|name| name.to_string_lossy())
    .unwrap_or_else(|| path.display().to_string().into());
```

### Rule
**Never use `.unwrap()` on `Path::file_name()`, `Path::parent()`, or `Path::extension()`. Always handle the `Option` with `if-let`, `match`, or `unwrap_or_else`.**

### When unwrap() IS Safe
```rust
// These unwrap() calls are safe:

// 1. Regex compilation with hardcoded patterns (compile-time constant)
static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"foo").unwrap());

// 2. Building known-good structures
let builder = GlobSetBuilder::new();
builder.add(Glob::new("*.rs").unwrap());  // Pattern is a literal

// 3. With documented safety invariant
// SAFETY: Path came from walkdir which only returns files with valid names
let name = path.file_name().unwrap();
```

---

## Duplicated Logic

### Problem
Checking the same condition in multiple places creates maintenance burden and potential bugs if conditions diverge.

### Example - file_analyzer.rs (Phase 2.4)
```rust
// ❌ WRONG - Same logic in two places
fn main() -> Result<()> {
    let args = Args::parse();

    // First check (lines 196-198)
    if args.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    // ... 20 lines later ...

    // Second check (line 217) - DUPLICATE!
    let use_color = !args.no_color && std::env::var("NO_COLOR").is_err();

    if use_color {
        println!("{}", "text".bright_blue());
    }
}
```

**Issues:**
1. Same condition logic appears twice
2. If you update one, must remember to update the other
3. Logical inverse makes it harder to verify they're equivalent

### Solution
```rust
// ✅ CORRECT - Calculate once, use everywhere
fn main() -> Result<()> {
    let args = Args::parse();

    // Calculate color decision ONCE at the start
    let use_color = !args.no_color && std::env::var("NO_COLOR").is_err();

    // Set the global override based on our decision
    if !use_color {
        colored::control::set_override(false);
    }

    // ... rest of code uses `use_color` variable ...

    if use_color {
        println!("{}", "text".bright_blue());
    }
}
```

### Rule
**Calculate conditions once at the start of a function, store in a well-named variable, and reference that variable everywhere. Don't re-calculate the same condition.**

### Benefits
1. Single source of truth
2. Easier to modify behavior
3. More efficient (calculate once vs multiple times)
4. Clearer intent with descriptive variable name

---

## Checklist Before Submitting PR

Use this checklist to catch common issues before code review:

- [ ] All crates used have explicit `use` statements
- [ ] Binaries using `tracing` initialize subscribers in `main()`
- [ ] No `.unwrap()` on `Path` operations (`file_name()`, `parent()`, `extension()`)
- [ ] No duplicated conditional logic
- [ ] Run `cargo clippy -- -D warnings` (treats warnings as errors)
- [ ] Run `cargo fmt --all` (consistent formatting)
- [ ] Build in release mode: `cargo build --release`
- [ ] Test all features: `cargo test --all-features`

---

## Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

**Document Version:** 1.0 (Phase 2.4 PR #6 Review)
**Last Updated:** 2025-10-31
**Maintainer:** Catalyst Project Team

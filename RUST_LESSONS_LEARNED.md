# Rust Lessons Learned

This document captures common Rust mistakes and their solutions discovered during code reviews. Reference this before submitting PRs to avoid these issues.

## Table of Contents
1. [Redundant Single-Component Imports](#redundant-single-component-imports)
2. [Uninitialized Tracing Subscribers](#uninitialized-tracing-subscribers)
3. [Unsafe unwrap() on Path Operations](#unsafe-unwrap-on-path-operations)
4. [When to Use expect() vs unwrap() vs Proper Error Handling](#when-to-use-expect-vs-unwrap-vs-proper-error-handling)
5. [Duplicated Logic](#duplicated-logic)
6. [Performance-Critical Loop Optimizations](#performance-critical-loop-optimizations)
7. [When NOT to Use Zero-Copy Abstractions](#when-not-to-use-zero-copy-abstractions)
8. [Atomic File Writes](#atomic-file-writes)
9. [Parent Directory Creation](#parent-directory-creation)
10. [TTY Detection for Colored Output](#tty-detection-for-colored-output)
11. [File I/O Testing with tempfile](#file-io-testing-with-tempfile)

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

## When to Use expect() vs unwrap() vs Proper Error Handling

### Problem
Knowing when to use `.unwrap()`, `.expect()`, or proper error handling (`?` operator) is crucial for writing maintainable Rust code.

### Guidelines

**Use `.expect("message")` when:**
- You have a clear invariant that should never fail
- You want to document WHY failure is impossible
- Failure indicates a programming error (bug), not a runtime condition

**Use `.unwrap()` when:**
- Prototyping or example code where failure is acceptable
- The operation literally cannot fail (e.g., compiling hardcoded regexes)
- ONLY in test code

**Use proper error handling (`?`) when:**
- In production code where failure is a possibility
- The error should propagate to the caller
- You want to provide context about what failed

### Examples

```rust
// ✅ GOOD: expect() with clear message for invariants
fn process_config() -> Result<()> {
    // This is a hardcoded path that we control
    let config_path = Path::new("/etc/myapp/config.toml");
    let name = config_path.file_name()
        .expect("config_path is a literal with a filename");

    // ... use name ...
    Ok(())
}

// ✅ GOOD: expect() documents why failure is impossible
static VALID_REGEX: Lazy<Regex> = Lazy::new(|| {
    // This pattern is a string literal - if it's invalid, it's a bug
    Regex::new(r"^\d{3}-\d{2}-\d{4}$")
        .expect("SSN regex pattern is valid")
});

// ✅ GOOD: Proper error handling for runtime conditions
fn read_user_file(path: &Path) -> Result<String> {
    // User-provided path might not exist or be readable
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?
}

// ❌ BAD: unwrap() on user input
fn parse_user_input(input: &str) -> u32 {
    input.parse().unwrap()  // Will panic on invalid input!
}

// ✅ GOOD: Return Result for user input
fn parse_user_input(input: &str) -> Result<u32> {
    input.parse()
        .with_context(|| format!("Invalid number: {}", input))
}

// ✅ GOOD: unwrap_or_else() with graceful fallback
fn print_json(data: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            // Even though serialization rarely fails, handle it gracefully
            format!(r#"{{"error": "Failed to serialize: {}"}}"#, e)
        })
    );
}
```

### Decision Tree

```
Is this production code?
├─ No (prototype/example) → unwrap() is acceptable
└─ Yes → Continue...
    │
    ├─ Can this operation fail at runtime?
    │  ├─ Yes (user input, file I/O, network) → Use ? operator
    │  └─ No → Continue...
    │      │
    │      ├─ Is failure a programming bug?
    │      │  ├─ Yes (hardcoded values, invariants) → Use .expect("why")
    │      │  └─ No → Use unwrap_or_else() with fallback
    │      │
    │      └─ Can I provide a sensible default?
    │          ├─ Yes → Use unwrap_or() or unwrap_or_else()
    │          └─ No → Use .expect("why")
```

### Real-World Examples from This Project

```rust
// ✅ GOOD: expect() for compile-time regex (file_analyzer.rs)
static TRY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"try\s*\{|try:|except:")
        .expect("TRY_REGEX pattern is valid")
});

// ✅ GOOD: unwrap_or_else() for path operations (file_analyzer.rs:285)
let file_name = path
    .file_name()
    .map(|name| name.to_string_lossy())
    .unwrap_or_else(|| path.display().to_string().into());

// ✅ GOOD: unwrap_or_else() for JSON serialization (file_analyzer.rs:149)
println!(
    "{}",
    serde_json::to_string_pretty(&json).unwrap_or_else(|e| {
        format!(r#"{{"error": "Failed to serialize JSON: {}"}}"#, e)
    })
);

// ✅ GOOD: ? operator for user input (file_analyzer.rs:211)
if !args.directory.exists() {
    anyhow::bail!("Directory does not exist: {}", args.directory.display());
}
```

### Rule
**In production code: Use `?` for runtime errors, `.expect("why")` for invariants, and `.unwrap_or_else()` for graceful degradation. Never use bare `.unwrap()` except in tests.**

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

## Performance-Critical Loop Optimizations

### Problem
Creating objects inside hot loops (loops that execute many times) can severely degrade performance, even if each individual operation is cheap. This is especially critical for zero-cost abstractions like `UniCase` that are meant to avoid allocations.

### Example - skill_activation_prompt.rs (Phase 2.5 CRITICAL Issue)

```rust
// ❌ WRONG - Creates UniCase wrapper inside the loop for EVERY keyword
let keyword_match = triggers.keywords.iter().any(|kw| {
    let prompt_unicase = UniCase::new(prompt);     // Created 100 times!
    let keyword_unicase = UniCase::new(kw.as_str());

    prompt_unicase.as_ref().contains(keyword_unicase.as_ref())
});
```

**Problem:** With 100 keywords, this creates `UniCase::new(prompt)` **100 times** - completely defeating the zero-allocation optimization!

**Performance Impact:**
- **Before fix**: 100 UniCase wrapper creations per skill activation
- **After fix**: 1 UniCase wrapper creation per skill activation
- **Savings**: 99% reduction in wrapper allocations

### Solution

```rust
// ✅ CORRECT - Create prompt wrapper ONCE outside the loop
let prompt_unicase = UniCase::new(prompt.as_str());  // Created once!

let keyword_match = triggers.keywords.iter().any(|kw| {
    let keyword_unicase = UniCase::new(kw.as_str());  // Only keyword wrapper created per iteration
    prompt_unicase.as_ref().contains(keyword_unicase.as_ref())
});
```

### General Pattern

**Identify what's loop-invariant:**
- If a value doesn't change between loop iterations, compute it ONCE before the loop
- Move ALL invariant computations outside the loop

```rust
// ❌ BAD - Recomputes invariant inside loop
for item in items {
    let config = load_config();  // Same config every time!
    process(item, config);
}

// ✅ GOOD - Compute once, reuse
let config = load_config();
for item in items {
    process(item, &config);
}
```

### Common Loop Anti-Patterns

**1. String operations**
```rust
// ❌ BAD
for keyword in keywords {
    if prompt.to_lowercase().contains(&keyword.to_lowercase()) { }
}

// ✅ GOOD
let prompt_lower = prompt.to_lowercase();
for keyword in keywords {
    if prompt_lower.contains(&keyword.to_lowercase()) { }
}
```

**2. Regex compilation**
```rust
// ❌ BAD - Compiles regex on every iteration
for line in lines {
    let re = Regex::new(r"\d+").unwrap();
    if re.is_match(line) { }
}

// ✅ GOOD - Compile once
let re = Regex::new(r"\d+").unwrap();
for line in lines {
    if re.is_match(line) { }
}

// ✅ BEST - Use lazy static
static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+").unwrap());
for line in lines {
    if RE.is_match(line) { }
}
```

**3. Collection allocations**
```rust
// ❌ BAD - Creates Vec on every iteration
for item in items {
    let mut buffer = Vec::new();
    buffer.push(item);
    process(&buffer);
}

// ✅ GOOD - Reuse buffer
let mut buffer = Vec::new();
for item in items {
    buffer.clear();
    buffer.push(item);
    process(&buffer);
}
```

### How to Spot Loop Inefficiencies

1. **Code Review Checklist:**
   - Look for `new()`, `clone()`, `to_owned()`, `to_string()` inside loops
   - Look for repeated function calls with same arguments
   - Look for collection allocations (`Vec::new()`, `HashMap::new()`)

2. **Profiling:**
   ```bash
   # Use cargo flamegraph to find hot loops
   cargo flamegraph --bin your-binary

   # Use cargo bench for microbenchmarks
   cargo bench
   ```

3. **Think Like the Compiler:**
   - Ask: "Does this value change between iterations?"
   - If NO → Move it outside the loop
   - If YES → Keep it inside, but minimize allocations

### Rule
**CRITICAL: In hot loops (>100 iterations), move ALL loop-invariant computations outside the loop. Profile performance-critical code to verify optimizations.**

### Real-World Impact

**skill_activation_prompt.rs with 100 keywords:**
- **Before fix**: `100 * num_skills` UniCase wrapper creations
- **After fix**: `num_skills` UniCase wrapper creations
- **With 10 skills**: 1000 → 10 wrapper creations (99% reduction)

---

## When NOT to Use Zero-Copy Abstractions

### Problem
Zero-copy abstractions like `UniCase` are designed for **specific use cases** (equality comparison). Using them incorrectly for other operations (like substring matching) can lead to bugs or unexpected behavior.

### Example - skill_activation_prompt.rs (Phase 2.5 CRITICAL Bug)

**❌ WRONG - UniCase doesn't work for substring matching:**
```rust
use unicase::UniCase;

// This may NOT match correctly!
let prompt_unicase = UniCase::new("I need API help");
let keyword_unicase = UniCase::new("api");
prompt_unicase.as_ref().contains(keyword_unicase.as_ref())  // BUG!
```

**Why it's wrong:**
- `UniCase` is designed for **equality comparison** (`==`), NOT substring operations
- `contains()` on `UniCase` may not provide case-insensitive substring matching
- The abstraction gives false confidence about functionality

**✅ CORRECT - Use pre-lowercased strings:**
```rust
// Pre-lowercase keywords once at compile time
struct CompiledTriggers {
    keywords_lower: Vec<String>,  // Pre-lowercased
    intent_regexes: Vec<Regex>,
}

impl CompiledTriggers {
    fn from_triggers(triggers: &PromptTriggers) -> Self {
        let keywords_lower = triggers
            .keywords
            .iter()
            .map(|kw| kw.to_lowercase())
            .collect();

        Self { keywords_lower, intent_regexes }
    }
}

// Lowercase prompt once per activation
let prompt_lower = prompt.to_lowercase();

// Use standard string contains() with pre-lowercased keywords
let keyword_match = triggers.keywords_lower.iter()
    .any(|kw_lower| prompt_lower.contains(kw_lower));
```

### When to Use Each Approach

| Use Case | Recommended Approach | Why |
|----------|---------------------|-----|
| **Equality comparison** | `UniCase` or `to_lowercase()` | `UniCase` avoids allocation for `==` checks |
| **Substring matching** | `to_lowercase()` + `contains()` | Standard string methods work correctly |
| **HashMap keys** | `UniCase` wrapper | Zero-allocation case-insensitive keys |
| **Sorting/ordering** | `UniCase` wrapper | Zero-allocation case-insensitive comparison |
| **Regex matching** | `(?i)` flag or `to_lowercase()` | Regex has built-in case-insensitive support |

### Rule of Thumb

**Read the documentation carefully** for zero-copy/zero-allocation abstractions:
- Understand what operations they support
- Don't assume standard operations (like `contains()`) work the same way
- When in doubt, use standard library methods with explicit lowercasing
- Premature optimization can introduce subtle bugs

### Performance Impact of Correct Approach

**Before fix (broken UniCase approach):**
- Unknown behavior, potential bugs

**After fix (pre-lowercased keywords):**
- One allocation per activation: `prompt.to_lowercase()` (~50-200 bytes)
- Keywords lowercased once at startup, not in hot loop
- Predictable, correct behavior with minimal overhead

### Key Takeaway

**Zero-copy abstractions are powerful but specialized.** Always verify they support your actual use case. In this case:
- ✅ `UniCase` for equality: `if key1 == key2`
- ❌ `UniCase` for substring: `if text.contains(substring)`

---

## Atomic File Writes

### Problem
Writing files directly can result in data corruption if the process crashes or is interrupted mid-write. This leaves the file in a partially-written, invalid state.

### Example - settings.rs (Phase 2.6)

```rust
// ❌ WRONG - Direct write can corrupt file if interrupted
pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(self)?;
    fs::write(path.as_ref(), json)?;  // File can be corrupted!
    Ok(())
}
```

**Failure scenarios:**
- Process killed mid-write
- Disk full during write
- I/O error after partial write
- Power loss during write

**Result:** File contains partial JSON that can't be parsed

### Solution - Atomic Write with Temp File + Rename

```rust
use std::fs;
use std::io::Write;
use anyhow::{Context, Result};

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)
        .context("Failed to serialize settings")?;

    // Write to temporary file first
    let temp_path = path.with_extension("tmp");
    let mut temp_file = fs::File::create(&temp_path)
        .context("Failed to create temporary file")?;

    temp_file.write_all(json.as_bytes())
        .context("Failed to write to temporary file")?;

    // Ensure data is flushed to disk
    temp_file.sync_all()
        .context("Failed to sync temporary file")?;

    // Atomic rename (POSIX guarantees atomicity)
    fs::rename(&temp_path, path)
        .context("Failed to rename temporary file")?;

    Ok(())
}
```

### Why This Works

**Atomic rename guarantees:**
1. If rename succeeds, the new file is complete and valid
2. If rename fails, the old file remains unchanged
3. No intermediate state where file is partially written
4. On POSIX systems (Linux, macOS), rename is atomic even across overwrites

### When to Use

**Use atomic writes for:**
- ✅ Configuration files (settings.json)
- ✅ State files (databases, caches)
- ✅ Any file where corruption would break functionality
- ✅ Files that are read by other processes

**Don't need atomic writes for:**
- ❌ Log files (append-only, partial writes are acceptable)
- ❌ Temporary scratch files
- ❌ Files that are write-once, never-overwritten

### Alternative: Use tempfile Crate

```rust
use tempfile::NamedTempFile;
use std::io::Write;

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)?;

    // Create temp file in same directory (important for atomic rename)
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir)?;

    temp_file.write_all(json.as_bytes())?;
    temp_file.sync_all()?;

    // Atomic persist to final location
    temp_file.persist(path)?;

    Ok(())
}
```

**Benefits of tempfile crate:**
- Automatic cleanup if operation fails
- Handles temp file naming automatically
- Cross-platform temp file location
- Automatic deletion if NamedTempFile is dropped

### Rule
**Always use atomic writes (temp file + rename) for important configuration or state files. Use the `tempfile` crate for convenience and automatic cleanup.**

---

## Parent Directory Creation

### Problem
Writing a file fails if parent directories don't exist, even if the path is valid. This is especially common when creating new configuration files in subdirectories.

### Example - settings.rs (Phase 2.6)

```rust
// ❌ WRONG - Fails if parent directory doesn't exist
pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(self)?;
    fs::write(path.as_ref(), json)?;  // Error: No such file or directory
    Ok(())
}

// Example failure:
settings.write("config/user/settings.json")?;  // Fails if config/user/ doesn't exist
```

**Error message:**
```
Error: No such file or directory (os error 2)
```

### Solution - Create Parent Directories First

```rust
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)
        .context("Failed to serialize settings")?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create parent directories")?;
    }

    // Now write the file
    fs::write(path, json)
        .context("Failed to write settings file")?;

    Ok(())
}
```

### Combined with Atomic Writes

```rust
use tempfile::NamedTempFile;
use std::io::Write;

pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(self)?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create temp file in same directory
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir)?;

    temp_file.write_all(json.as_bytes())?;
    temp_file.sync_all()?;

    // Atomic persist
    temp_file.persist(path)?;

    Ok(())
}
```

### Why create_dir_all() is Safe

**`fs::create_dir_all()` is idempotent:**
- If directory exists, does nothing (no error)
- If parent directories exist, creates only missing ones
- Creates entire path in one call
- Returns success if directory already exists

```rust
// All of these succeed, even if directories exist:
fs::create_dir_all("/existing/path")?;      // OK
fs::create_dir_all("/new/nested/path")?;    // Creates all levels
fs::create_dir_all(".")?;                   // OK (current dir exists)
```

### Rule
**Always call `fs::create_dir_all()` on the parent directory before writing files. It's safe, fast, and prevents "No such file or directory" errors.**

### Checklist for File Writes

```rust
pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    // 1. Serialize data
    let json = serde_json::to_string_pretty(self)?;

    // 2. Create parent directories
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 3. Atomic write
    let mut temp_file = NamedTempFile::new_in(
        path.parent().unwrap_or_else(|| Path::new("."))
    )?;
    temp_file.write_all(json.as_bytes())?;
    temp_file.sync_all()?;
    temp_file.persist(path)?;

    Ok(())
}
```

---

## TTY Detection for Colored Output

### Problem
Sending ANSI color codes to non-terminal outputs (pipes, files, CI logs) creates unreadable garbage characters and pollutes logs.

### Example - settings_manager.rs (Phase 2.6)

```rust
// ❌ WRONG - Always uses color codes based on NO_COLOR env var only
fn main() -> Result<()> {
    let use_color = env::var("NO_COLOR").is_err();

    if use_color {
        println!("{}", "✅ Success".green());  // Garbage in CI logs!
    }
}
```

**Problem scenarios:**
```bash
# Piped to file - color codes in file
settings-manager read settings.json > output.txt  # File contains \x1b[32m codes

# Piped to grep - can't match colored text
settings-manager validate settings.json | grep "Success"  # May not match

# CI logs - unreadable
# [32m✅ Success[0m  ← Garbage in GitHub Actions logs
```

### Solution - Check if stdout is a Terminal

```rust
use std::io::{self, IsTerminal};

fn main() -> Result<()> {
    // Check both NO_COLOR and whether stdout is a terminal
    let use_color = env::var("NO_COLOR").is_err() && io::stdout().is_terminal();

    if use_color {
        println!("{}", "✅ Success".green());
    } else {
        println!("✅ Success");
    }

    Ok(())
}
```

### TTY Detection Methods

**Stable Rust (1.70+):**
```rust
use std::io::{self, IsTerminal};

// Check stdout
let is_tty = io::stdout().is_terminal();

// Check stderr (for error messages)
let is_tty = io::stderr().is_terminal();
```

**With `atty` crate (older Rust):**
```rust
use atty::Stream;

let is_tty = atty::is(Stream::Stdout);
```

### Complete Color Detection Pattern

```rust
use std::env;
use std::io::{self, IsTerminal};

fn should_use_color() -> bool {
    // Respect NO_COLOR environment variable (standard)
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Respect FORCE_COLOR (for testing)
    if env::var("FORCE_COLOR").is_ok() {
        return true;
    }

    // Only use color if stdout is a terminal
    io::stdout().is_terminal()
}

fn main() -> Result<()> {
    let use_color = should_use_color();

    // Use color decision consistently
    if use_color {
        println!("{}", "Success".green());
    } else {
        println!("Success");
    }

    Ok(())
}
```

### Integration with `colored` Crate

```rust
use colored::*;

fn main() -> Result<()> {
    // Set global override at startup
    if !should_use_color() {
        colored::control::set_override(false);
    }

    // Now all colored output respects the setting
    println!("{}", "This respects TTY detection".green());

    Ok(())
}
```

### When to Check TTY

**Check stdout TTY for:**
- ✅ Regular output (results, status messages)
- ✅ JSON output (some tools colorize JSON)
- ✅ Table formatting

**Check stderr TTY for:**
- ✅ Error messages
- ✅ Warning messages
- ✅ Progress indicators

**Both might be different:**
```bash
# stdout piped, stderr to terminal
program 2> errors.log | less

# stdout to terminal, stderr piped
program > output.txt
```

### Rule
**Always check if stdout is a terminal (`io::stdout().is_terminal()`) in addition to checking `NO_COLOR`. This prevents ANSI codes from polluting pipes, files, and CI logs.**

### Testing

```bash
# Should NOT have color codes:
settings-manager read settings.json > output.txt
cat output.txt  # Should be plain text

# Should have color codes:
settings-manager read settings.json  # To terminal

# Should respect NO_COLOR:
NO_COLOR=1 settings-manager read settings.json  # No colors
```

---

## File I/O Testing with tempfile

### Problem
Testing file I/O operations without integration tests leaves file handling bugs undetected. Unit tests alone can't catch issues like:
- Files not written correctly
- Race conditions in file access
- Permission errors
- Parent directory creation failures

### Example - Missing Tests for settings.rs (Phase 2.6)

```rust
// ❌ WRONG - Only unit tests, no actual file I/O
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_roundtrip() {
        let settings = ClaudeSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: ClaudeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }

    // No tests for:
    // - Actually reading from files
    // - Actually writing to files
    // - Error handling when file doesn't exist
    // - Error handling when parent directory doesn't exist
}
```

### Solution - Integration Tests with tempfile

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs;

    #[test]
    fn test_write_and_read_roundtrip() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        // Create settings
        let mut settings = ClaudeSettings::default();
        settings.enable_all_project_mcp_servers = true;
        settings.add_hook("UserPromptSubmit", HookConfig {
            matcher: None,
            hooks: vec![Hook {
                r#type: "command".to_string(),
                command: "test.sh".to_string(),
            }],
        });

        // Write to file
        settings.write(&settings_path).unwrap();

        // Verify file exists
        assert!(settings_path.exists());

        // Read back from file
        let loaded = ClaudeSettings::read(&settings_path).unwrap();

        // Verify contents match
        assert_eq!(settings, loaded);
        assert_eq!(loaded.hooks.len(), 1);
    }

    #[test]
    fn test_write_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Path with nested non-existent directories
        let settings_path = temp_dir.path()
            .join("config")
            .join("user")
            .join("settings.json");

        let settings = ClaudeSettings::default();

        // Should create parent directories automatically
        settings.write(&settings_path).unwrap();

        assert!(settings_path.exists());
        assert!(settings_path.parent().unwrap().exists());
    }

    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("nonexistent.json");

        // Should return error, not panic
        let result = ClaudeSettings::read(&settings_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON
        fs::write(&settings_path, "{ not valid json }").unwrap();

        // Should return parse error
        let result = ClaudeSettings::read(&settings_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_overwrite_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        // Write first settings
        let mut settings1 = ClaudeSettings::default();
        settings1.enable_all_project_mcp_servers = true;
        settings1.write(&settings_path).unwrap();

        // Overwrite with different settings
        let mut settings2 = ClaudeSettings::default();
        settings2.enabled_mcpjson_servers.push("mysql".to_string());
        settings2.write(&settings_path).unwrap();

        // Verify new settings
        let loaded = ClaudeSettings::read(&settings_path).unwrap();
        assert_eq!(loaded.enabled_mcpjson_servers.len(), 1);
        assert!(!loaded.enable_all_project_mcp_servers);
    }
}
```

### Using tempfile Crate

**Add to Cargo.toml:**
```toml
[dev-dependencies]
tempfile = "3.8"
```

**Key tempfile types:**

```rust
use tempfile::{TempDir, NamedTempFile};

// Temporary directory (deleted when dropped)
let temp_dir = TempDir::new()?;
let path = temp_dir.path().join("file.txt");

// Temporary file (deleted when dropped)
let temp_file = NamedTempFile::new()?;
let path = temp_file.path();

// Keep temp file after test
let (file, path) = temp_file.keep()?;
```

### Testing CLI Commands

```rust
use std::process::Command;

#[test]
fn test_cli_validate_command() {
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.json");

    // Create valid settings file
    let settings = ClaudeSettings::default();
    settings.write(&settings_path).unwrap();

    // Run CLI command
    let output = Command::new("./target/debug/settings-manager")
        .arg("validate")
        .arg(settings_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid"));
}
```

### Rule
**Always add integration tests using `tempfile` for code that reads or writes files. Unit tests alone don't catch file I/O bugs.**

### Test Coverage Checklist

For file I/O operations, test:
- [ ] Round-trip write + read produces identical data
- [ ] Writing to non-existent directories creates parents
- [ ] Reading non-existent file returns error (doesn't panic)
- [ ] Reading invalid file format returns error
- [ ] Overwriting existing file works correctly
- [ ] File permissions are correct (if applicable)
- [ ] Atomic write behavior (if implemented)

---

## Checklist Before Submitting PR

Use this checklist to catch common issues before code review:

**Code Quality:**
- [ ] All crates used have explicit `use` statements
- [ ] Binaries using `tracing` initialize subscribers in `main()`
- [ ] No `.unwrap()` on `Path` operations (`file_name()`, `parent()`, `extension()`)
- [ ] No duplicated conditional logic
- [ ] Loop-invariant computations moved outside loops

**File I/O:**
- [ ] File writes use atomic write pattern (temp file + rename)
- [ ] File writes call `fs::create_dir_all()` on parent directory
- [ ] File I/O has integration tests using `tempfile` crate

**CLI/UX:**
- [ ] Colored output checks both `NO_COLOR` AND `io::stdout().is_terminal()`
- [ ] Error messages are helpful and actionable

**Testing & CI:**
- [ ] Run `cargo clippy -- -D warnings` (treats warnings as errors)
- [ ] Run `cargo fmt --all` (consistent formatting)
- [ ] Run `cargo test --all-features` (includes doctests)
- [ ] Build in release mode: `cargo build --release`
- [ ] Test with piped output: `program | cat` (should not have color codes)

---

## Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

**Document Version:** 1.2 (Phase 2.6 PR #8 Review)
**Last Updated:** 2025-11-01
**Maintainer:** Catalyst Project Team

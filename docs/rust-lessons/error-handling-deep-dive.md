# Error Handling Deep Dive

This guide covers Rust error handling patterns, from basic Option handling to production-ready error propagation. These lessons were discovered during code reviews across 6 months of development.

## What This Guide Covers

1. **[Understanding Option Types](#1-understanding-option-types)** - Universal patterns for handling Option<T>
2. **[Common Footgun: Path Operations](#2-common-footgun-path-operations)** - Why Path methods are a frequent source of bugs
3. **[expect() vs unwrap() vs ? Decision Guide](#3-expect-vs-unwrap-vs--decision-guide)** - When to use each error handling approach

**Quick Reference:** See [quick-reference.md](quick-reference.md) for scannable checklists

---

## 1. Understanding Option Types

### The Core Concept

The `Option<T>` type represents a value that may or may not exist. Calling `.unwrap()` on an Option assumes it will always be `Some(value)`, but if it's actually `None`, the program will panic.

**This applies to ALL functions returning Option, not just specific cases.**

### Common Functions Returning Option

```rust
// Collections
vec.get(index)              // Option<&T>
vec.first()                 // Option<&T>
vec.last()                  // Option<&T>
map.get(key)                // Option<&V>

// Paths
path.file_name()            // Option<&OsStr>
path.parent()               // Option<&Path>
path.extension()            // Option<&OsStr>

// Strings
str.chars().next()          // Option<char>
str.split_once(':')         // Option<(&str, &str)>

// Parsing
env::var("KEY").ok()        // Option<String>
```

### Idiomatic Option Handling Patterns

#### Pattern 1: Handle Both Cases with if-let

Use when you need to do something different for the None case.

```rust
// ✅ GOOD - Handle Some and None cases
if let Some(value) = map.get("key") {
    println!("Found: {}", value);
} else {
    println!("Not found");
}
```

#### Pattern 2: Handle Both Cases with match

Use when you need explicit handling of both variants.

```rust
// ✅ GOOD - Explicit handling of both variants
match vec.get(index) {
    Some(value) => process(value),
    None => println!("Index out of bounds"),
}
```

#### Pattern 3: Provide a Default with unwrap_or

Use when you have a sensible default value.

```rust
// ✅ GOOD - Graceful fallback
let count = map.get("count")
    .unwrap_or(&0);

// ✅ GOOD - Computed default
let name = path.file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("unknown");
```

#### Pattern 4: Transform with map

Use when you want to apply a transformation only to Some values.

```rust
// ✅ GOOD - Apply transformation to Some values
let len = name.map(|n| n.len());  // Option<usize>

// ✅ GOOD - Chain transformations
let upper = env::var("NAME").ok()
    .map(|s| s.to_uppercase());
```

#### Pattern 5: Convert to Result and Propagate

Use when None represents an error that should propagate.

```rust
// ✅ GOOD - Use ? operator to propagate None as error
fn get_config_value(key: &str) -> Result<String> {
    let value = map.get(key)
        .ok_or_else(|| anyhow!("Missing config: {}", key))?;
    Ok(value.clone())
}
```

#### Pattern 6: Use expect() for Documented Invariants

Use when None indicates a programming bug (see [Section 3](#3-expect-vs-unwrap-vs--decision-guide) for full guidance).

```rust
// ✅ GOOD - Documented reason why None is impossible
let first = vec.first()
    .expect("vector is never empty due to initialization");

// ✅ GOOD - Programming error if None
let name = Path::new("/etc/config.toml").file_name()
    .expect("hardcoded path has filename");
```

### When to Use Each Pattern

| Pattern | Use When | Example |
|---------|----------|---------|
| **if-let** | Need to handle None case differently | `if let Some(x) = opt { use(x) }` |
| **match** | Need explicit handling of both cases | `match opt { Some(x) => ..., None => ... }` |
| **unwrap_or** | Have a sensible default value | `opt.unwrap_or(0)` |
| **unwrap_or_else** | Default requires computation | `opt.unwrap_or_else(\|\| expensive())` |
| **map** | Transform Some values, keep None | `opt.map(\|x\| x * 2)` |
| **and_then** | Chain operations that return Option | `opt.and_then(\|x\| parse(x))` |
| **ok_or** | Convert to Result for ? operator | `opt.ok_or(err)?` |
| **expect** | None indicates programming bug | `opt.expect("why None impossible")` |

### The Golden Rule

**Always handle `Option<T>` explicitly in production code:**

- ✅ Use `if-let`, `match`, `unwrap_or`, or `unwrap_or_else` for normal operation
- ✅ Use `.expect("reason")` only when None indicates a programming error
- ✅ Use `.ok_or()` with `?` to propagate None as an error
- ❌ Avoid bare `.unwrap()` except in tests, examples, or prototypes

### Examples from Real Code

```rust
// ❌ BAD - Assumes Option is always Some
let value = map.get("key").unwrap();  // Panics if key doesn't exist

// ✅ GOOD - Handle missing key gracefully
let value = map.get("key")
    .ok_or_else(|| anyhow!("Missing required key"))?;

// ❌ BAD - Bare unwrap
let first = vec.first().unwrap();

// ✅ GOOD - Provide context
let first = vec.first()
    .expect("vec is guaranteed non-empty by validation");

// ❌ BAD - Ignores None case
let name = path.file_name().unwrap().to_string_lossy();

// ✅ GOOD - Handle None with fallback
let name = path.file_name()
    .map(|n| n.to_string_lossy())
    .unwrap_or_else(|| path.display().to_string().into());
```

**[↑ Back to Quick Reference](quick-reference.md#3-handling-option-types-safely)**

---

## 2. Common Footgun: Path Operations

### Why This Deserves Special Attention

Path operations are a **common source of unwrap() bugs** because developers incorrectly assume paths always have a filename, parent, or extension. In reality, these methods return `Option<T>` because there are valid cases where they're `None`.

This is just a specific application of the general Option handling patterns covered above - but it's worth highlighting because it's such a frequent mistake.

### Why Path Methods Return Option

```rust
// file_name() returns None for:
Path::new("/")                  // Root directory - no filename
Path::new("foo/..")             // Parent reference - no filename
Path::new("")                   // Empty path

// parent() returns None for:
Path::new("/")                  // Root has no parent
Path::new("")                   // Empty path has no parent

// extension() returns None for:
Path::new("Makefile")           // No extension
Path::new(".gitignore")         // Dotfile with no extension (debatable)
Path::new("archive.tar.gz")     // Returns Some("gz"), not "tar.gz"
```

### The Mistake (Phase 2.4 Example)

```rust
// ❌ WRONG - Assumes file_name() always returns Some
if analysis.has_async && !analysis.has_try_catch {
    let file_name = path.file_name().unwrap().to_string_lossy();
    println!("⚠️  {} - Async without try/catch", file_name);
}
```

**This will panic if:**
- Path is root directory `/` or `C:\`
- Path ends with `..` (e.g., `foo/..`)
- Path is empty

### The Solution - Apply General Option Patterns

All the patterns from [Section 1](#1-understanding-option-types) apply. Here are the most common for Path operations:

```rust
// ✅ Pattern 1: Defensive with fallback
let file_name = path
    .file_name()
    .map(|name| name.to_string_lossy())
    .unwrap_or_else(|| path.display().to_string().into());

// ✅ Pattern 2: Use if-let to skip None cases
if let Some(name) = path.file_name() {
    println!("⚠️  {} - Async without try/catch", name.to_string_lossy());
}

// ✅ Pattern 3: Use expect() with documented invariant
// Safe: We know this is a file from walkdir, so file_name() won't be None
let file_name = path.file_name()
    .expect("walkdir only returns files with valid names");

// ✅ Pattern 4: Convert to Result and propagate
let file_name = path.file_name()
    .ok_or_else(|| anyhow!("Path has no filename: {}", path.display()))?;
```

### Why This is Such a Common Mistake

**Incorrect mental model:**
- ❌ "I'm only working with files, so `file_name()` always works"
- ❌ "I just created this path, it must have a parent"
- ❌ "All files have extensions"

**Correct mental model:**
- ✅ Path methods return Option because edge cases exist
- ✅ Even "obvious" cases can fail (root paths, empty paths)
- ✅ Use the same Option patterns as for any other Option type

### Path Methods That Return Option

- `path.file_name()` → `Option<&OsStr>`
- `path.parent()` → `Option<&Path>`
- `path.extension()` → `Option<&OsStr>`
- `path.file_stem()` → `Option<&OsStr>`

**[↑ Back to Quick Reference](quick-reference.md#4-common-footgun-path-operations)**

---

## 3. expect() vs unwrap() vs ? Decision Guide

### The Question

Knowing **when** to use `.unwrap()`, `.expect()`, or proper error handling (`?` operator) is crucial for writing maintainable Rust code.

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

### The Golden Rule

**In production code: Use `?` for runtime errors, `.expect("why")` for invariants, and `.unwrap_or_else()` for graceful degradation. Never use bare `.unwrap()` except in tests.**

**[↑ Back to Quick Reference](quick-reference.md#5-expect-vs-unwrap-vs-proper-error-handling)**

---

## Related Topics

### Error Handling Related
- **[Result vs Option](quick-reference.md)** - When to use Result<T> vs Option<T>
- **[Context and anyhow](quick-reference.md)** - Providing error context with anyhow crate

### Common Footguns
- **[Path Operations](quick-reference.md#4-common-footgun-path-operations)** - Path-specific Option gotchas
- **[TOCTOU Races](common-footguns.md#toctou-races)** - Time-of-check-time-of-use issues with file I/O

### Type Safety
- **[Validation Patterns](type-safety-deep-dive.md)** - Moving from strings to type-safe code
- **[Immediate Validation](type-safety-deep-dive.md#immediate-validation)** - Validating inputs early

---

**[← Back to Index](index.md)** | **[Quick Reference →](quick-reference.md)**

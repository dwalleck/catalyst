# Rust Lessons Learned - Quick Reference Checklist

**Purpose:** Scannable checklist of all Rust best practices and common mistakes.

**How to use:**

- Scan rules during code review
- Check your code against each pattern
- Click deep-dive links for detailed examples
- Each lesson: Rule + Quick check + Example + Full guide link

[◀ Back to Index](index.md) | **Total:** 21 lessons

---

## 1. Redundant Single-Component Imports

**Rule:** ✅ Use fully qualified paths OR import specific items | ❌ Never `use crate_name;` alone

**Quick Check:**

- Do you write `crate::function()`? Don't add `use crate;`
- Using `serde_json::json!()`? Don't add `use serde_json;`
- Clippy warning `clippy::single_component_path_imports`?

**Common Pattern:**

```rust
// ❌ BAD
use serde_json;
serde_json::json!({"key": "value"})

// ✅ GOOD - Option 1: Fully qualified
serde_json::json!({"key": "value"})

// ✅ GOOD - Option 2: Import specific items
use serde_json::json;
json!({"key": "value"})
```

📖 **[Full Guide: Fundamentals →](fundamentals-deep-dive.md#redundant-imports)**

---

## 2. Uninitialized Tracing Subscribers

**Rule:** ✅ Every binary using `tracing` MUST initialize a subscriber in `main()` | ❌ Don't assume it's initialized

**Quick Check:**

- Using `tracing::debug!`, `info!`, `warn!`?
- Is this a binary (not library)?
- Does `main()` call `tracing_subscriber::fmt().init()`?

**Common Pattern:**

```rust
// ✅ CORRECT
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Now logs will appear
}
```

📖 **[Full Guide: Fundamentals →](fundamentals-deep-dive.md#tracing-subscribers)**

---

## 3. Handling Option Types Safely

**Rule:** ✅ Use `if-let`, `match`, `unwrap_or`, `expect("why")` | ❌ Avoid bare `.unwrap()` except in tests

**Quick Check:**

- Does function return `Option<T>`? Handle explicitly
- `vec.get()`, `map.get()`, `path.file_name()` all return Option
- Can you provide a sensible default or handle None?

**Common Patterns:**

```rust
// ✅ Pattern 1: Handle both cases
if let Some(value) = map.get("key") {
    use(value);
}

// ✅ Pattern 2: Provide default
let count = map.get("count").unwrap_or(&0);

// ✅ Pattern 3: Propagate as error
let value = map.get("key")
    .ok_or_else(|| anyhow!("Missing key"))?;

// ✅ Pattern 4: Document invariant
let value = map.get("key")
    .expect("key always present after initialization");
```

📖 **[Full Guide: Error Handling →](error-handling-deep-dive.md#option-types)**

---

## 4. Path Operations Return Options

**Rule:** ✅ Apply general Option patterns to Path operations | ❌ Don't assume paths always have file_name/parent/extension

**Quick Check:**

- Using `path.file_name()`, `.parent()`, `.extension()`?
- Could path be `/`, `..`, or empty?
- Path from user input or external source?

**Common Patterns:**

```rust
// ✅ Pattern 1: Fallback
let name = path.file_name()
    .map(|n| n.to_string_lossy())
    .unwrap_or_else(|| path.display().to_string().into());

// ✅ Pattern 2: Skip if None
if let Some(name) = path.file_name() {
    process(name);
}

// ✅ Pattern 3: Document invariant
let name = path.file_name()
    .expect("walkdir only returns files with valid names");
```

📖 **[Full Guide: Common Footguns →](common-footguns.md#path-operations)**

---

## 5. expect() vs unwrap() vs Proper Error Handling

**Rule:** ✅ Use `?` for runtime errors, `.expect("why")` for invariants, `.unwrap_or` for defaults | ❌ Never bare `.unwrap()` in production

**Quick Check:**

- Can this operation fail at runtime? Use `?`
- Is failure a programming bug? Use `.expect("reason")`
- Is this test code? `.unwrap()` is OK
- Can you provide a default? Use `.unwrap_or()`

**Decision Tree:**

```rust
// ✅ Runtime errors → ? operator
fs::read_to_string(path)?

// ✅ Programming invariants → expect()
Regex::new(r"hardcoded").expect("pattern is valid")

// ✅ Graceful defaults → unwrap_or_else()
serde_json::to_string_pretty(&data).unwrap_or_else(|e| {
    format!(r#"{{"error": "{}"}}"#, e)
})
```

📖 **[Full Guide: Error Handling →](error-handling-deep-dive.md#expect-vs-unwrap)**

---

## 6. Duplicated Logic

**Rule:** ✅ Calculate conditions once, store in variable, reference everywhere | ❌ Don't re-calculate same condition

**Quick Check:**

- Do you check the same condition in multiple places?
- Same logic with slightly different expressions?
- Could one variable replace multiple checks?

**Common Pattern:**

```rust
// ❌ BAD
if args.no_color || env::var("NO_COLOR").is_ok() {
    colored::control::set_override(false);
}
// ... later ...
let use_color = !args.no_color && env::var("NO_COLOR").is_err();

// ✅ GOOD
let use_color = !args.no_color && env::var("NO_COLOR").is_err();
if !use_color {
    colored::control::set_override(false);
}
// ... use use_color everywhere ...
```

📖 **[Full Guide: Fundamentals →](fundamentals-deep-dive.md#duplicated-logic)**

---

## 7. Performance-Critical Loop Optimizations

**Rule:** ✅ Move ALL loop-invariant computations outside loops (>100 iterations) | ❌ Don't create/allocate inside hot loops

**Quick Check:**

- Does value change between iterations? NO → Move outside
- Creating objects inside loop? (`new()`, `clone()`, `to_string()`)
- Calling same function repeatedly with same args?

**Common Pattern:**

```rust
// ❌ BAD
for item in items {
    let config = load_config();  // Same every time!
    process(item, config);
}

// ✅ GOOD
let config = load_config();  // Once before loop
for item in items {
    process(item, &config);
}
```

📖 **[Full Guide: Performance →](performance-deep-dive.md#loop-optimizations)**

---

## 8. When NOT to Use Zero-Copy Abstractions

**Rule:** ✅ Use zero-copy for intended operations (equality) | ❌ Don't assume they work for all operations (substring matching)

**Quick Check:**

- Using `UniCase` for substring matching? Won't work correctly
- Read the crate docs for supported operations
- When in doubt, use standard library with explicit lowercasing

**Common Pattern:**

```rust
// ❌ WRONG - UniCase for substring
let text = UniCase::new("Hello World");
let keyword = UniCase::new("hello");
text.as_ref().contains(keyword.as_ref())  // May not work!

// ✅ CORRECT - Use to_lowercase()
let text_lower = "Hello World".to_lowercase();
let keyword_lower = "hello".to_lowercase();
text_lower.contains(&keyword_lower)
```

📖 **[Full Guide: Performance →](performance-deep-dive.md#zero-copy)**

---

## 9. Atomic File Writes

**Rule:** ✅ Use temp file + rename for important files | ❌ Don't write directly (can corrupt on interruption)

**Quick Check:**

- Writing config, state, or critical data files?
- Could corruption break functionality?
- Process might be killed mid-write?

**Common Pattern:**

```rust
// ✅ GOOD - Atomic write
use tempfile::NamedTempFile;

let mut temp = NamedTempFile::new_in(dir)?;
temp.write_all(data)?;
temp.sync_all()?;
temp.persist(final_path)?;  // Atomic rename
```

📖 **[Full Guide: File I/O →](file-io-deep-dive.md#atomic-writes)**

---

## 10. Parent Directory Creation

**Rule:** ✅ Always call `fs::create_dir_all()` on parent before writing files | ❌ Don't assume directories exist

**Quick Check:**

- Writing to nested path? (`config/user/settings.json`)
- Creating file in subdirectory?
- Error: "No such file or directory"?

**Common Pattern:**

```rust
// ✅ CORRECT
if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;  // Idempotent, safe
}
fs::write(path, data)?;
```

📖 **[Full Guide: File I/O →](file-io-deep-dive.md#parent-directories)**

---

## 11. TTY Detection for Colored Output

**Rule:** ✅ Check both `NO_COLOR` AND `io::stdout().is_terminal()` | ❌ Don't send ANSI codes to pipes/files

**Quick Check:**

- Using `colored` crate or ANSI codes?
- Output might be piped? (`program | less`, `program > file`)
- CI logs showing garbage characters?

**Common Pattern:**

```rust
// ✅ CORRECT
use std::io::{self, IsTerminal};

let use_color = env::var("NO_COLOR").is_err()
    && io::stdout().is_terminal();

if use_color {
    println!("{}", "Success".green());
} else {
    println!("Success");
}
```

📖 **[Full Guide: Fundamentals →](fundamentals-deep-dive.md#tty-detection)**

---

## 12. File I/O Testing with tempfile

**Rule:** ✅ Always add integration tests using `tempfile` for file operations | ❌ Unit tests alone don't catch file I/O bugs

**Quick Check:**

- Code reads or writes files?
- Tests only check serialization, not actual I/O?
- Testing parent directory creation, overwrites, errors?

**Common Pattern:**

```rust
#[test]
fn test_write_and_read_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.json");

    // Write
    data.write(&path).unwrap();

    // Verify
    assert!(path.exists());
    let loaded = Data::read(&path).unwrap();
    assert_eq!(data, loaded);
}
```

📖 **[Full Guide: File I/O →](file-io-deep-dive.md#testing)**

---

## 13. Using Constants for Validation

**Rule:** ✅ Use constants for semi-dynamic values, enums for fixed sets | ❌ Never use magic strings for validation

**Quick Check:**

- Validating against fixed list of strings?
- Typos possible in string comparisons?
- Want IDE autocomplete?

**Common Pattern:**

```rust
// ✅ GOOD - Constants
pub const EVENT_USER_PROMPT: &str = "UserPromptSubmit";
pub const VALID_EVENTS: &[&str] = &[EVENT_USER_PROMPT, ...];

if !VALID_EVENTS.contains(&event) {
    bail!("Invalid event. Valid: {}", VALID_EVENTS.join(", "));
}

// ✅ BETTER - Enums (see lesson 19)
pub enum HookEvent { UserPromptSubmit, ... }
```

📖 **[Full Guide: Type Safety →](type-safety-deep-dive.md#constants)**

---

## 14. CLI User Feedback for File Operations

**Rule:** ✅ Tell users what happened, where, and whether it succeeded | ❌ Don't perform silent file operations

**Quick Check:**

- Creating, modifying, or deleting files?
- User needs to know the outcome?
- Distinguishing between create vs modify?

**Common Pattern:**

```rust
// ✅ GOOD
let file_existed = path.exists();

// ... perform operation ...

if file_existed {
    println!("✅ Updated existing file: {}", path.display());
} else {
    println!("✅ Created new file: {}", path.display());
}
println!("   Size: {} bytes", metadata.len());
```

📖 **[Full Guide: Fundamentals →](fundamentals-deep-dive.md#cli-feedback)**

---

## 15. Using NamedTempFile for Automatic Cleanup

**Rule:** ✅ Use `tempfile::NamedTempFile` for atomic writes | ❌ Manual temp file handling leaves garbage on failure

**Quick Check:**

- Need atomic file write?
- Manual `.tmp` files left on disk after errors?
- Want automatic cleanup?

**Common Pattern:**

```rust
// ✅ CORRECT - Auto cleanup
use tempfile::NamedTempFile;

let dir = path.parent().unwrap_or(Path::new("."));
let mut temp = NamedTempFile::new_in(dir)?;

temp.write_all(data)?;
temp.sync_all()?;
temp.persist(path)?;  // Atomic + auto-cleanup on error
```

📖 **[Full Guide: File I/O →](file-io-deep-dive.md#namedtempfile)**

---

## 16. Immediate Validation in Setter Methods

**Rule:** ✅ Validate immediately in setters, return `Result<()>` | ❌ Don't defer validation to separate method

**Quick Check:**

- Setter can receive invalid data?
- Want errors at source, not later?
- Building object with multiple steps?

**Common Pattern:**

```rust
// ✅ GOOD - Validate immediately
pub fn add_hook(&mut self, event: &str, config: HookConfig) -> Result<()> {
    if !VALID_EVENTS.contains(&event) {
        bail!("Invalid event '{}'", event);
    }
    if config.hooks.is_empty() {
        bail!("Empty hooks array");
    }

    // Only add if validation passes
    self.hooks.entry(event.to_string()).or_default().push(config);
    Ok(())
}
```

📖 **[Full Guide: Type Safety →](type-safety-deep-dive.md#immediate-validation)**

---

## 17. Avoiding Borrow Checker Issues with HashSet

**Rule:** ✅ Use `.cloned()` or `.copied()` when creating HashSet from borrowed data you need to mutate | ❌ Don't collect references while mutating

**Quick Check:**

- Creating HashSet with `.iter().collect()`?
- Then trying to mutate original collection?
- Error: "cannot borrow as mutable because also borrowed as immutable"?

**Common Pattern:**

```rust
// ❌ BAD
let existing: HashSet<&String> = self.vec.iter().collect();
for item in other.vec {
    if !existing.contains(&item) {
        self.vec.push(item);  // ERROR: can't mutate!
    }
}

// ✅ GOOD
let existing: HashSet<String> = self.vec.iter().cloned().collect();
for item in other.vec {
    if !existing.contains(&item) {
        self.vec.push(item);  // OK!
    }
}
```

📖 **[Full Guide: Common Footguns →](common-footguns.md#borrow-checker)**

---

## 18. Fixing TOCTOU Races

**Rule:** ✅ Check state via operation Result, not separate filesystem check | ❌ Never check `path.exists()` then use it

**Quick Check:**

- Checking if file exists separately from opening it?
- Time gap between check and use?
- Could file state change between check and operation?

**Common Pattern:**

```rust
// ❌ BAD - Race condition
if path.exists() {  // File could be deleted here!
    let data = fs::read(path)?;
}

// ✅ GOOD - Check via operation
match fs::read(path) {
    Ok(data) => { /* file existed */ },
    Err(e) if e.kind() == io::ErrorKind::NotFound => { /* didn't exist */ },
    Err(e) => return Err(e.into()),
}
```

📖 **[Full Guide: Common Footguns →](common-footguns.md#toctou-races)**

---

## 19. Using Enums Instead of Strings for Fixed Value Sets

**Rule:** ✅ Use enums for fixed sets you control | ❌ Strings lose compile-time safety

**Quick Check:**

- Fixed set of valid values (event types, states, modes)?
- Typos possible?
- Want compile-time validation?

**Common Pattern:**

```rust
// ✅ CORRECT - Type-safe enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    UserPromptSubmit,
    PostToolUse,
    Stop,
}

// Impossible to create invalid values!
pub struct Settings {
    pub hooks: HashMap<HookEvent, Vec<Config>>,
}

settings.add_hook(HookEvent::UserPromptSubmit, config);  // ✅ Type-safe
settings.add_hook(HookEvent::InvalidEvent, config);      // ❌ Compile error!
```

📖 **[Full Guide: Type Safety →](type-safety-deep-dive.md#enums-vs-strings)**

---

## 20. The Newtype Pattern for Preventing Type Confusion

**Rule:** ✅ Wrap primitives in distinct types to prevent mixing up IDs, units, paths | ❌ Using raw primitives allows parameter order mistakes

**Quick Check:**

- Multiple IDs with same primitive type (UserId, AssessmentId both i32)?
- Function parameters that could be swapped (all String or all i32)?
- Values with units that could be confused (meters vs kilometers)?
- Different file paths that shouldn't be mixed?

**Common Pattern:**

```rust
// ✅ CORRECT - Distinct newtype wrappers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssessmentId(i32);

fn get_user_assessment(user_id: UserId, assessment_id: AssessmentId) -> Result<Assessment> {
    // Compiler prevents parameter swap!
}

// Usage
get_user_assessment(UserId(42), AssessmentId(7));  // ✅ Type-safe
get_user_assessment(AssessmentId(7), UserId(42));  // ❌ Compile error!

// Zero runtime cost - newtype IS the inner type
// Access inner value: user_id.0
```

📖 **[Full Guide: Type Safety →](type-safety-deep-dive.md#4-the-newtype-pattern)**

---

## 21. "Did You Mean" Suggestions with Levenshtein Distance

**Rule:** ✅ Implement suggestions for validation errors on fixed value sets using `strsim` | ❌ Don't just list valid options

**Quick Check:**

- Validation error for fixed set of values?
- Users making typos?
- Want helpful error messages?

**Common Pattern:**

```rust
// ✅ GOOD - With suggestions
use strsim::levenshtein;

fn find_closest(input: &str, valid: &[&str]) -> Option<&str> {
    valid.iter()
        .map(|&opt| (opt, levenshtein(input, opt)))
        .filter(|(_, dist)| *dist <= 3)
        .min_by_key(|(_, dist)| *dist)
        .map(|(opt, _)| opt)
}

// Error: Unknown event 'UserPromtSubmit'. Did you mean 'UserPromptSubmit'?
if let Some(closest) = find_closest(input, VALID_EVENTS) {
    bail!("Unknown '{}'. Did you mean '{}'?", input, closest);
}
```

📖 **[Full Guide: Type Safety →](type-safety-deep-dive.md#did-you-mean)**

---

## Pre-PR Checklist

Quick checklist before submitting code:

**Code Quality:**

- [ ] All Option/Result types handled explicitly
- [ ] No bare `.unwrap()` except in tests
- [ ] No redundant imports (clippy clean)
- [ ] Loop-invariant computations outside loops
- [ ] No magic strings for validation

**File I/O:**

- [ ] Atomic writes for important files
- [ ] Parent directories created
- [ ] Integration tests with tempfile

**CLI/UX:**

- [ ] TTY detection for colored output
- [ ] User feedback for file operations
- [ ] Error messages are helpful

**Testing:**

- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] `cargo fmt --all` applied

---

**Need more details?** Jump to the relevant deep-dive guide

**[◀ Back to Index](index.md)** | **Document Version:** 2.0

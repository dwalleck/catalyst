# Common Footguns

This guide covers specific gotchas and common mistakes that trip up even experienced Rust developers.

## What This Guide Covers

1. **[Path Operations Return Options](#1-path-operations-return-options)** - Why Path methods are error-prone
2. **[TOCTOU Races](#2-toctou-races)** - Time-of-check-time-of-use vulnerabilities
3. **[Borrow Checker with HashSet](#3-borrow-checker-with-hashset)** - Collection borrowing pitfalls

**Quick Reference:** See [quick-reference.md](quick-reference.md) for scannable checklists

---

## 1. Path Operations Return Options

### Why This is a Common Footgun

Path operations like `file_name()`, `parent()`, and `extension()` return `Option<T>`, but developers often assume they'll always succeed and use `.unwrap()`. This is covered in detail in the [Error Handling Deep Dive](error-handling-deep-dive.md#2-common-footgun-path-operations).

### Quick Summary

```rust
// ❌ WRONG - Assumes file_name() always returns Some
let file_name = path.file_name().unwrap().to_string_lossy();

// ✅ CORRECT - Handle None case
let file_name = path.file_name()
    .map(|n| n.to_string_lossy())
    .unwrap_or_else(|| path.display().to_string().into());
```

**Why Path methods return Option:**

```rust
Path::new("/").file_name()          // None - root has no filename
Path::new("foo/..").file_name()     // None - parent reference
Path::new("/").parent()              // None - root has no parent
Path::new("Makefile").extension()    // None - no extension
```

**See full guide:** [Error Handling Deep Dive - Path Operations](error-handling-deep-dive.md#2-common-footgun-path-operations)

**[↑ Back to Quick Reference](quick-reference.md#4-common-footgun-path-operations)**

---

## 2. TOCTOU Races

### The Problem

**TOCTOU = Time-Of-Check-Time-Of-Use**

Checking if a file exists separately from using it creates a race condition where the file state can change between the check and use.

### Critical Example

**❌ BAD - Race condition:**

```rust
Commands::AddHook { path, ... } => {
    // Check if file exists
    let file_exists = std::path::Path::new(&path).exists();

    // ⚠️  Time passes... file could be created/deleted here!

    // Try to read based on old check
    let mut settings = ClaudeSettings::read(&path).unwrap_or_default();

    // Later, use outdated file_exists
    if file_exists {
        println!("Modified existing file");  // Might be wrong!
    } else {
        println!("Created new file");  // Might be wrong!
    }
}
```

**Race scenarios:**
1. **False negative:** File doesn't exist during check, gets created before read → wrong message
2. **False positive:** File exists during check, gets deleted before read → wrong message

### Solution: Check the Result, Not the Filesystem

**✅ GOOD - No race condition:**

```rust
Commands::AddHook { path, ... } => {
    // Try to read and let the Result tell us if it existed
    let (mut settings, file_existed) = match ClaudeSettings::read(&path) {
        Ok(s) => (s, true),   // File existed and was readable
        Err(_) => (ClaudeSettings::default(), false),  // File didn't exist
    };

    // ... add hook ...

    // Use the result from the ACTUAL operation
    if file_existed {
        println!("Modified existing file");  // We actually read it
    } else {
        println!("Created new file");  // We actually created it
    }
}
```

**Why this is better:**
1. **Atomic check-and-use:** Read attempt is a single atomic operation
2. **Truth from operation:** We know the file existed because we successfully read it
3. **No race window:** No time between check and use for state to change
4. **Handles all cases:** Covers not-exists, exists-but-unreadable, etc.

### Common TOCTOU Patterns

**File existence:**

```rust
// ❌ exists() then open()
if path.exists() { fs::File::open(path)? }

// ✅ Try open, handle NotFound
match fs::File::open(path) {
    Ok(f) => f,
    Err(e) if e.kind() == io::ErrorKind::NotFound => { /* handle */ },
    Err(e) => return Err(e.into()),
}
```

**Directory creation:**

```rust
// ❌ exists() then create
if !dir.exists() { fs::create_dir(dir)? }

// ✅ create_dir_all (idempotent)
fs::create_dir_all(dir)?;  // Succeeds if exists
```

**File metadata:**

```rust
// ❌ Check then use
if path.metadata()?.is_file() {
    fs::read(path)?
}

// ✅ Try operation, handle error
match fs::read(path) {
    Ok(data) => data,
    Err(e) if e.kind() == io::ErrorKind::InvalidInput => { /* not a file */ },
    Err(e) => return Err(e.into()),
}
```

### Security Implications

**Critical in security contexts:**

```rust
// 🔒 SECURITY ISSUE - TOCTOU vulnerability
fn check_and_open_secure_file(path: &Path) -> Result<File> {
    // Attacker could create symlink to /etc/passwd here!
    if path.exists() && is_safe_path(path) {
        // Between check and open, attacker swaps file
        fs::File::open(path)?  // Opens attacker's file!
    }
}

// ✅ SECURE - Open with specific flags
fn open_secure_file(path: &Path) -> Result<File> {
    fs::OpenOptions::new()
        .read(true)
        .create(false)    // Don't create
        .truncate(false)  // Don't modify
        .open(path)?      // Atomic open
    // Then verify it's what we expect
}
```

### The Golden Rule

**Never check filesystem state separately from using it. Let the operation itself tell you the state through its Result. Use idempotent operations like `create_dir_all()` instead of conditional operations.**

**[↑ Back to Quick Reference](quick-reference.md#18-toctou-races)**

---

## 3. Borrow Checker with HashSet

### The Problem

Creating a HashSet from borrowed data while simultaneously trying to mutate the original collection causes borrow checker errors.

### Classic Example

**❌ WRONG - Borrow checker error:**

```rust
pub fn merge(&mut self, other: ClaudeSettings) {
    // Immutable borrow here
    let existing_servers: HashSet<_> = self.enabled_mcpjson_servers.iter().collect();

    for server in other.enabled_mcpjson_servers {
        if !existing_servers.contains(&server) {
            // ERROR: Mutable borrow while immutable borrow exists
            self.enabled_mcpjson_servers.push(server);
        }
    }
}
```

**Compiler error:**
```
error[E0502]: cannot borrow `self.enabled_mcpjson_servers` as mutable
because it is also borrowed as immutable
```

**Why it fails:**
- `.iter()` creates references to items in `self.enabled_mcpjson_servers`
- These references live in the `HashSet<&String>`
- We then try to push (mut borrow) while HashSet still holds references (immut borrow)

### Solution: Clone or Copy Elements

**✅ CORRECT - Clone elements to break the borrow:**

```rust
pub fn merge(&mut self, other: ClaudeSettings) {
    // Clone elements, no references to self
    let existing_servers: HashSet<_> =
        self.enabled_mcpjson_servers.iter().cloned().collect();

    for server in other.enabled_mcpjson_servers {
        if !existing_servers.contains(&server) {
            self.enabled_mcpjson_servers.push(server);  // Now OK!
        }
    }
}
```

### Why .cloned() Works

```rust
// Without .cloned() - HashSet<&String> (references to self)
let bad: HashSet<&String> = self.vec.iter().collect();

// With .cloned() - HashSet<String> (owned copies, no borrows)
let good: HashSet<String> = self.vec.iter().cloned().collect();
```

### Alternative Solutions

**Option 1: Drain and rebuild** (if you're replacing the whole vec)

```rust
let existing: HashSet<_> = self.vec.drain(..).collect();
// Now self.vec is empty, no borrow issues
for item in other.vec {
    if !existing.contains(&item) {
        self.vec.push(item);
    }
}
```

**Option 2: Build new vec then swap**

```rust
let existing: HashSet<_> = self.vec.iter().cloned().collect();
let mut new_vec = self.vec.clone();
for item in other.vec {
    if !existing.contains(&item) {
        new_vec.push(item);
    }
}
self.vec = new_vec;
```

**Option 3: Use Entry API** (for HashMap)

```rust
for (key, value) in other.map {
    self.map.entry(key).or_insert(value);  // No borrow issues
}
```

### Performance Considerations

**Cost of .cloned():**
- O(n) time to clone elements
- O(n) space for owned copies

**Still better than O(n²) contains():**

```rust
// ❌ O(n²) - contains() is O(n) in Vec
for item in other.vec {
    if !self.vec.contains(&item) {  // O(n) lookup
        self.vec.push(item);
    }
}

// ✅ O(n) - HashSet lookup is O(1)
let existing: HashSet<_> = self.vec.iter().cloned().collect();  // O(n)
for item in other.vec {  // O(n)
    if !existing.contains(&item) {  // O(1) lookup
        self.vec.push(item);
    }
}
```

### The Golden Rule

**Use `.cloned()` or `.copied()` when creating a HashSet/HashMap from borrowed data if you need to mutate the original collection. This breaks the borrow relationship and satisfies the borrow checker.**

**[↑ Back to Quick Reference](quick-reference.md#17-borrow-checker-with-hashset)**

---

## Related Topics

### Error Handling
- **[Option handling](error-handling-deep-dive.md#1-understanding-option-types)** - Universal Option patterns
- **[Path operations](error-handling-deep-dive.md#2-common-footgun-path-operations)** - Full Path footgun guide

### File I/O
- **[Atomic writes](file-io-deep-dive.md#1-atomic-file-writes)** - Preventing data corruption
- **[Parent directory creation](file-io-deep-dive.md#2-parent-directory-creation)** - Avoiding TOCTOU with create_dir_all

### Performance
- **[Loop optimizations](performance-deep-dive.md#1-performance-critical-loop-optimizations)** - HashSet performance in loops

---

**[← Back to Index](index.md)** | **[Quick Reference →](quick-reference.md)**

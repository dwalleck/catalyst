# Type Safety Deep Dive

This guide shows the journey from string-based validation to compile-time type safety. Learn how to progressively improve code quality by leveraging Rust's type system.

## What This Guide Covers

1. **[The Journey: Strings → Constants → Enums](#1-the-journey-strings--constants--enums)** - Progressive improvement path
2. **[Using Constants for Validation](#2-using-constants-for-validation)** - First step from magic strings
3. **[Using Enums for Fixed Value Sets](#3-using-enums-for-fixed-value-sets)** - Compile-time safety
4. **[The Newtype Pattern](#4-the-newtype-pattern)** - Preventing type confusion with wrapper types
5. **[Immediate Validation in Setters](#5-immediate-validation-in-setters)** - Fail-fast pattern
6. **["Did You Mean" Suggestions](#6-did-you-mean-suggestions)** - User-friendly validation errors

**Quick Reference:** See [quick-reference.md](quick-reference.md) for scannable checklists

---

## 1. The Journey: Strings → Constants → Enums

### The Progressive Improvement Path

Most validation code starts with strings and can be progressively improved:

**Level 0: Magic Strings (❌ Never do this)**
```rust
if hook.r#type != "command" {  // What are the valid types?
    bail!("Invalid type");     // No context, no suggestions
}
```

**Level 1: Constants (✅ Better)**
```rust
const VALID_TYPES: &[&str] = &["command", "script"];
if !VALID_TYPES.contains(&hook.r#type.as_str()) {
    bail!("Invalid type. Valid: {}", VALID_TYPES.join(", "));
}
```

**Level 2: Enums (✅ Best)**
```rust
enum HookType { Command, Script }
// Compiler enforces - no validation needed!
```

This guide shows each step of this journey.

---

## 2. Using Constants for Validation

### The Problem

Using magic strings for validation makes code fragile and error-prone. Typos in string comparisons won't be caught at compile time, and adding new valid values requires searching through code to find all validation points.

```rust
// ❌ WRONG - Magic strings scattered throughout code
pub fn validate(&self) -> Result<()> {
    for hook in &config.hooks {
        if hook.r#type != "command" {  // Magic string
            anyhow::bail!("Unknown hook type '{}'", hook.r#type);
        }
    }
    Ok(())
}

// In CLI:
fn main() {
    // More magic strings
    settings.add_hook("UserPromptSubmit", hook_config);  // Typo-prone
    settings.add_hook("PostToolUse", hook_config);       // No validation
}
```

**Problems:**
- Typos not caught until runtime: `"UserPromtSubmit"` (missing 'p')
- No autocomplete/IDE support
- Can't easily see all valid values
- Changing a value requires finding all occurrences
- No compile-time validation

### Solution: Define Constants

```rust
// ✅ CORRECT - Define constants for all valid values

// In settings.rs or constants.rs
pub mod constants {
    // Hook types
    pub const HOOK_TYPE_COMMAND: &str = "command";
    // Future: HOOK_TYPE_SCRIPT, HOOK_TYPE_FUNCTION, etc.

    // Hook events (from Claude Code documentation)
    pub const EVENT_USER_PROMPT_SUBMIT: &str = "UserPromptSubmit";
    pub const EVENT_POST_TOOL_USE: &str = "PostToolUse";
    pub const EVENT_STOP: &str = "Stop";

    // All valid events for validation
    pub const VALID_EVENTS: &[&str] = &[
        EVENT_USER_PROMPT_SUBMIT,
        EVENT_POST_TOOL_USE,
        EVENT_STOP,
    ];

    // All valid hook types
    pub const VALID_HOOK_TYPES: &[&str] = &[
        HOOK_TYPE_COMMAND,
    ];
}

use constants::*;

pub fn validate(&self) -> Result<()> {
    for (event, configs) in &self.hooks {
        // Validate event name
        if !VALID_EVENTS.contains(&event.as_str()) {
            anyhow::bail!(
                "Unknown event '{}'. Valid events: {}",
                event,
                VALID_EVENTS.join(", ")
            );
        }

        for config in configs {
            for hook in &config.hooks {
                // Validate hook type
                if !VALID_HOOK_TYPES.contains(&hook.r#type.as_str()) {
                    anyhow::bail!(
                        "Unknown hook type '{}'. Valid types: {}",
                        hook.r#type,
                        VALID_HOOK_TYPES.join(", ")
                    );
                }
            }
        }
    }
    Ok(())
}
```

### CLI with Constants

```rust
use catalyst_core::settings::constants::*;

fn main() -> Result<()> {
    // Autocomplete and compile-time validation
    settings.add_hook(EVENT_USER_PROMPT_SUBMIT, HookConfig {
        matcher: None,
        hooks: vec![Hook {
            r#type: HOOK_TYPE_COMMAND.to_string(),
            command: "skill-activation.sh".to_string(),
        }],
    });

    // Typos caught by IDE (no such constant)
    // settings.add_hook(EVENT_USER_PROMT_SUBMIT, ...);  // Won't compile!

    Ok(())
}
```

### Benefits of Constants

- ✅ Autocomplete in IDE
- ✅ Typos caught at compile time
- ✅ Centralized valid values
- ✅ Easy to add new values
- ✅ Helpful validation error messages

### When to Use Constants vs Enums

| Approach | Use When | Benefits | Drawbacks |
|----------|----------|----------|-----------|
| **Magic Strings** | Never in production | Quick prototyping | No safety, typo-prone |
| **Constants** | Semi-dynamic values, external API | Flexible, clear, validated | Runtime validation needed |
| **Enums** | Fixed set of values you control | Compile-time safety, refactorable | Less flexible |

**[↑ Back to Quick Reference](quick-reference.md#13-using-constants-for-validation)**

---

## 3. Using Enums for Fixed Value Sets

### The Problem

Using strings (`&str` or `String`) to represent a fixed set of values (like event types, states, modes) loses compile-time type safety. Typos, invalid values, and inconsistencies can only be caught at runtime through validation code.

**❌ WRONG - String-based approach:**

```rust
// Settings uses HashMap<String, Vec<HookConfig>>
pub struct ClaudeSettings {
    pub hooks: HashMap<String, Vec<HookConfig>>,
}

// Must validate strings at runtime
pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) -> Result<()> {
    const VALID_EVENTS: &[&str] = &["UserPromptSubmit", "PostToolUse", "Stop"];

    // Manual validation required
    if !VALID_EVENTS.contains(&event) {
        anyhow::bail!("Unknown event '{}'", event);
    }

    self.hooks.entry(event.to_string()).or_default().push(hook_config);
    Ok(())
}

// Caller can make typos
settings.add_hook("UserPromtSubmit", config)?;  // Typo - caught at runtime
settings.add_hook("InvalidEvent", config)?;      // Invalid - caught at runtime
```

### Solution: Use Enums

**✅ CORRECT - Enum-based approach:**

```rust
// Define enum for fixed value set
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    UserPromptSubmit,
    PostToolUse,
    Stop,
}

// Implement Display for string representation
impl fmt::Display for HookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookEvent::UserPromptSubmit => write!(f, "UserPromptSubmit"),
            HookEvent::PostToolUse => write!(f, "PostToolUse"),
            HookEvent::Stop => write!(f, "Stop"),
        }
    }
}

// Implement FromStr for parsing (CLI use)
impl FromStr for HookEvent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "UserPromptSubmit" => Ok(HookEvent::UserPromptSubmit),
            "PostToolUse" => Ok(HookEvent::PostToolUse),
            "Stop" => Ok(HookEvent::Stop),
            _ => anyhow::bail!(
                "Unknown event '{}'. Valid events: UserPromptSubmit, PostToolUse, Stop",
                s
            ),
        }
    }
}

// Settings uses HashMap<HookEvent, Vec<HookConfig>>
pub struct ClaudeSettings {
    pub hooks: HashMap<HookEvent, Vec<HookConfig>>,
}

// No runtime validation needed - type system enforces correctness
pub fn add_hook(&mut self, event: HookEvent, hook_config: HookConfig) -> Result<()> {
    // Event is already validated by type system
    self.hooks.entry(event).or_default().push(hook_config);
    Ok(())
}

// Compiler catches typos and invalid values
settings.add_hook(HookEvent::UserPromptSubmit, config)?;  // ✅ Compiles
settings.add_hook(HookEvent::UserPromtSubmit, config)?;   // ❌ Compile error
settings.add_hook(HookEvent::InvalidEvent, config)?;      // ❌ Compile error
```

### Benefits of Enum Approach

**1. Compile-Time Safety**
- Typos caught by compiler, not at runtime
- Impossible to use invalid values
- IDE autocomplete shows all valid options
- Refactoring is safe (compiler finds all usages)

**2. Less Validation Code**
- No need to check strings against valid values
- No need to maintain validation constants
- Methods can be simpler and more focused

**3. Better Performance**
- Enums are stack-allocated (no heap allocation)
- Hash lookups are faster (enum hash vs string hash)
- Comparisons are faster (integer vs string comparison)

**4. Better Documentation**
- Valid values are explicit in the type definition
- No need to document valid strings in comments
- Self-documenting API

### When to Use Enums

Use enums for:
- ✅ Fixed set of values (event types, states, modes)
- ✅ Configuration options with known variants
- ✅ Status codes or result types
- ✅ Command types or operation modes
- ✅ HashMap/HashSet keys with limited domain

Keep strings for:
- ❌ User-generated content
- ❌ File paths
- ❌ External data from APIs
- ❌ Open-ended text fields
- ❌ Values that can be extended by users

### Integration with Serde

Enums serialize to strings automatically with serde:

```rust
#[derive(Serialize, Deserialize)]
pub enum HookEvent {
    UserPromptSubmit,  // Serializes as "UserPromptSubmit"
    PostToolUse,       // Serializes as "PostToolUse"
    Stop,              // Serializes as "Stop"
}

// JSON roundtrip works seamlessly
let json = r#"{"hooks": {"UserPromptSubmit": [...]}}"#;
let settings: ClaudeSettings = serde_json::from_str(json)?;  // ✅ Works
```

### Required Trait Derives

For enum HashMap keys, derive these traits:

```rust
#[derive(
    Debug,           // Debugging output
    Clone,           // Can be cloned
    Copy,            // Stack-copyable (for simple enums)
    PartialEq,       // Equality comparison
    Eq,              // Full equality (required for Hash)
    Hash,            // HashMap key support
    Serialize,       // JSON serialization
    Deserialize,     // JSON deserialization
)]
pub enum HookEvent {
    UserPromptSubmit,
    PostToolUse,
    Stop,
}
```

### Impact on Code Quality

**Before (strings, 30 lines of validation):**
```rust
const VALID_EVENTS: &[&str] = &["UserPromptSubmit", "PostToolUse", "Stop"];

pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) -> Result<()> {
    // Validate event name (10 lines)
    if !VALID_EVENTS.contains(&event) {
        anyhow::bail!("Unknown event '{}'. Valid events: {}",
            event, VALID_EVENTS.join(", "));
    }
    // ...
}
```

**After (enums, 10 lines total):**
```rust
pub fn add_hook(&mut self, event: HookEvent, hook_config: HookConfig) -> Result<()> {
    // Event validation unnecessary - type system guarantees correctness
    self.hooks.entry(event).or_default().push(hook_config);
    Ok(())
}
```

**[↑ Back to Quick Reference](quick-reference.md#19-using-enums-instead-of-strings)**

---

## 4. The Newtype Pattern

### The Problem

Using primitive types (like `i32`, `u32`, `String`) for different concepts leads to **type confusion** - accidentally mixing up values that have the same underlying type but completely different meanings.

**❌ WRONG - Primitive types everywhere:**

```rust
// All these IDs are just i32 - easy to mix up!
fn transfer_points(from_user: i32, to_user: i32, assessment_id: i32, points: i32) -> Result<()> {
    // Which parameter is which? Compiler can't help!
    // What if someone calls: transfer_points(points, assessment_id, from_user, to_user)?
    // All i32, so it compiles... but disaster!
}

// Similar problem with String types
fn save_config(db_path: String, config_path: String, backup_path: String) -> Result<()> {
    // Easy to pass paths in wrong order - all are String!
}

// Or units that can be confused
fn calculate_velocity(distance: f64, time: f64) -> f64 {
    distance / time
    // Are these meters or kilometers? Seconds or hours?
    // calculate_velocity(100.0, 2.0) - what does this mean?
}
```

**Problems:**
- Compiler allows mixing up parameters of same primitive type
- Function signatures don't document units or meaning
- Typos in parameter order aren't caught
- No autocomplete hints about what values mean
- Refactoring is error-prone

### Solution: Newtype Pattern

Wrap primitives in single-field structs to create distinct types:

**✅ CORRECT - Newtype wrappers:**

```rust
// Distinct types prevent confusion
struct UserId(i32);
struct AssessmentId(i32);
struct Points(i32);

fn transfer_points(
    from_user: UserId,
    to_user: UserId,
    assessment_id: AssessmentId,
    points: Points
) -> Result<()> {
    // Wrong parameter order caught by compiler!
    // transfer_points(points, assessment_id, from_user, to_user) ❌ Compile error!

    // Access inner value when needed
    let from_id: i32 = from_user.0;
    let points_value: i32 = points.0;

    // ... implementation ...
    Ok(())
}

// Usage - type safety at call site
transfer_points(
    UserId(42),
    UserId(100),
    AssessmentId(7),
    Points(50)
)?;
```

### Basic Newtype Implementation

**Minimal implementation:**
```rust
// Simple wrapper
struct UserId(i32);

// Access inner value
let user_id = UserId(42);
let id_value: i32 = user_id.0;

// Cannot mix with other i32 values
let assessment_id: AssessmentId = user_id; // ❌ Compile error - different types!
```

**Production-ready implementation:**
```rust
// Derive common traits for ergonomics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(i32);

impl UserId {
    // Constructor with validation
    pub fn new(id: i32) -> Result<Self> {
        if id <= 0 {
            anyhow::bail!("User ID must be positive, got {}", id);
        }
        Ok(UserId(id))
    }

    // Safe getter
    pub fn get(&self) -> i32 {
        self.0
    }
}

// Display for user-facing output
impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User#{}", self.0)
    }
}

// Usage
let user_id = UserId::new(42)?;
println!("{}", user_id); // Prints: User#42
```

### Common Use Cases

**1. Preventing ID Confusion:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssessmentId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResponseId(i32);

// Compiler prevents mixing these up
fn get_user_assessment(user_id: UserId, assessment_id: AssessmentId) -> Result<Assessment> {
    // Cannot accidentally swap parameters
}
```

**2. Path Type Safety:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabasePath(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPath(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupPath(PathBuf);

impl DatabasePath {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Validate it's a valid database path
        if path.extension().and_then(|s| s.to_str()) != Some("db") {
            anyhow::bail!("Database path must end with .db, got: {}", path.display());
        }
        Ok(DatabasePath(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

// Usage
fn init_database(db_path: DatabasePath, backup_path: BackupPath) -> Result<()> {
    // Cannot accidentally swap paths
    let conn = Connection::open(db_path.as_path())?;
    // ...
}
```

**3. Unit Type Safety:**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seconds(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetersPerSecond(f64);

impl MetersPerSecond {
    pub fn calculate(distance: Meters, time: Seconds) -> Self {
        MetersPerSecond(distance.0 / time.0)
    }
}

// Usage
let distance = Meters(100.0);
let time = Seconds(10.0);
let velocity = MetersPerSecond::calculate(distance, time);

// Cannot accidentally use raw floats
// let velocity = MetersPerSecond::calculate(100.0, 10.0); // ❌ Compile error
```

**4. Configuration Values:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Result<Self> {
        if port < 1024 {
            anyhow::bail!("Port must be >= 1024 (privileged ports), got {}", port);
        }
        Ok(Port(port))
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaxConnections(usize);

impl MaxConnections {
    pub fn new(max: usize) -> Result<Self> {
        if max == 0 {
            anyhow::bail!("Max connections must be > 0");
        }
        Ok(MaxConnections(max))
    }
}
```

### Serde Integration

Newtypes work seamlessly with serde:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)] // Serialize as the inner type
pub struct UserId(i32);

// JSON roundtrip
let user_id = UserId(42);
let json = serde_json::to_string(&user_id)?; // "42"
let parsed: UserId = serde_json::from_str(&json)?; // UserId(42)

// In structs
#[derive(Serialize, Deserialize)]
pub struct User {
    id: UserId,        // Serializes as plain i32
    name: String,
}

// JSON: {"id": 42, "name": "Alice"}
```

### Benefits of Newtype Pattern

**1. Compile-Time Safety**
- Impossible to mix up different ID types
- Parameter order mistakes caught by compiler
- Type mismatches caught early

**2. Self-Documenting Code**
- Function signatures show exactly what's expected
- No need to document "id1 is user ID, id2 is assessment ID"
- IDE shows type hints at call sites

**3. Centralized Validation**
- Validation logic in one place (the constructor)
- Once constructed, value is known to be valid
- No need to re-validate throughout code

**4. Safer Refactoring**
- Changing ID representation (i32 → u64) is easy
- Compiler finds all usages
- Change inner type without changing API

**5. Better Type Checking**
```rust
// Before (primitives)
fn process(user: i32, score: i32, attempts: i32) { ... }
process(score, user, attempts); // ❌ Compiles but wrong!

// After (newtypes)
fn process(user: UserId, score: Score, attempts: Attempts) { ... }
process(score, user, attempts); // ✅ Compile error - caught immediately!
```

### When to Use Newtypes

**Use newtypes for:**
- ✅ Different kinds of IDs (UserId, PostId, CommentId)
- ✅ File paths with different purposes (ConfigPath, DataPath, BackupPath)
- ✅ Values with units (Meters, Seconds, Bytes)
- ✅ Configuration values with validation (Port, MaxConnections, Timeout)
- ✅ Preventing primitive obsession (Email, PhoneNumber, ZipCode)

**Keep primitives for:**
- ❌ Simple counters or temporary calculations
- ❌ Generic numeric operations
- ❌ Values that truly are interchangeable
- ❌ Performance-critical hot paths (though cost is usually negligible)

### Cost and Performance

Newtypes are **zero-cost abstractions:**
```rust
struct UserId(i32);

// At runtime, UserId IS an i32
// No memory overhead, no performance cost
// #[repr(transparent)] guarantees same layout
```

Memory layout:
```rust
std::mem::size_of::<i32>()     // 4 bytes
std::mem::size_of::<UserId>()  // 4 bytes - same!
```

### Pattern: Type Alias vs Newtype

**Type alias (NOT safe):**
```rust
type UserId = i32;  // Just an alias, not a new type!

fn process(user_id: UserId, points: i32) {
    // Can still mix them up - UserId is just i32
}

let user = 42_i32;
let points = 100_i32;
process(points, user); // ❌ Compiles but wrong! Type alias doesn't prevent this.
```

**Newtype (safe):**
```rust
struct UserId(i32);  // New distinct type

fn process(user_id: UserId, points: i32) { ... }

let user = UserId(42);
let points = 100_i32;
process(points, user); // ✅ Compile error - caught!
```

**Rule:** Use newtypes for type safety, not type aliases.

### Real-World Example: Catalyst Database IDs

```rust
// Before: All IDs are i32
fn get_assessment_responses(
    db: &Database,
    user_id: i32,
    assessment_id: i32,
    response_id: i32
) -> Result<Response> {
    // Easy to mix these up in queries
}

// After: Distinct newtype IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssessmentId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResponseId(i32);

fn get_assessment_responses(
    db: &Database,
    user_id: UserId,
    assessment_id: AssessmentId,
    response_id: ResponseId
) -> Result<Response> {
    // Impossible to mix up parameters now!
    let query = "SELECT * FROM responses WHERE user_id = ?1 AND assessment_id = ?2 AND id = ?3";
    db.query_row(query, params![user_id.0, assessment_id.0, response_id.0], |row| {
        // ...
    })
}
```

### Combining with Enums

Newtypes and enums work together:

```rust
// Newtype for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(i32);

// Enum for fixed states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

// Both provide different kinds of safety
struct User {
    id: UserId,      // Newtype: prevents ID confusion
    role: UserRole,  // Enum: prevents invalid roles
}
```

**[↑ Back to Quick Reference](quick-reference.md)**

---

## 5. Immediate Validation in Setters

### The Problem

Deferring validation to a separate `validate()` method allows invalid state to be created, leading to confusing errors far from where the problem originated.

```rust
// ❌ BAD - Can create invalid state
pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) {
    self.hooks
        .entry(event.to_string())
        .or_default()
        .push(hook_config);
    // Invalid data is now in the struct!
}

// Later, somewhere else...
fn main() -> Result<()> {
    let mut settings = ClaudeSettings::default();

    // This succeeds even with invalid event name
    settings.add_hook("InvalidEvent", HookConfig { ... });

    // Error happens here, far from the source
    settings.validate()?;  // Error: "Unknown event 'InvalidEvent'"

    Ok(())
}
```

**Problems:**
1. Invalid state can exist in memory
2. Error discovered far from where it was created
3. Multiple invalid items can accumulate before validation
4. Harder to debug - which add_hook() call was wrong?

### Solution: Immediate Validation

```rust
// ✅ GOOD - Validate immediately
pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) -> Result<()> {
    use constants::*;

    // Validate event name
    if !VALID_EVENTS.contains(&event) {
        anyhow::bail!(
            "Unknown event '{}'. Valid events: {}",
            event,
            VALID_EVENTS.join(", ")
        );
    }

    // Validate hooks array not empty
    if hook_config.hooks.is_empty() {
        anyhow::bail!("Empty hooks array for {} event", event);
    }

    // Validate hook types
    for hook in &hook_config.hooks {
        if !VALID_HOOK_TYPES.contains(&hook.r#type.as_str()) {
            anyhow::bail!(
                "Unknown hook type '{}' in {} event. Valid types: {}",
                hook.r#type, event, VALID_HOOK_TYPES.join(", ")
            );
        }
    }

    // Only add if all validations pass
    self.hooks.entry(event.to_string()).or_default().push(hook_config);

    Ok(())
}
```

### Benefits

1. **Fail fast:** Errors caught immediately at the source
2. **Clear error location:** Stack trace points to exact add_hook() call
3. **No invalid state:** Struct always remains valid
4. **Better error messages:** Can include context about what was being added
5. **Separate validate() becomes optional:** Only needed for loaded/deserialized data

### When to Use

**Immediate validation for:**
- ✅ Builder/setter methods that modify state
- ✅ Operations that can have invalid inputs
- ✅ Data transformations that may fail

**Deferred validation for:**
- ❌ Batch operations where you want to collect all errors
- ❌ Data loaded from external sources (validate after deserialization)
- ❌ Performance-critical code where validation overhead is too high

### Pattern: Keep Both Methods

```rust
impl ClaudeSettings {
    // Immediate validation for programmatic use
    pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) -> Result<()> {
        // ... validate immediately ...
        Ok(())
    }

    // Separate validate() for loaded data
    pub fn validate(&self) -> Result<()> {
        // Validate entire struct (for data loaded from JSON)
        for (event, configs) in &self.hooks {
            // ... validate each hook ...
        }
        Ok(())
    }
}
```

**Usage:**
```rust
// Programmatic use - immediate validation
settings.add_hook("UserPromptSubmit", hook_config)?;  // Fails immediately

// Loaded data - batch validation
let settings = ClaudeSettings::read("settings.json")?;
settings.validate()?;  // Validate everything at once
```

**[↑ Back to Quick Reference](quick-reference.md#16-immediate-validation-in-setter-methods)**

---

## 6. "Did You Mean" Suggestions

### The Problem

Validation error messages that only list valid options force users to manually spot typos and correct them. When users make small typos, the error message should suggest the closest valid option.

**❌ WRONG - No suggestions:**

```rust
impl FromStr for HookEvent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "UserPromptSubmit" => Ok(HookEvent::UserPromptSubmit),
            "PostToolUse" => Ok(HookEvent::PostToolUse),
            "Stop" => Ok(HookEvent::Stop),
            _ => anyhow::bail!(
                "Unknown event '{}'. Valid events: UserPromptSubmit, PostToolUse, Stop",
                s
            ),
        }
    }
}
```

**Error output:**
```
Error: Unknown event 'UserPromtSubmit'. Valid events: UserPromptSubmit, PostToolUse, Stop
```

User must manually compare the input against all valid options to find the typo.

### Solution: Use strsim for Suggestions

**✅ CORRECT - With suggestions:**

```rust
use strsim::levenshtein;

/// Find the closest match from a list of valid options using Levenshtein distance
fn find_closest_match<'a>(input: &str, valid_options: &[&'a str]) -> Option<&'a str> {
    let threshold = 3; // Maximum edit distance for suggestions

    valid_options
        .iter()
        .map(|&option| (option, levenshtein(input, option)))
        .filter(|(_, distance)| *distance <= threshold)
        .min_by_key(|(_, distance)| *distance)
        .map(|(option, _)| option)
}

impl FromStr for HookEvent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "UserPromptSubmit" => Ok(HookEvent::UserPromptSubmit),
            "PostToolUse" => Ok(HookEvent::PostToolUse),
            "Stop" => Ok(HookEvent::Stop),
            _ => {
                let valid_events = ["UserPromptSubmit", "PostToolUse", "Stop"];
                let suggestion = find_closest_match(s, &valid_events);

                if let Some(closest) = suggestion {
                    anyhow::bail!(
                        "Unknown event '{}'. Did you mean '{}'? Valid events: {}",
                        s,
                        closest,
                        valid_events.join(", ")
                    );
                } else {
                    anyhow::bail!(
                        "Unknown event '{}'. Valid events: {}",
                        s,
                        valid_events.join(", ")
                    );
                }
            }
        }
    }
}
```

**Error output:**
```
Error: Unknown event 'UserPromtSubmit'. Did you mean 'UserPromptSubmit'? Valid events: UserPromptSubmit, PostToolUse, Stop
```

User immediately sees what they typed wrong and the correct spelling.

### Benefits

**1. Faster Error Resolution**
- Users don't waste time manually comparing strings
- Immediately see likely correction
- Reduces frustration with validation errors

**2. Better User Experience**
- CLI feels intelligent and helpful
- Professional error messaging
- Reduces support burden

**3. Minimal Performance Cost**
- Levenshtein distance is O(mn) where m,n are string lengths
- Only computed on error path (not hot path)
- strsim crate has no dependencies

### Implementation Details

**Adding the strsim crate:**

```toml
[dependencies]
strsim = "0.11"  # String similarity for "did you mean" suggestions
```

**Choosing the threshold:**
- **Threshold = 3**: Catches most typos without false positives
- Too low (1-2): May miss valid suggestions
- Too high (5+): May suggest unrelated strings

**Examples of edit distances:**

```rust
levenshtein("UserPromtSubmit", "UserPromptSubmit") // 1 (missing 'p')
levenshtein("PostTolUse", "PostToolUse")            // 1 (missing 'o')
levenshtein("aceptEdits", "acceptEdits")            // 1 (missing 'c')
levenshtein("askk", "ask")                          // 1 (extra 'k')
levenshtein("CompletlyWrong", "UserPromptSubmit")   // 14 (too different)
```

### When to Use Suggestions

**Use suggestions for:**
- ✅ Fixed enums/constants with known valid values
- ✅ Configuration keys (permission modes, event names, etc.)
- ✅ Command-line arguments
- ✅ Status values or operation modes

**Don't use suggestions for:**
- ❌ User-generated content (names, descriptions)
- ❌ Open-ended inputs
- ❌ File paths (use "file not found" instead)
- ❌ Large sets of valid options (>20 items - too slow)

### Real-World Results

**Before (no suggestions):**
```
Error: Unknown event 'UserPromtSubmit'. Valid events: UserPromptSubmit, PostToolUse, Stop
```
User time to fix: ~30 seconds (manual comparison)

**After (with suggestions):**
```
Error: Unknown event 'UserPromtSubmit'. Did you mean 'UserPromptSubmit'? Valid events: UserPromptSubmit, PostToolUse, Stop
```
User time to fix: ~5 seconds (copy suggested value)

**Time saved per error:** ~25 seconds

**[↑ Back to Quick Reference](quick-reference.md#20-did-you-mean-suggestions)**

---

## Related Topics

### Error Handling
- **[Option handling](error-handling-deep-dive.md#1-understanding-option-types)** - Type-safe null handling
- **[expect vs unwrap](error-handling-deep-dive.md#3-expect-vs-unwrap-vs--decision-guide)** - Error messaging

### Fundamentals
- **[CLI user feedback](fundamentals-deep-dive.md#cli-user-feedback)** - Helpful user messages
- **[Validation patterns](fundamentals-deep-dive.md)** - General validation strategies

### Performance
- **[HashMap optimization](performance-deep-dive.md)** - Enum vs String performance

---

**[← Back to Index](index.md)** | **[Quick Reference →](quick-reference.md)**

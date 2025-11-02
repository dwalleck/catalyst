# Type Safety Deep Dive

This guide shows the journey from string-based validation to compile-time type safety. Learn how to progressively improve code quality by leveraging Rust's type system.

## What This Guide Covers

1. **[The Journey: Strings → Constants → Enums](#1-the-journey-strings--constants--enums)** - Progressive improvement path
2. **[Using Constants for Validation](#2-using-constants-for-validation)** - First step from magic strings
3. **[Using Enums for Fixed Value Sets](#3-using-enums-for-fixed-value-sets)** - Compile-time safety
4. **[Immediate Validation in Setters](#4-immediate-validation-in-setters)** - Fail-fast pattern
5. **["Did You Mean" Suggestions](#5-did-you-mean-suggestions)** - User-friendly validation errors

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

## 4. Immediate Validation in Setters

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

## 5. "Did You Mean" Suggestions

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

# Performance Deep Dive

This guide covers performance optimization patterns in Rust, focusing on hot loops and proper use of zero-cost abstractions.

## What This Guide Covers

1. **[Performance-Critical Loop Optimizations](#1-performance-critical-loop-optimizations)** - Moving invariant computations out of hot loops
2. **[When NOT to Use Zero-Copy Abstractions](#2-when-not-to-use-zero-copy-abstractions)** - Understanding abstraction limitations

**Quick Reference:** See [quick-reference.md](quick-reference.md) for scannable checklists

---

## 1. Performance-Critical Loop Optimizations

### The Problem

Creating objects inside hot loops (loops that execute many times) can severely degrade performance, even if each individual operation is cheap. This is especially critical for zero-cost abstractions like `UniCase` that are meant to avoid allocations.

### Critical Example: UniCase in Hot Loop

**❌ WRONG - Creates UniCase wrapper inside the loop for EVERY keyword:**

```rust
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

**✅ CORRECT - Create prompt wrapper ONCE outside the loop:**

```rust
let prompt_unicase = UniCase::new(prompt.as_str());  // Created once!

let keyword_match = triggers.keywords.iter().any(|kw| {
    let keyword_unicase = UniCase::new(kw.as_str());  // Only keyword wrapper created per iteration
    prompt_unicase.as_ref().contains(keyword_unicase.as_ref())
});
```

### General Pattern: Loop-Invariant Code Motion

**Identify what's loop-invariant:**

If a value doesn't change between loop iterations, compute it ONCE before the loop.

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

#### 1. String Operations

```rust
// ❌ BAD - Lowercases prompt for every keyword
for keyword in keywords {
    if prompt.to_lowercase().contains(&keyword.to_lowercase()) { }
}

// ✅ GOOD - Lowercase prompt once
let prompt_lower = prompt.to_lowercase();
for keyword in keywords {
    if prompt_lower.contains(&keyword.to_lowercase()) { }
}

// ✅ EVEN BETTER - Pre-lowercase keywords at startup
// (See Section 2 for full pattern)
```

#### 2. Regex Compilation

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

#### 3. Collection Allocations

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

**1. Code Review Checklist:**
- Look for `new()`, `clone()`, `to_owned()`, `to_string()` inside loops
- Look for repeated function calls with same arguments
- Look for collection allocations (`Vec::new()`, `HashMap::new()`)

**2. Profiling:**

```bash
# Use cargo flamegraph to find hot loops
cargo flamegraph --bin your-binary

# Use cargo bench for microbenchmarks
cargo bench
```

**3. Think Like the Compiler:**
- Ask: "Does this value change between iterations?"
- If NO → Move it outside the loop
- If YES → Keep it inside, but minimize allocations

### Real-World Impact

**skill_activation_prompt.rs with 100 keywords:**

- **Before fix**: `100 * num_skills` UniCase wrapper creations
- **After fix**: `num_skills` UniCase wrapper creations
- **With 10 skills**: 1000 → 10 wrapper creations (99% reduction)

### The Golden Rule

**CRITICAL: In hot loops (>100 iterations), move ALL loop-invariant computations outside the loop. Profile performance-critical code to verify optimizations.**

**[↑ Back to Quick Reference](quick-reference.md#7-performance-critical-loop-optimizations)**

---

## 2. When NOT to Use Zero-Copy Abstractions

### The Problem

Zero-copy abstractions like `UniCase` are designed for **specific use cases** (equality comparison). Using them incorrectly for other operations (like substring matching) can lead to bugs or unexpected behavior.

### Critical Bug Example: UniCase for Substring Matching

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

### The Correct Approach: Pre-Lowercased Strings

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

- ✅ Understand what operations they support
- ✅ Don't assume standard operations (like `contains()`) work the same way
- ✅ When in doubt, use standard library methods with explicit lowercasing
- ⚠️ Premature optimization can introduce subtle bugs

### Performance Impact of Correct Approach

**Before fix (broken UniCase approach):**
- Unknown behavior, potential bugs

**After fix (pre-lowercased keywords):**
- One allocation per activation: `prompt.to_lowercase()` (~50-200 bytes)
- Keywords lowercased once at startup, not in hot loop
- Predictable, correct behavior with minimal overhead

### Real-World Examples

**✅ Correct use of UniCase - Equality comparison:**

```rust
use unicase::UniCase;

// HashMap with case-insensitive keys
let mut map: HashMap<UniCase<String>, Value> = HashMap::new();
map.insert(UniCase::new("API".to_string()), value);

// Equality check works perfectly
if UniCase::new("api") == UniCase::new("API") {  // ✅ Works correctly
    println!("Match!");
}
```

**❌ Incorrect use of UniCase - Substring matching:**

```rust
// DON'T DO THIS
let text = UniCase::new("The API is ready");
let keyword = UniCase::new("api");
if text.as_ref().contains(keyword.as_ref()) {  // ❌ May not work as expected
    // ...
}
```

**✅ Correct alternative - Pre-lowercased strings:**

```rust
// DO THIS INSTEAD
let text_lower = "The API is ready".to_lowercase();
let keyword_lower = "api";
if text_lower.contains(keyword_lower) {  // ✅ Works correctly
    // ...
}
```

### Key Takeaway

**Zero-copy abstractions are powerful but specialized.** Always verify they support your actual use case:

- ✅ `UniCase` for equality: `if key1 == key2`
- ❌ `UniCase` for substring: `if text.contains(substring)`
- ✅ `UniCase` for HashMap keys: `HashMap<UniCase<String>, V>`
- ❌ `UniCase` for arbitrary string operations

**[↑ Back to Quick Reference](quick-reference.md#8-when-not-to-use-zero-copy-abstractions)**

---

## Related Topics

### Error Handling
- **[Option handling](error-handling-deep-dive.md#1-understanding-option-types)** - Type-safe iteration patterns
- **[expect vs unwrap](error-handling-deep-dive.md#3-expect-vs-unwrap-vs--decision-guide)** - Error handling in loops

### Fundamentals
- **[Tracing subscribers](fundamentals-deep-dive.md#tracing-subscribers)** - Logging performance
- **[Duplicated logic](fundamentals-deep-dive.md#duplicated-logic)** - DRY principle

### Type Safety
- **[Enums vs Strings](type-safety-deep-dive.md#3-using-enums-for-fixed-value-sets)** - Enum performance benefits
- **[HashMap optimization](type-safety-deep-dive.md)** - Type-safe collections

---

**[← Back to Index](index.md)** | **[Quick Reference →](quick-reference.md)**

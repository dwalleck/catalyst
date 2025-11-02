# Contributing New Rust Lessons

This guide explains how to add new lessons learned to the Rust Lessons documentation.

## 📋 Quick Checklist

When adding a new lesson:

- [ ] Determine which deep-dive guide it belongs to
- [ ] Add entry to quick-reference.md
- [ ] Add detailed section to appropriate deep-dive guide
- [ ] Update index.md if adding a new category
- [ ] Add cross-references to related topics
- [ ] Test all links work correctly

---

## 🎯 Step-by-Step Process

### Step 1: Identify the Category

Determine which deep-dive guide your lesson belongs to:

| Category | Use When Lesson Is About |
|----------|-------------------------|
| **error-handling-deep-dive.md** | Option/Result handling, expect vs unwrap, error propagation |
| **file-io-deep-dive.md** | File operations, atomic writes, testing file I/O |
| **type-safety-deep-dive.md** | Validation, constants, enums, type system usage |
| **performance-deep-dive.md** | Optimization, loop performance, memory allocation |
| **common-footguns.md** | Common mistakes, gotchas, tricky edge cases |
| **fundamentals-deep-dive.md** | Basic patterns, imports, logging, CLI UX |

**Can't decide?** Pick the most specific category. If it spans multiple categories, choose one as primary and add cross-references to others.

---

### Step 2: Add to Quick Reference

Edit **quick-reference.md** and add your lesson in the appropriate section.

**Format template:**

```markdown
## [NUMBER]. [Lesson Title]

**Rule:** ✅ [What to do] | ❌ [What not to do]

**Quick Check:**
- [Question 1 to identify this issue]
- [Question 2 to identify this issue]
- [Question 3 to identify this issue]

**Common Pattern:**
\`\`\`rust
// ❌ BAD
[bad example - 1-3 lines]

// ✅ GOOD
[good example - 1-3 lines]
\`\`\`

📖 **[Full Guide: [Category] →]([deep-dive-file].md#[anchor])**
```

**Example:**

```markdown
## 21. Using ? Operator Instead of unwrap()

**Rule:** ✅ Use `?` to propagate errors | ❌ Don't use `unwrap()` in functions that return Result

**Quick Check:**
- Does your function return `Result<T, E>`?
- Are you calling fallible operations?
- Could the operation fail in production?

**Common Pattern:**
\`\`\`rust
// ❌ BAD
fn read_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path).unwrap(); // Panics!
    parse_config(&content)
}

// ✅ GOOD
fn read_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?; // Propagates error
    parse_config(&content)
}
\`\`\`

📖 **[Full Guide: Error Handling →](error-handling-deep-dive.md#using-operator)**
```

**Important:**
- Keep examples SHORT (1-3 lines each)
- Focus on the core pattern
- The full explanation goes in the deep-dive

---

### Step 3: Add to Deep-Dive Guide

Edit the appropriate deep-dive guide and add a full section.

**Format template:**

```markdown
## [NUMBER]. [Lesson Title]

### The Problem

[Explain what goes wrong when developers don't follow this pattern. Include specific scenarios.]

```rust
// ❌ WRONG - [Why this is wrong]
[bad example with comments explaining the issue]
```

**Problems:**
- [Specific issue 1]
- [Specific issue 2]
- [Specific issue 3]

### Solution

**✅ CORRECT - [Why this is right]:**

```rust
[good example with comments explaining the approach]
```

**Why this works:**
1. [Reason 1]
2. [Reason 2]
3. [Reason 3]

### When to Use This Pattern

[Guidance on when this pattern applies vs when alternatives are better]

### Real-World Examples

[Examples from the actual codebase showing the pattern in practice]

```rust
// Example from [file.rs:123]
[real code example]
```

### The Golden Rule

**[One-sentence summary of the lesson that captures the essence]**

**[↑ Back to Quick Reference](quick-reference.md#[lesson-number]-[anchor])**
```

**Tips for writing deep-dives:**
- Start with a concrete problem
- Show the mistake BEFORE the fix
- Explain WHY, not just WHAT
- Include real code examples from the project
- Add a memorable "golden rule" summary
- Keep sections under 200 lines

---

### Step 4: Update Cross-References

Add references to your new lesson in related sections.

**Example:** If adding a lesson about HashMap performance, update:
- **performance-deep-dive.md** → Add to "Related Topics" section
- **type-safety-deep-dive.md** → Reference in "Enums vs Strings" section if relevant
- **quick-reference.md** → Ensure your entry links to the deep-dive

**Pattern for cross-references:**

```markdown
### Related Topics

- **[Your New Topic](deep-dive-file.md#your-topic)** - Brief description
```

---

### Step 5: Update Index (If Needed)

If your lesson introduces a **new major category** (rare), update **index.md**:

1. Add to the "Documentation Structure" table
2. Add to the "Topic Index" (alphabetically)
3. Add to appropriate "Learning Path" if relevant

**Most lessons won't require this step** - they'll fit into existing categories.

---

### Step 6: Test Links

Verify all your links work:

```bash
# Check quick-reference links to deep-dives
grep -n "Full Guide:" docs/rust-lessons/quick-reference.md

# Check cross-references
grep -n "Related Topics" docs/rust-lessons/*.md

# Check all markdown links (if you have a link checker)
markdown-link-check docs/rust-lessons/*.md
```

**Manual check:**
- Click every link you added
- Verify anchors work (GitHub auto-generates them from headers)
- Ensure "Back to Quick Reference" links work

---

## 🔧 Common Scenarios

### Scenario 1: New Pattern in Existing Category

**Example:** Adding "Using PathBuf vs &Path" to error-handling

1. Add to **quick-reference.md** as section 22 (next available number)
2. Add full section to **error-handling-deep-dive.md**
3. Add cross-reference from **file-io-deep-dive.md** (related)
4. Done!

### Scenario 2: Expanding Existing Lesson

**Example:** Adding more examples to "Option Handling"

1. Update the deep-dive section with new examples
2. Optionally add a note to quick-reference if the pattern changed
3. Update "Last Updated" date in deep-dive file
4. Done!

### Scenario 3: Generalizing a Specific Lesson

**Example:** "Path Options" → "Any Options"

1. Update quick-reference title and description
2. Rewrite deep-dive section to show general pattern first
3. Keep specific case (Path) as "Common Application" subsection
4. Update cross-references
5. Done!

---

## 📝 Writing Guidelines

### Do's:
- ✅ Use real examples from the codebase
- ✅ Show the WRONG way before the RIGHT way
- ✅ Explain WHY, not just HOW
- ✅ Include compiler errors when relevant
- ✅ Add "Why this works" explanations
- ✅ Keep quick-reference entries SHORT (20-30 lines)
- ✅ Make deep-dives COMPREHENSIVE (100-200 lines)
- ✅ Use emojis sparingly for visual markers (❌ ✅ ⚠️)

### Don'ts:
- ❌ Don't duplicate content between quick-reference and deep-dive
- ❌ Don't add lessons without real code examples
- ❌ Don't create new categories unnecessarily
- ❌ Don't break existing links
- ❌ Don't use overly complex examples
- ❌ Don't skip the "Golden Rule" summary

---

## 🎨 Style Guide

### Code Examples

**In quick-reference:**
```rust
// Keep it SHORT - 1-3 lines max
// ❌ BAD
let x = vec.get(0).unwrap();

// ✅ GOOD
let x = vec.get(0).expect("vec is never empty");
```

**In deep-dives:**
```rust
// Show MORE CONTEXT - realistic scenarios
// ❌ WRONG - Assumes vec is non-empty
fn process_first_item(vec: &[String]) -> Result<()> {
    let first = vec.get(0).unwrap();  // Panics on empty vec!
    println!("First: {}", first);
    Ok(())
}

// ✅ CORRECT - Handles empty vec gracefully
fn process_first_item(vec: &[String]) -> Result<()> {
    let first = vec.get(0)
        .ok_or_else(|| anyhow!("Vector is empty"))?;
    println!("First: {}", first);
    Ok(())
}
```

### Headers

Use consistent header levels:
- `##` for main lesson number/title
- `###` for subsections (Problem, Solution, etc.)
- `####` for sub-subsections (rarely needed)

### Links

**Internal links:**
```markdown
[Link text](file.md#anchor-name)
```

**Anchors are auto-generated** from headers by GitHub:
- "## 3. Using Enums" → `#3-using-enums`
- Lowercase, spaces → hyphens, special chars removed

---

## ✅ Review Checklist

Before submitting your new lesson:

**Content:**
- [ ] Real-world example from codebase (with file:line reference)
- [ ] Clear explanation of the problem
- [ ] Clear explanation of the solution
- [ ] "Why this works" reasoning
- [ ] "Golden Rule" one-sentence summary

**Structure:**
- [ ] Added to quick-reference.md with correct number
- [ ] Added to appropriate deep-dive guide
- [ ] Cross-references added to related topics
- [ ] All links tested and working

**Quality:**
- [ ] Examples are concise (quick-ref) and comprehensive (deep-dive)
- [ ] No duplicate content across files
- [ ] Consistent formatting with existing lessons
- [ ] Grammar and spelling checked

---

## 🤝 Questions?

If you're unsure about:
- **Which category?** → Choose the most specific one, add cross-refs
- **Too long?** → Break into multiple lessons
- **Too short?** → Might be a note in an existing lesson instead
- **New category?** → Discuss first - very rare

**Remember:** It's better to add to an existing category than create a new one. The structure is intentionally flat to keep navigation simple.

---

**Last Updated:** 2025-11-01
**Document Version:** 2.0

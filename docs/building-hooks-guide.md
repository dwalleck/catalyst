# Building Claude Code Hooks: A Complete Guide

**Based on real-world experience building the cargo_check PostToolUse hook**

This guide captures lessons learned, best practices, and common pitfalls from building production-quality hooks for Claude Code.

---

## Table of Contents

1. [Hook Fundamentals](#hook-fundamentals)
2. [JSON Output Structure](#json-output-structure)
3. [Exit Codes and Their Meanings](#exit-codes-and-their-meanings)
4. [Testing Strategies](#testing-strategies)
5. [Cross-Platform Considerations](#cross-platform-considerations)
6. [Performance Optimization](#performance-optimization)
7. [Common Pitfalls](#common-pitfalls)
8. [Complete Example](#complete-example)

---

## Hook Fundamentals

### What Are Hooks?

Hooks are executables that run at specific points in Claude Code's workflow:

- **SessionStart**: When a new session begins
- **UserPromptSubmit**: Before processing user input
- **PreToolUse**: Before executing a tool (Read, Write, Edit, etc.)
- **PostToolUse**: After a tool executes successfully
- **Stop/SubagentStop**: Before stopping execution

### Choosing the Right Hook Type

**PostToolUse** (our example):
- ✅ Validate code after edits
- ✅ Run linters/formatters
- ✅ Check compilation
- ❌ Cannot prevent the edit (already happened)

**PreToolUse**:
- ✅ Can block dangerous operations
- ✅ Can modify tool parameters
- ❌ Cannot see results

**Key Decision**: Use PostToolUse when you want to validate results and provide feedback. Use PreToolUse when you need to prevent or modify actions.

---

## JSON Output Structure

### The Critical Lesson

**We spent hours debugging because we used the wrong JSON structure!** Here's what we learned:

### ❌ WRONG (What We Initially Did)

```json
{
  "decision": "block",
  "reasoning": "Compilation failed",
  "additionalContext": "error output here"
}
```

**Problems:**
- Field name is `reasoning` not `reason` ❌
- `additionalContext` is at wrong level ❌
- Missing `hookEventName` ❌
- Claude Code couldn't parse it ❌

### ✅ CORRECT (Official API)

```json
{
  "decision": "block",
  "reason": "Compilation failed - code contains errors",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "Full error output with line numbers..."
  },
  "systemMessage": "Cargo check found compilation errors"
}
```

**Why this matters:**
- `reason`: Shown to Claude (the AI)
- `additionalContext`: Also shown to Claude (full details)
- `systemMessage`: Shown to the user as a warning
- Proper nesting enables Claude Code to parse and display correctly

### Complete Field Reference

```rust
#[derive(Serialize)]
struct HookResponse {
    decision: String,              // "block" or omit
    reason: String,                // Why are we blocking

    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: HookSpecificOutput,

    #[serde(rename = "systemMessage", skip_serializing_if = "Option::is_none")]
    system_message: Option<String>,  // User-visible warning
}

#[derive(Serialize)]
struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    hook_event_name: String,       // "PostToolUse"

    #[serde(rename = "additionalContext")]
    additional_context: String,    // Full details for Claude
}
```

### Common Fields (All Hook Types)

```json
{
  "continue": false,        // Stop execution entirely
  "stopReason": "message",  // Shown to user when stopped
  "suppressOutput": true    // Hide from transcript (Ctrl-R)
}
```

---

## Exit Codes and Their Meanings

### The Exit Code Discovery

We initially used exit code 1, then discovered exit code 2 shows stderr to Claude!

### Exit Code Matrix

| Code | Behavior | Use Case |
|------|----------|----------|
| **0** | Success | Everything is fine, continue |
| **2** | Show stderr to MODEL | Compilation errors Claude should fix |
| **1+** | Show stderr to USER only | Non-blocking warnings for user |

### Important: JSON vs Exit Codes

**With proper JSON output:**
- Always exit with code **0**
- The `decision: "block"` field controls blocking
- stderr is ignored (JSON provides the context)

**Before we learned this:**
```rust
// OLD WAY (before JSON)
if compilation_failed {
    eprintln!("{}", errors);
    std::process::exit(2);  // Hope Claude sees stderr
}
```

**After learning JSON:**
```rust
// NEW WAY (with JSON)
if compilation_failed {
    let response = HookResponse {
        decision: "block".to_string(),
        reason: "Compilation failed".to_string(),
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "PostToolUse".to_string(),
            additional_context: errors,
        },
        system_message: Some("Found compilation errors".to_string()),
    };
    println!("{}", serde_json::to_string_pretty(&response)?);
    std::process::exit(0);  // Always 0 with JSON!
}
```

---

## Testing Strategies

### Lesson: Test Race Conditions Are Real

**Initial approach (WRONG):**
```rust
let temp_dir = std::env::temp_dir().join("my_test");
fs::create_dir_all(&temp_dir).unwrap();
// ... test code ...
fs::remove_dir_all(temp_dir).unwrap();  // Manual cleanup
```

**Problems:**
- Tests use same path → collisions when running in parallel
- If test panics, cleanup doesn't happen
- Leaves garbage in /tmp

**Solution: Use `tempfile` crate:**
```rust
use tempfile::TempDir;

#[test]
fn my_test() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("file.txt");
    // ... test code ...
    // Automatic cleanup on drop!
}
```

### Testing JSON Parsing

**Critical test: Verify you parse MultiEdit correctly**

```rust
#[test]
fn test_multiedit_tool_handling() {
    let multiedit_json = r#"{
        "session_id": "test",
        "tool_name": "MultiEdit",
        "tool_input": {
            "edits": [
                {"file_path": "src/main.rs", ...},
                {"file_path": "src/lib.rs", ...},
                {"file_path": "README.md", ...}
            ]
        }
    }"#;

    let input: HookInput = serde_json::from_str(multiedit_json).unwrap();

    // Verify you only process .rs files
    let rust_files = extract_rust_files(&input);
    assert_eq!(rust_files.len(), 2);  // Not 3!
}
```

### Unit Test Coverage Checklist

✅ Parse single file (Edit/Write tools)
✅ Parse multiple files (MultiEdit tool)
✅ Handle non-relevant file extensions
✅ Handle missing/invalid JSON fields
✅ Test error paths (what if cargo isn't installed?)
✅ Test workspace vs package detection
✅ Test edge cases (empty paths, relative paths)

---

## Cross-Platform Considerations

### The Quiet Mode Inconsistency Bug

**We had different defaults on different platforms!**

```bash
# Linux/macOS wrapper
export CARGO_CHECK_QUIET="${CARGO_CHECK_QUIET:-false}"  # Good!
```

```powershell
# Windows wrapper (WRONG initially)
if (-not $env:CARGO_CHECK_QUIET) {
    $env:CARGO_CHECK_QUIET = "true"  # Different default!
}
```

**Lesson:** Test your wrapper scripts on ALL platforms. Defaults must match!

### Wrapper Script Best Practices

**Linux/macOS (.sh):**
```bash
#!/bin/bash

# 1. Set PATH to find cargo
export PATH="$HOME/.cargo/bin:$PATH"

# 2. Consistent defaults
export CARGO_CHECK_QUIET="${CARGO_CHECK_QUIET:-false}"

# 3. Optional debug logging (opt-in!)
if [ "${CARGO_CHECK_DEBUG:-false}" = "true" ]; then
    echo "$(date) - Hook executed" >> /tmp/hook.log
fi

# 4. Check binary exists
BINARY="$HOME/.claude-hooks/bin/my-hook"
if [ ! -x "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY" >&2
    exit 1
fi

# 5. Run and preserve exit code
cat | "$BINARY"
exit $?
```

**Windows (.ps1):**
```powershell
# 1. Set PATH
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

# 2. Consistent defaults
if (-not $env:CARGO_CHECK_QUIET) {
    $env:CARGO_CHECK_QUIET = "false"
}

# 3. Check binary exists
$Binary = "$env:USERPROFILE\.claude-hooks\bin\my-hook.exe"
if (-not (Test-Path $Binary)) {
    Write-Error "Error: Binary not found at $Binary"
    exit 1
}

# 4. Run and preserve exit code
$input | & $Binary
exit $LASTEXITCODE
```

---

## Performance Optimization

### 1. Thread-Safe Output Capture

**Problem:** Reading stdout then stderr sequentially can deadlock if pipe buffer fills.

**Solution:** Read concurrently with threads.

```rust
// Spawn threads to read both streams
let stdout_thread = std::thread::spawn(move || {
    BufReader::new(stdout)
        .lines()
        .map_while(Result::ok)
        .collect::<Vec<_>>()
});

let stderr_thread = std::thread::spawn(move || {
    BufReader::new(stderr)
        .lines()
        .map_while(Result::ok)
        .collect::<Vec<_>>()
});

// Join and handle panics gracefully
let stdout_lines = stdout_thread.join().unwrap_or_else(|_| {
    eprintln!("Warning: stdout thread panicked");
    Vec::new()
});

let stderr_lines = stderr_thread.join().unwrap_or_else(|_| {
    eprintln!("Warning: stderr thread panicked");
    Vec::new()
});
```

### 2. Efficient String Building

**Before:**
```rust
for line in lines {
    output.push_str(&line);
    output.push('\n');  // Two allocations per line!
}
```

**After:**
```rust
use std::fmt::Write;

for line in lines {
    writeln!(output, "{}", line).unwrap();  // One allocation
}
```

### 3. Deduplication

**If processing multiple files, deduplicate expensive operations:**

```rust
use std::collections::HashSet;

let mut processed_roots = HashSet::new();

for file in edited_files {
    let root = find_root(&file)?;

    // Only run check once per root
    if processed_roots.insert(root.clone()) {
        run_expensive_check(&root)?;
    }
}
```

---

## Common Pitfalls

### 1. ❌ Debug Logging in Production

**WRONG:**
```bash
echo "$(date) - Hook executed" >> /tmp/hook.log  # Always logs!
```

**RIGHT:**
```bash
if [ "${MY_HOOK_DEBUG:-false}" = "true" ]; then
    echo "$(date) - Hook executed" >> /tmp/hook.log
fi
```

### 2. ❌ Using .expect() in Production

**WRONG:**
```rust
let output = stdout_thread.join().expect("thread panicked");
```

**RIGHT:**
```rust
let output = stdout_thread.join().unwrap_or_else(|_| {
    eprintln!("Warning: thread panicked, output may be incomplete");
    Vec::new()
});
```

### 3. ❌ Hardcoded Paths

**WRONG:**
```rust
let config = fs::read_to_string("/home/user/.config/my-hook.toml")?;
```

**RIGHT:**
```rust
use std::env;

let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
let config_path = PathBuf::from(home).join(".config/my-hook.toml");
```

### 4. ❌ Silent Failures

**WRONG:**
```rust
if let Ok(json) = serde_json::to_string(&response) {
    println!("{}", json);
}
// Falls through silently if serialization fails!
```

**RIGHT:**
```rust
let json = serde_json::to_string_pretty(&response)
    .expect("Failed to serialize response - this is a bug");
println!("{}", json);
```

### 5. ❌ Assuming Tool Names

**WRONG:**
```rust
if tool_name == "Edit" {
    let file = tool_input["file_path"].as_str().unwrap();
}
```

**RIGHT:**
```rust
match tool_name.as_str() {
    "Edit" | "Write" => {
        if let Some(file) = tool_input.get("file_path").and_then(|v| v.as_str()) {
            // Process single file
        }
    }
    "MultiEdit" => {
        if let Some(edits) = tool_input.get("edits").and_then(|v| v.as_array()) {
            // Process multiple files
        }
    }
    _ => return Ok(None), // Not a tool we care about
}
```

---

## Complete Example

### Full Rust Hook Implementation

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read};

#[derive(Deserialize)]
struct HookInput {
    #[serde(rename = "session_id")]
    _session_id: String,
    tool_name: Option<String>,
    tool_input: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize)]
struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    hook_event_name: String,
    #[serde(rename = "additionalContext")]
    additional_context: String,
}

#[derive(Serialize)]
struct HookResponse {
    decision: String,
    reason: String,
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: HookSpecificOutput,
    #[serde(rename = "systemMessage", skip_serializing_if = "Option::is_none")]
    system_message: Option<String>,
}

fn main() {
    match run() {
        Ok(Some(response)) => {
            // Output JSON and exit successfully
            let json = serde_json::to_string_pretty(&response)
                .expect("Failed to serialize response");
            println!("{}", json);
            std::process::exit(0);
        }
        Ok(None) => {
            // Success, no output needed
            std::process::exit(0);
        }
        Err(e) => {
            // Hook error - report to Claude
            let response = HookResponse {
                decision: "block".to_string(),
                reason: format!("Hook error: {}", e),
                hook_specific_output: HookSpecificOutput {
                    hook_event_name: "PostToolUse".to_string(),
                    additional_context: "Internal hook error occurred".to_string(),
                },
                system_message: Some("Hook encountered an error".to_string()),
            };

            let json = serde_json::to_string_pretty(&response)
                .expect("Failed to serialize error response");
            println!("{}", json);
            std::process::exit(0);
        }
    }
}

fn run() -> Result<Option<HookResponse>, Box<dyn std::error::Error>> {
    // Read JSON from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Parse input
    let input: HookInput = serde_json::from_str(&buffer)?;

    // Check if relevant tool
    let tool_name = match input.tool_name {
        Some(name) => name,
        None => return Ok(None),
    };

    if !matches!(tool_name.as_str(), "Edit" | "Write" | "MultiEdit") {
        return Ok(None);
    }

    // Extract files and validate
    let files = extract_files(&input)?;

    if files.is_empty() {
        return Ok(None);
    }

    // Run your validation logic
    let validation_result = validate_files(&files)?;

    if !validation_result.success {
        // Return block response with details
        Ok(Some(HookResponse {
            decision: "block".to_string(),
            reason: validation_result.summary,
            hook_specific_output: HookSpecificOutput {
                hook_event_name: "PostToolUse".to_string(),
                additional_context: validation_result.details,
            },
            system_message: Some("Validation failed".to_string()),
        }))
    } else {
        // All good, no output
        Ok(None)
    }
}
```

---

## Quick Start Checklist

When building a new hook:

- [ ] Choose correct hook type (PostToolUse, PreToolUse, etc.)
- [ ] Use correct JSON structure (`reason`, `hookSpecificOutput`, etc.)
- [ ] Exit with code 0 (JSON controls blocking)
- [ ] Handle Edit, Write, AND MultiEdit tools
- [ ] Use `tempfile` crate for tests
- [ ] Test on Linux, macOS, AND Windows
- [ ] Make debug logging opt-in
- [ ] Use `.unwrap_or_else()` not `.expect()` for graceful degradation
- [ ] Deduplicate expensive operations
- [ ] Add comprehensive test coverage
- [ ] Provide wrapper scripts for both .sh and .ps1

---

## References

- [Claude Code Hooks Documentation](https://docs.claude.com/en/docs/claude-code/hooks)
- [PostToolUse Decision Control](https://docs.claude.com/en/docs/claude-code/hooks#posttooluse-decision-control)
- [Common JSON Fields](https://docs.claude.com/en/docs/claude-code/hooks#common-json-fields)

---

## Credits

This guide is based on real-world experience building the `cargo_check` hook for automatic Rust compilation checking. All pitfalls, solutions, and best practices come from actual debugging sessions and PR reviews.

**Key lesson:** Read the official documentation carefully! We spent hours debugging because we used `reasoning` instead of `reason`. The correct JSON structure is critical for Claude Code integration.

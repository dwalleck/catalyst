# Cargo Check Hook

**Purpose:** Automatically runs `cargo check` (and optionally clippy, tests, formatting) when you edit Rust files in Claude Code.

**Type:** PostToolUse hook (runs after Edit/Write/MultiEdit tools complete)

---

## Features

✅ **Automatic workspace/package detection** - Finds Cargo.toml and runs appropriate command
✅ **Cross-platform** - Works on Linux, macOS, and Windows
✅ **Zero external dependencies** - Pure Rust, no jq or bash required
✅ **Structured JSON output** - Provides compilation errors directly to the AI
✅ **Optional checks** - Enable clippy, tests, or formatting via environment variables
✅ **Intelligent blocking** - Uses Claude Code's JSON hook API to block on compilation failures

---

## Installation

### 1. Install Catalyst Hooks

**Linux / macOS:**
```bash
cd /path/to/catalyst
./install.sh
```

**Windows:**
```powershell
cd C:\path\to\catalyst
.\install.ps1
```

This installs `cargo-check` to `~/.claude-hooks/bin/` (or `%USERPROFILE%\.claude-hooks\bin\` on Windows).

### 2. Copy Hook Wrapper to Your Project

**Linux / macOS:**
```bash
cd your-rust-project/.claude/hooks/
cp /path/to/catalyst/.claude/hooks/cargo-check-stop.sh .
chmod +x cargo-check-stop.sh
```

**Windows:**
```powershell
cd your-rust-project\.claude\hooks\
Copy-Item C:\path\to\catalyst\.claude\hooks\cargo-check-stop.ps1 .
```

### 3. Configure settings.json

Add to your project's `.claude/settings.json`:

**Linux / macOS:**
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/cargo-check-stop.sh"
          }
        ]
      }
    ]
  }
}
```

**Windows:**
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/cargo-check-stop.ps1"
          }
        ]
      }
    ]
  }
}
```

---

## How It Works

1. **Trigger:** Runs after Edit, Write, or MultiEdit tools complete
2. **Filter:** Only processes `.rs` files
3. **Find Root:** Walks up directory tree to find:
   - Workspace root (if `[workspace]` in Cargo.toml)
   - Package root (if standard Cargo.toml)
4. **Run Check:** Executes `cargo check` (with `--workspace` for workspaces)
5. **Capture Output:** Collects all compilation errors/warnings
6. **Block on Failure:** Returns JSON response with `decision: "block"` if compilation fails

### JSON Output Format

When checks fail, the hook returns a structured JSON response to Claude Code:

```json
{
  "decision": "block",
  "reason": "Rust compilation checks failed - code contains errors that must be fixed before proceeding",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "<full cargo output with errors>"
  },
  "systemMessage": "Cargo check found compilation errors - see details below"
}
```

**Important:** The hook always exits with code 0, even on failure. The `decision: "block"` field controls blocking behavior, not the exit code.

This allows Claude to:
- See the compilation errors immediately
- Understand what needs to be fixed
- Automatically suggest corrections

### Exit Code Evolution

This hook evolved through several iterations:

**v1 (Exit Code Strategy):**
- Failed checks → exit 2 (shows stderr to AI)
- Problem: Inconsistent output handling, no structured data

**v2 (JSON Strategy - Current):**
- Always exit 0
- Use JSON `decision: "block"` for blocking
- Benefits:
  - Structured error data
  - Consistent output formatting
  - User and AI see same information
  - Better integration with Claude Code's hook API

**Why Exit 0?**

The PostToolUse hook API expects:
- Exit 0 = Hook executed successfully (check the JSON for results)
- Exit non-zero = Hook itself failed (internal error)

By using JSON with exit 0, we separate "hook execution status" from "check results"

---

## Optional Checks

Enable additional checks via environment variables.

**Accepted values:** `1`, `true`, `yes`, `on` (case-insensitive)

### Clippy (Linting)

**Linux / macOS:**
```bash
export CARGO_CHECK_CLIPPY=true   # or 1, yes, on
```

**Windows:**
```powershell
$env:CARGO_CHECK_CLIPPY = "true"   # or 1, yes, on
```

Runs: `cargo clippy --workspace -- -D warnings`

### Test Compilation

**Linux / macOS:**
```bash
export CARGO_CHECK_TESTS=yes   # or 1, true, on
```

**Windows:**
```powershell
$env:CARGO_CHECK_TESTS = "yes"   # or 1, true, on
```

Runs: `cargo test --workspace --no-run`

### Formatting Check

**Linux / macOS:**
```bash
export CARGO_CHECK_FMT=on   # or 1, true, yes
```

**Windows:**
```powershell
$env:CARGO_CHECK_FMT = "on"   # or 1, true, yes
```

Runs: `cargo fmt --all -- --check`

### Enable All Checks

**Linux / macOS:**
```bash
export CARGO_CHECK_CLIPPY=true
export CARGO_CHECK_TESTS=true
export CARGO_CHECK_FMT=true
```

**Windows:**
```powershell
$env:CARGO_CHECK_CLIPPY = "true"
$env:CARGO_CHECK_TESTS = "true"
$env:CARGO_CHECK_FMT = "true"
```

---

## Example Output

### Success (No Output)

When all checks pass, the hook exits silently with no output. Claude Code continues normally.

### Compilation Error (JSON Response)

When checks fail, the hook returns a JSON response that Claude Code displays to you and feeds back to the AI:

```json
{
  "decision": "block",
  "reason": "Rust compilation checks failed - code contains errors that must be fixed before proceeding",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "🦀 Running check on workspace...\nerror[E0425]: cannot find value `foo` in this scope\n  --> catalyst-cli/src/bin/example.rs:10:9\n   |\n10 |         foo\n   |         ^^^ not found in this scope\n\nerror: could not compile `catalyst-cli` (bin \"example\")\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n❌ Cargo check failed with exit code 101\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
  },
  "systemMessage": "Cargo check found compilation errors - see details below"
}
```

The AI receives this information and can:
- See exactly what errors occurred
- Understand the compilation failure
- Suggest fixes or automatically correct the code

### Multiple Check Failures

If multiple checks are enabled and fail, all output is accumulated:

```json
{
  "decision": "block",
  "reason": "Rust compilation checks failed - code contains errors that must be fixed before proceeding",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "🦀 Running check on workspace...\n<cargo check errors>\n\n📎 Running clippy on workspace...\n<clippy warnings>\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n❌ Cargo clippy failed with exit code 101\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
  },
  "systemMessage": "Cargo check found compilation errors - see details below"
}
```

**Note on Output Size:** The hook limits output to 50KB to prevent overwhelming Claude with massive error output from very large workspaces. If output is truncated, focus on fixing the first few errors shown.

---

## Troubleshooting

### Hook doesn't run

**Check:**
1. Is `~/.claude-hooks/bin/cargo-check` installed?
   ```bash
   ls -la ~/.claude-hooks/bin/cargo-check
   ```
2. Is the wrapper script executable?
   ```bash
   ls -la .claude/hooks/cargo-check-stop.sh
   ```
3. Is `settings.json` valid JSON?
   ```bash
   cat .claude/settings.json | jq .
   ```

### "Cargo.toml not found" error

The hook walks up from the edited file to find Cargo.toml. Make sure:
- You're editing a file inside a Cargo project
- The file path is correct
- Cargo.toml exists in a parent directory

### Permission denied (Windows)

If you get "script execution is disabled", run:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

## Performance

| Project Size | Cargo Check Time | Hook Overhead |
|--------------|------------------|---------------|
| Small (<10 crates) | ~0.1-0.5s | ~2ms |
| Medium (10-50 crates) | ~0.5-2s | ~2ms |
| Large (50+ crates) | ~2-10s | ~2ms |

The hook adds negligible overhead (~2ms startup time). Most time is spent in cargo check itself.

---

## Comparison to Shell Script

| Feature | Rust Hook | Shell Script |
|---------|-----------|--------------|
| **Cross-platform** | ✅ Linux/macOS/Windows | ❌ Linux/macOS only |
| **Dependencies** | ✅ None | ❌ Requires `jq` |
| **Performance** | ✅ 2ms startup | ⚠️ ~50ms startup |
| **Type Safety** | ✅ Compile-time | ❌ Runtime |
| **Error Handling** | ✅ Structured errors | ⚠️ Basic |
| **Maintainability** | ✅ Easy to test | ⚠️ Harder to test |

---

## Integration with CI/CD

This hook enforces the same checks you'd run in CI:

```yaml
# .github/workflows/rust.yml
jobs:
  test:
    steps:
      - run: cargo check --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace --no-run
      - run: cargo fmt --all -- --check
```

Enable all checks locally to match CI:
```bash
export CARGO_CHECK_CLIPPY=1
export CARGO_CHECK_TESTS=1
export CARGO_CHECK_FMT=1
```

---

## FAQ

**Q: Should I use this for large projects?**
A: Yes! The hook is fast (2ms overhead) and only runs on .rs file edits.

**Q: Can I disable it temporarily?**
A: Yes, comment out the hook in settings.json or rename the wrapper script.

**Q: Does this work with monorepos?**
A: Yes! It detects workspaces and runs `cargo check --workspace`.

**Q: What if I have multiple Rust projects in one directory?**
A: The hook finds the closest Cargo.toml (workspace > package).

**Q: Can I customize the cargo commands?**
A: Yes! Edit the `cargo-check` binary source and rebuild with:
```bash
cargo build --release --bin cargo-check
cp target/release/cargo-check ~/.claude-hooks/bin/
```

---

## Advanced: Custom Checks

To add custom checks, edit `catalyst-cli/src/bin/cargo_check.rs`:

```rust
// Add after existing checks
if env::var("CARGO_CHECK_CUSTOM").unwrap_or_default() == "1" {
    run_cargo_command(
        cargo_root,
        "your-custom-command",
        &["--your-args"],
        "🔧",
        "✅ Custom check passed",
    )?;
}
```

Then rebuild and reinstall:
```bash
cargo build --release --bin cargo-check
cp target/release/cargo-check ~/.claude-hooks/bin/
```

---

## See Also

- [Rust Hooks Documentation](../docs/rust-hooks.md)
- [Standalone Installation](../docs/standalone-installation.md)
- [Catalyst Hook Architecture](../CLAUDE.md#rust-hook-implementation)

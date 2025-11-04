#!/bin/bash
# Cargo Check Hook - Runs cargo check on .rs file edits
#
# This hook automatically runs cargo check when you edit Rust files.
# It finds the workspace or package root and runs the appropriate command.
#
# Hook Type: PostToolUse (runs after Edit/Write/MultiEdit tools)
#
# Optional environment variables (accepts: 1, true, yes, on):
#   CARGO_CHECK_CLIPPY=true    - Also run clippy with -D warnings
#   CARGO_CHECK_TESTS=yes      - Also check test compilation (--no-run)
#   CARGO_CHECK_FMT=on         - Also check formatting (--check)
#   CARGO_CHECK_QUIET=false    - Show all output (default: true for silent mode)
#
# Example settings.json configuration:
#   "PostToolUse": [
#     {
#       "hooks": [
#         {
#           "type": "command",
#           "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/cargo-check-stop.sh"
#         }
#       ]
#     }
#   ]

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# IMPORTANT: Claude Code does NOT display hook stderr in the UI
# This means quiet mode hides ALL output including errors!
# Running with quiet=false so cargo output goes to logs
# Set CARGO_CHECK_QUIET=true to enable quiet mode (but you won't see errors!)
export CARGO_CHECK_QUIET="${CARGO_CHECK_QUIET:-false}"

# Log hook execution for debugging (optional - comment out when not needed)
echo "$(date '+%Y-%m-%d %H:%M:%S') - cargo-check hook executed" >> /tmp/cargo-check.log

# Check if cargo-check binary exists
CARGO_CHECK_BIN="$HOME/.claude-hooks/bin/cargo-check"
if [ ! -x "$CARGO_CHECK_BIN" ]; then
    echo "Error: cargo-check binary not found at $CARGO_CHECK_BIN" >&2
    echo "Please run ./install.sh from the catalyst repository" >&2
    exit 1
fi

# Run cargo-check and explicitly exit with its exit code
cat | "$CARGO_CHECK_BIN"
exit $?

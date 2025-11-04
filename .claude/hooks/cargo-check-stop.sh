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

# Run in quiet mode by default (only show output on errors)
# Set CARGO_CHECK_QUIET=0 to see all output
export CARGO_CHECK_QUIET="${CARGO_CHECK_QUIET:-true}"

# Run cargo-check and explicitly exit with its exit code
cat | ~/.claude-hooks/bin/cargo-check
exit $?

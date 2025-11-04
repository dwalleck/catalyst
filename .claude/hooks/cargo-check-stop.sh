#!/bin/bash
# Cargo Check Hook - Runs cargo check on .rs file edits
#
# This hook automatically runs cargo check when you edit Rust files.
# It finds the workspace or package root and runs the appropriate command.
#
# Optional environment variables (accepts: 1, true, yes, on):
#   CARGO_CHECK_CLIPPY=true    - Also run clippy with -D warnings
#   CARGO_CHECK_TESTS=yes      - Also check test compilation (--no-run)
#   CARGO_CHECK_FMT=on         - Also check formatting (--check)
#
# Example settings.json configuration:
#   "Stop": [
#     {
#       "hooks": [
#         {
#           "type": "command",
#           "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/cargo-check-stop.sh"
#         }
#       ]
#     }
#   ]

cat | ~/.claude-hooks/bin/cargo-check

# Cargo Check Hook - Runs cargo check on .rs file edits
#
# This hook automatically runs cargo check when you edit Rust files.
# It finds the workspace or package root and runs the appropriate command.
#
# Optional environment variables:
#   $env:CARGO_CHECK_CLIPPY=1    - Also run clippy with -D warnings
#   $env:CARGO_CHECK_TESTS=1     - Also check test compilation (--no-run)
#   $env:CARGO_CHECK_FMT=1       - Also check formatting (--check)
#
# Example settings.json configuration:
#   "Stop": [
#     {
#       "hooks": [
#         {
#           "type": "command",
#           "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/cargo-check-stop.ps1"
#         }
#       ]
#     }
#   ]

$input | & "$env:USERPROFILE\.claude-hooks\bin\cargo-check.exe"

# Cargo Check Hook - Runs cargo check on .rs file edits
#
# This hook automatically runs cargo check when you edit Rust files.
# It finds the workspace or package root and runs the appropriate command.
#
# Optional environment variables (accepts: 1, true, yes, on):
#   $env:CARGO_CHECK_CLIPPY="true"    - Also run clippy with -D warnings
#   $env:CARGO_CHECK_TESTS="yes"      - Also check test compilation (--no-run)
#   $env:CARGO_CHECK_FMT="on"         - Also check formatting (--check)
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

# Ensure cargo is in PATH
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

# Run in quiet mode by default (only show output on errors)
# Set $env:CARGO_CHECK_QUIET="0" to see all output
if (-not $env:CARGO_CHECK_QUIET) {
    $env:CARGO_CHECK_QUIET = "true"
}

# Run cargo-check and exit with its exit code
$input | & "$env:USERPROFILE\.claude-hooks\bin\cargo-check.exe"
exit $LASTEXITCODE

# Cargo Check Hook - Runs cargo check on .rs file edits
#
# This hook automatically runs cargo check when you edit Rust files.
# It finds the workspace or package root and runs the appropriate command.
#
# Hook Type: PostToolUse (runs after Edit/Write/MultiEdit tools)
#
# Optional environment variables (accepts: 1, true, yes, on):
#   $env:CARGO_CHECK_CLIPPY="true"    - Also run clippy with -D warnings
#   $env:CARGO_CHECK_TESTS="yes"      - Also check test compilation (--no-run)
#   $env:CARGO_CHECK_FMT="on"         - Also check formatting (--check)
#   $env:CARGO_CHECK_QUIET="true"     - Enable quiet mode (default: false shows all output)
#
# Example settings.json configuration:
#   "PostToolUse": [
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

# Show all output by default for visibility
# Set $env:CARGO_CHECK_QUIET="true" to enable quiet mode
if (-not $env:CARGO_CHECK_QUIET) {
    $env:CARGO_CHECK_QUIET = "false"
}

# Check if cargo-check binary exists
$CargoCheckBin = "$env:USERPROFILE\.claude-hooks\bin\cargo-check.exe"
if (-not (Test-Path $CargoCheckBin)) {
    Write-Error "Error: cargo-check binary not found at $CargoCheckBin"
    Write-Error "Please run .\install.ps1 from the catalyst repository"
    exit 1
}

# Run cargo-check and exit with its exit code
$input | & $CargoCheckBin
exit $LASTEXITCODE

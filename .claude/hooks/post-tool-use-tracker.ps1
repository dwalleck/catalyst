#!/usr/bin/env pwsh
# .claude/hooks/post-tool-use-tracker.ps1
#
# Windows PowerShell wrapper for post-tool-use-tracker hook
# This script reads from stdin and pipes to the Rust binary

# Primary: Use standalone installation
$standaloneExe = "$env:USERPROFILE\.claude-hooks\bin\post-tool-use-tracker-sqlite.exe"

# Fallback: Use project-local binary
$projectExe = "$env:CLAUDE_PROJECT_DIR\target\release\post-tool-use-tracker-sqlite.exe"

if (Test-Path $standaloneExe) {
    # Use standalone installation
    $input | & $standaloneExe
} elseif (Test-Path $projectExe) {
    # Fallback to project-local binary
    $input | & $projectExe
} else {
    # Binary not found - provide helpful error
    Write-Error @"
post-tool-use-tracker-sqlite.exe not found!

Please install the hooks with SQLite support:
  .\install.ps1 -Sqlite

Or build locally:
  cargo build --release --features sqlite
"@
    exit 1
}

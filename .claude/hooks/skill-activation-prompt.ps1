#!/usr/bin/env pwsh
# .claude/hooks/skill-activation-prompt.ps1
#
# Windows PowerShell wrapper for skill-activation-prompt hook
# This script reads from stdin and pipes to the Rust binary

# Primary: Use standalone installation
$standaloneExe = "$env:USERPROFILE\.claude-hooks\bin\skill-activation-prompt.exe"

# Fallback: Use project-local binary
$projectExe = "$env:CLAUDE_PROJECT_DIR\target\release\skill-activation-prompt.exe"

if (Test-Path $standaloneExe) {
    # Use standalone installation
    $input | & $standaloneExe
} elseif (Test-Path $projectExe) {
    # Fallback to project-local binary
    $input | & $projectExe
} else {
    # Binary not found - provide helpful error
    Write-Error @"
skill-activation-prompt.exe not found!

Please install the hooks first:
  .\install.ps1

Or build locally:
  cargo build --release
"@
    exit 1
}

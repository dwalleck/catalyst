# Auto-generated wrapper for {{BINARY_NAME}}
# Created by Catalyst CLI

$BinaryName = "{{BINARY_NAME}}.exe"

# Try to find the binary in standard locations
$BinaryPath = Join-Path $env:USERPROFILE ".claude-hooks\bin\$BinaryName"

# If not found, try local project build
if (-not (Test-Path $BinaryPath)) {
    if ($env:CATALYST_PROJECT_DIR) {
        $BinaryPath = Join-Path $env:CATALYST_PROJECT_DIR "target\release\$BinaryName"
    }

    # If still not found, try relative to this script
    if (-not (Test-Path $BinaryPath)) {
        $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
        $ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..\..")
        $BinaryPath = Join-Path $ProjectRoot "catalyst\target\release\$BinaryName"
    }
}

# Check if binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Error: $BinaryName binary not found"
    Write-Error "Searched locations:"
    Write-Error "  - $env:USERPROFILE\.claude-hooks\bin\$BinaryName"
    Write-Error "  - `$env:CATALYST_PROJECT_DIR\target\release\$BinaryName"
    Write-Error ""
    Write-Error "Please run: .\install.ps1"
    exit 1
}

# Execute the binary, piping stdin through it
$input | & $BinaryPath @args

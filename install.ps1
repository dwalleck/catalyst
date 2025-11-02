#!/usr/bin/env pwsh
# install.ps1 - Windows installation script for Catalyst hooks
#
# For PowerShell best practices and lessons learned while writing this script,
# see: docs/powershell-lessons.md

param(
    [switch]$Sqlite,
    [switch]$Help
)

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host "Catalyst - Claude Code Hooks Installer"
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""

if ($Help) {
    Write-Host "Usage: .\install.ps1 [-Sqlite] [-Help]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Sqlite    Build with SQLite support for state management"
    Write-Host "  -Help      Show this help message"
    Write-Host ""
    exit 0
}

# Check for Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed!"
    Write-Host ""
    Write-Host "Install Rust first:"
    Write-Host "  Visit: https://rustup.rs/"
    Write-Host "  Or run: winget install Rustlang.Rustup"
    Write-Host ""
    exit 1
}

$rustVersion = (cargo --version)
Write-Host "✅ Rust found: $rustVersion"
Write-Host ""

# Detect OS and architecture
$os = [System.Environment]::OSVersion.Platform
$arch = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")

Write-Host "Detected: Windows / $arch"
Write-Host ""

# Installation directory
$installDir = "$env:USERPROFILE\.claude-hooks"
$binDir = "$installDir\bin"
$srcDir = "$installDir\src"

# Create directories
Write-Host "📁 Creating directories..."
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType Directory -Force -Path $srcDir | Out-Null

# Check if we're in the catalyst directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "⚠️  Cargo.toml not found"
    Write-Host "Please run this script from the catalyst repository root"
    exit 1
}

# Copy source code
Write-Host "📦 Copying source code..."
# Note: Copy-Item -Exclude has limitations with directory recursion
# Using Get-ChildItem for more reliable exclusion
$excludeDirs = @("target", ".git", "node_modules")
Get-ChildItem -Path . -Recurse -Force | Where-Object {
    $item = $_
    $shouldExclude = $false
    foreach ($exclude in $excludeDirs) {
        if ($item.FullName -like "*\$exclude\*" -or $item.Name -eq $exclude) {
            $shouldExclude = $true
            break
        }
    }
    -not $shouldExclude
} | ForEach-Object {
    $targetPath = $_.FullName.Replace($PWD.Path, $srcDir)
    if ($_.PSIsContainer) {
        New-Item -ItemType Directory -Path $targetPath -Force | Out-Null
    } else {
        Copy-Item $_.FullName -Destination $targetPath -Force
    }
}
Set-Location $srcDir

# Build release binaries
Write-Host ""
if ($Sqlite) {
    Write-Host "🔨 Building release binaries with SQLite support..."
    cargo build --release --features sqlite
} else {
    Write-Host "🔨 Building release binaries (core only)..."
    Write-Host "   (Use -Sqlite to enable SQLite-backed state management)"
    cargo build --release
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed"
    exit 1
}

# Copy core binaries
Write-Host ""
Write-Host "📦 Installing binaries to $binDir..."

$coreBinaries = @(
    "target\release\skill-activation-prompt.exe",
    "target\release\file-analyzer.exe"
)

foreach ($binary in $coreBinaries) {
    if (Test-Path $binary) {
        $binaryName = Split-Path $binary -Leaf
        Copy-Item $binary "$binDir\$binaryName" -Force
        Write-Host "   ✅ Installed: $binaryName"
    } else {
        $binaryName = Split-Path $binary -Leaf
        Write-Warning "Binary not found: $binaryName (expected at $binary)"
    }
}

# Copy SQLite binaries if built
if ($Sqlite) {
    $sqliteBinary = "target\release\post-tool-use-tracker-sqlite.exe"
    if (Test-Path $sqliteBinary) {
        Copy-Item $sqliteBinary "$binDir\" -Force
        Write-Host "   ✅ Installed: post-tool-use-tracker-sqlite.exe"
    } else {
        Write-Warning "SQLite binary not found: post-tool-use-tracker-sqlite.exe (expected at $sqliteBinary)"
    }
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host "✅ Installation Complete!"
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""
Write-Host "Binaries installed to: $binDir"
Write-Host ""
Write-Host "Next steps:"
Write-Host ""
Write-Host "1. (Optional) Add to your PATH:"
Write-Host "   `$env:PATH += `;$env:USERPROFILE\.claude-hooks\bin`"
Write-Host "   Or add permanently via System Properties > Environment Variables"
Write-Host ""
Write-Host "2. In your Claude Code project, copy wrapper scripts:"
Write-Host ""
Write-Host "   cd your-project\.claude\hooks\"
Write-Host ""
Write-Host "   # Copy PowerShell wrappers from catalyst"
Write-Host "   Copy-Item catalyst\.claude\hooks\*.ps1 ."
Write-Host ""
Write-Host "3. Configure .claude\settings.json:"
Write-Host "   See docs\standalone-installation.md for details"
Write-Host ""
Write-Host "📚 Documentation:"
Write-Host "   - README.md                       - Getting started"
Write-Host "   - docs\standalone-installation.md - Full setup guide"
Write-Host "   - docs\databases.md               - SQLite state management"
Write-Host ""

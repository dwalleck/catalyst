# PowerShell Lessons Learned

Best practices and common pitfalls discovered during PowerShell script development for the Catalyst hooks system.

---

## Overview

This document captures real-world PowerShell issues encountered during development, code reviews, and cross-platform testing. Each lesson includes:

- **The Problem** - What went wrong or could go wrong
- **Why It Matters** - Impact and consequences
- **The Solution** - How to fix it properly
- **Example Code** - Before and after comparisons

**Source:** Catalyst Phase 2.7 (Windows Support) development and PR reviews

---

## Lesson 1: Copy-Item -Exclude Directory Recursion

**Category:** File Operations
**Severity:** Medium
**Discovered:** Phase 2.7 PR Review (#10)

### The Problem

`Copy-Item` with `-Exclude` parameter has known limitations when used with `-Recurse`. On some PowerShell versions, excluded directories may still be copied, leading to:

- Large directories (like `target/`, `.git/`, `node_modules/`) being copied unnecessarily
- Longer copy times
- Increased disk usage
- Potential errors from copying locked files

### Why It Matters

**Performance Impact:**
- Copying `target/` directory: ~2GB of Rust build artifacts
- Copying `.git/` directory: Full repository history
- Copying `node_modules/`: Thousands of files

**Real Numbers (Catalyst Repository):**
- Without exclusion: ~2.5GB copied, 45+ seconds
- With exclusion: ~50MB copied, 2 seconds

### The Solution

Use `Get-ChildItem` with `Where-Object` filtering for reliable exclusion:

#### ❌ Bad (Unreliable)

```powershell
# May not exclude directories reliably
Copy-Item -Path . -Destination $dest -Recurse -Force -Exclude @("target", ".git", "node_modules")
```

**Problems:**
- `-Exclude` only filters items being copied, not recursion paths
- Directories may still be entered and their contents copied
- Behavior varies across PowerShell versions (5.1 vs 7+)

#### ✅ Good (Reliable)

```powershell
# Explicitly filter during traversal
$excludeDirs = @("target", ".git", "node_modules")

Get-ChildItem -Path . -Recurse -Force | Where-Object {
    $item = $_
    $shouldExclude = $false

    # Check if item or any parent path matches exclusions
    foreach ($exclude in $excludeDirs) {
        if ($item.FullName -like "*\$exclude\*" -or $item.Name -eq $exclude) {
            $shouldExclude = $true
            break
        }
    }

    -not $shouldExclude
} | ForEach-Object {
    # Build target path
    $targetPath = $_.FullName.Replace($PWD.Path, $destPath)

    if ($_.PSIsContainer) {
        # Create directory
        New-Item -ItemType Directory -Path $targetPath -Force | Out-Null
    } else {
        # Copy file
        Copy-Item $_.FullName -Destination $targetPath -Force
    }
}
```

**Benefits:**
- Explicitly controls which items are processed
- Works consistently across PowerShell versions
- Can handle complex exclusion patterns
- Clear and debuggable logic

### Alternative Solutions

#### Using Robocopy (Windows-Specific)

```powershell
# Robocopy has robust exclusion
robocopy . $dest /E /XD target .git node_modules /NFL /NDL /NJH /NJS
```

**Pros:**
- Very fast and reliable
- Built into Windows
- Handles locked files gracefully

**Cons:**
- Windows-only (not cross-platform)
- Different exit codes convention (0-7 are success!)
- Less PowerShell-idiomatic

### References

- **Location:** `install.ps1:65` (Catalyst repository)
- **PR:** #10 (Phase 2.7 - Windows Support)
- **Related Issues:** Copy-Item documentation limitations

---

## Lesson 2: Silent Failures Need Warnings

**Category:** Error Handling & User Experience
**Severity:** Low
**Discovered:** Phase 2.7 PR Review (#10)

### The Problem

When expected files or operations are missing/failed but the script continues, users are left confused:

```powershell
# Silent failure - user doesn't know what happened
if (Test-Path $binary) {
    Copy-Item $binary $dest
}
# Continues regardless...
```

**User Experience:**
1. Script runs and completes "successfully"
2. User tries to use the tool
3. Tool doesn't work (binary missing)
4. User has no idea what went wrong or where

### Why It Matters

**Debugging Time:**
- Without warnings: 10-15 minutes debugging why tool doesn't work
- With warnings: Immediate understanding of the issue

**Real Scenario (install.ps1):**
```powershell
# Build succeeds, but binary missing for unknown reason
cargo build --release --features sqlite

# User doesn't notice the warning in build output
# Script continues silently
# User runs hook, gets "file not found" error
# Has to debug from scratch
```

### The Solution

Use `Write-Warning` for expected conditions that fail:

#### ❌ Bad (Silent)

```powershell
foreach ($binary in $coreBinaries) {
    if (Test-Path $binary) {
        $binaryName = Split-Path $binary -Leaf
        Copy-Item $binary "$binDir\$binaryName" -Force
        Write-Host "   ✅ Installed: $binaryName"
    }
    # Missing binaries are silently skipped!
}
```

#### ✅ Good (Visible)

```powershell
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
```

**Benefits:**
- User immediately sees what's missing
- Shows expected path for troubleshooting
- Distinguishes warnings from errors (script can still succeed)
- Yellow color draws attention without failing

### When to Use Each Stream

PowerShell has multiple output streams for different purposes:

```powershell
# Success messages (green in many terminals)
Write-Host "✅ Installation complete!"

# Informational messages (no special formatting)
Write-Output "Processing file $fileName"

# Warning messages (yellow - something unexpected but not fatal)
Write-Warning "Optional binary not found: $binaryName"

# Error messages (red - actual errors)
Write-Error "Failed to build: $errorMessage"
throw "Fatal error occurred"  # Stops execution
```

**Decision Tree:**

```
Is this expected normal operation?
├─ Yes → Write-Host or Write-Output
└─ No → Is it fatal?
    ├─ Yes → throw or Write-Error
    └─ No → Write-Warning
```

### Warning vs Error Guidelines

**Use Write-Warning when:**
- Optional components are missing
- Deprecated features are used
- Configuration has minor issues
- Fallback behavior is triggered

**Use Write-Error/throw when:**
- Required components are missing
- Operations fail critically
- Cannot continue safely
- Data corruption risk

### Example: Complete Binary Installation

```powershell
# Core binaries (required)
$coreBinaries = @(
    "target\release\skill-activation-prompt.exe",
    "target\release\file-analyzer.exe"
)

$missingCore = @()
foreach ($binary in $coreBinaries) {
    if (Test-Path $binary) {
        Copy-Item $binary "$binDir\" -Force
        Write-Host "   ✅ Installed: $(Split-Path $binary -Leaf)"
    } else {
        $missingCore += $binary
        Write-Warning "Core binary missing: $(Split-Path $binary -Leaf)"
    }
}

# Fail if any core binaries missing
if ($missingCore.Count -gt 0) {
    Write-Error "Installation failed: Missing core binaries"
    throw "Required binaries not found: $($missingCore -join ', ')"
}

# Optional binaries (can warn and continue)
if ($Sqlite) {
    $sqliteBinary = "target\release\post-tool-use-tracker-sqlite.exe"
    if (Test-Path $sqliteBinary) {
        Copy-Item $sqliteBinary "$binDir\" -Force
        Write-Host "   ✅ Installed: post-tool-use-tracker-sqlite.exe"
    } else {
        Write-Warning "SQLite binary not found (expected at $sqliteBinary)"
        Write-Warning "SQLite features will not be available"
    }
}
```

### References

- **Location:** `install.ps1:113-122, 125-133` (Catalyst repository)
- **PR:** #10 (Phase 2.7 - Windows Support)
- **Related:** User experience, error handling

---

## Future Lessons

As we discover more PowerShell patterns and gotchas, they'll be documented here. Topics to cover as we encounter them:

### Planned Topics

- **Execution Policies** - Handling script signing requirements
- **Path Separators** - `\` vs `/` in cross-platform scripts
- **Exit Codes** - Proper use of `$LASTEXITCODE` vs `$?`
- **Parameter Validation** - Using `[ValidateSet]`, `[Parameter]` attributes
- **Error Action Preferences** - When to use `-ErrorAction Stop`
- **Pipeline Variables** - `$_` vs `$PSItem`
- **Quoting and Escaping** - Handling paths with spaces
- **Here-Strings** - `@"..."@` for multi-line strings
- **Splatting** - Using `@params` for clean parameter passing

### How to Contribute

Found a PowerShell gotcha? Add it to this document:

1. **Document the problem** - What went wrong in real usage
2. **Explain why it matters** - Impact and consequences
3. **Show the solution** - Before/after code examples
4. **Reference the source** - Where it was discovered (PR, file, line)

See [Rust Lessons CONTRIBUTING.md](rust-lessons/CONTRIBUTING.md) for format examples.

---

## Related Documentation

- **[Rust Lessons Learned](rust-lessons/index.md)** - Best practices for Rust development
- **[install.ps1](../install.ps1)** - Windows installation script (source of lessons)
- **[Standalone Installation Guide](standalone-installation.md)** - Cross-platform setup

---

## Document Metadata

**Version:** 1.0
**Created:** 2025-11-02 (Phase 2.7)
**Based On:** PR #10 code review feedback
**Platform:** PowerShell 5.1+ (Windows), PowerShell 7+ (cross-platform)

---

**Have a PowerShell lesson to add?** Follow the format above and submit a PR!

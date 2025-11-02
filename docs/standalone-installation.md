# Standalone Rust Hooks Installation

**Recommended approach:** Compile Rust hooks once, use across all projects.

---

## Architecture

Instead of embedding Rust source in each project, install a single binary globally:

### Linux / macOS
```
~/.claude-hooks/                    # Global installation
├── bin/
│   ├── skill-activation-prompt     # Pre-compiled binary
│   ├── file-analyzer              # Pre-compiled binary
│   └── post-tool-tracker          # Pre-compiled binary
└── config/
    └── default-config.json        # Default configuration

~/project-1/.claude/hooks/
└── skill-activation-prompt.sh     # Thin wrapper calling global binary

~/project-2/.claude/hooks/
└── skill-activation-prompt.sh     # Same wrapper, reuses binary
```

### Windows
```
%USERPROFILE%\.claude-hooks\        # Global installation
├── bin\
│   ├── skill-activation-prompt.exe     # Pre-compiled binary
│   ├── file-analyzer.exe              # Pre-compiled binary
│   └── post-tool-tracker-sqlite.exe   # Pre-compiled binary
└── config\
    └── default-config.json            # Default configuration

C:\Users\You\project-1\.claude\hooks\
└── skill-activation-prompt.ps1    # Thin wrapper calling global binary

C:\Users\You\project-2\.claude\hooks\
└── skill-activation-prompt.ps1    # Same wrapper, reuses binary
```

---

## Benefits

### Compile Once, Use Everywhere
```bash
# Build once
cd ~/.claude-hooks/src
cargo build --release

# Copy binaries to bin/
cp target/release/* ~/.claude-hooks/bin/

# Use in any project (just copy wrapper)
```

### Zero Per-Project Build Time
```bash
# Traditional approach
cd project/.claude/hooks/RustHooks
cargo build --release  # 45 seconds per project!

# Standalone approach
cd project/.claude/hooks
./skill-activation-prompt.sh  # Instant!
```

### Centralized Updates
```bash
# Fix bug or add feature once
cd ~/.claude-hooks/src
# make changes
cargo build --release
cp target/release/* ~/.claude-hooks/bin/

# All projects automatically use new version
```

---

## Installation

### Step 1: Build Hooks Globally

#### Linux / macOS
```bash
# Use the automated install script
cd catalyst
./install.sh

# Or with SQLite support
./install.sh --sqlite

# This will:
# - Create ~/.claude-hooks/{bin,src}
# - Build release binaries
# - Copy binaries to ~/.claude-hooks/bin/
```

#### Windows
```powershell
# Use the automated install script
cd catalyst
.\install.ps1

# Or with SQLite support
.\install.ps1 -Sqlite

# This will:
# - Create %USERPROFILE%\.claude-hooks\{bin,src}
# - Build release binaries
# - Copy binaries to %USERPROFILE%\.claude-hooks\bin\
```

### Step 2: Create Per-Project Wrappers

For each project:

#### Linux / macOS
```bash
cd ~/my-project/.claude/hooks

# Create wrapper script
cat > skill-activation-prompt.sh << 'EOF'
#!/bin/bash
set -e

# Call global binary
cat | ~/.claude-hooks/bin/skill-activation-prompt
EOF

chmod +x skill-activation-prompt.sh
```

#### Windows
```powershell
cd C:\Users\You\my-project\.claude\hooks

# Copy PowerShell wrappers from catalyst
Copy-Item C:\path\to\catalyst\.claude\hooks\*.ps1 .

# Or create manually:
@"
#!/usr/bin/env pwsh
`$input | & "`$env:USERPROFILE\.claude-hooks\bin\skill-activation-prompt.exe"
"@ | Out-File -FilePath skill-activation-prompt.ps1 -Encoding UTF8
```

### Step 3: Configure settings.json

#### Linux / macOS
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
          }
        ]
      }
    ]
  }
}
```

#### Windows
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.ps1"
          }
        ]
      }
    ]
  }
}
```

---

## Configuration Approach

### Option 1: Environment Variables

```bash
# In wrapper script
#!/bin/bash
set -e

# Project-specific config via env vars
export SKILL_RULES_PATH="$CLAUDE_PROJECT_DIR/.claude/skills/skill-rules.json"
export PROJECT_TYPE="backend"  # or "frontend"

cat | ~/.claude-hooks/bin/skill-activation-prompt
```

Rust binary reads env vars:
```rust
let rules_path = env::var("SKILL_RULES_PATH")
    .unwrap_or_else(|_| format!("{}/.claude/skills/skill-rules.json",
                                env::var("CLAUDE_PROJECT_DIR").unwrap()));
```

### Option 2: Config File Per Project

```bash
# .claude/hooks/config.json
{
  "skillRulesPath": ".claude/skills/skill-rules.json",
  "projectType": "backend",
  "enableDebug": false
}
```

Wrapper passes config location:
```bash
#!/bin/bash
set -e

cat | ~/.claude-hooks/bin/skill-activation-prompt \
    --config "$CLAUDE_PROJECT_DIR/.claude/hooks/config.json"
```

### Option 3: Command-line Arguments

```bash
#!/bin/bash
set -e

cat | ~/.claude-hooks/bin/skill-activation-prompt \
    --project-dir "$CLAUDE_PROJECT_DIR" \
    --skill-rules ".claude/skills/skill-rules.json"
```

---

## Advanced: Multiple Versions

You can maintain multiple versions:

```
~/.claude-hooks/
├── bin/
│   ├── skill-activation-prompt-v1.0
│   ├── skill-activation-prompt-v2.0     # New version
│   └── skill-activation-prompt -> skill-activation-prompt-v2.0  # Symlink
```

Projects can choose version:
```bash
# Use latest (symlink)
cat | ~/.claude-hooks/bin/skill-activation-prompt

# Pin to specific version
cat | ~/.claude-hooks/bin/skill-activation-prompt-v1.0
```

---

## Distribution Methods

### Method 1: GitHub Releases

```bash
# In your hooks repo
cargo build --release
cd target/release

# Create release artifacts
tar -czf claude-hooks-linux-x64.tar.gz skill-activation-prompt file-analyzer
tar -czf claude-hooks-macos-arm64.tar.gz skill-activation-prompt file-analyzer

# Upload to GitHub releases
gh release create v1.0.0 *.tar.gz
```

**Users install:**
```bash
# Download and extract
wget https://github.com/you/claude-hooks/releases/download/v1.0.0/claude-hooks-linux-x64.tar.gz
tar -xzf claude-hooks-linux-x64.tar.gz -C ~/.claude-hooks/bin/
```

### Method 2: Cargo Install

**If published to crates.io:**
```bash
cargo install claude-hooks --root ~/.claude-hooks

# Binaries installed to ~/.claude-hooks/bin/
```

### Method 3: Installation Script

```bash
# install.sh
#!/bin/bash
set -e

echo "Installing Claude Rust Hooks..."

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Download appropriate binary
RELEASE_URL="https://github.com/you/claude-hooks/releases/latest/download"
TARBALL="claude-hooks-${OS}-${ARCH}.tar.gz"

# Create directory
mkdir -p ~/.claude-hooks/bin

# Download and extract
curl -L "${RELEASE_URL}/${TARBALL}" | tar -xz -C ~/.claude-hooks/bin/

# Make executable
chmod +x ~/.claude-hooks/bin/*

echo "✅ Installation complete!"
echo "Binaries installed to: ~/.claude-hooks/bin/"
```

**Users run:**
```bash
curl -sSL https://your-repo.com/install.sh | bash
```

---

## Cross-Platform Considerations

### Build for Multiple Targets

```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-msvc

# Build for each platform
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc
```

### Platform-Specific Wrappers

**Linux/macOS (Bash):**
```bash
#!/bin/bash
# skill-activation-prompt.sh

# Read from stdin and pipe to Rust binary
cat | ~/.claude-hooks/bin/skill-activation-prompt
```

**Windows (PowerShell):**
```powershell
#!/usr/bin/env pwsh
# skill-activation-prompt.ps1

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
```

**Note:** The Windows wrapper includes fallback logic and helpful error messages, which is especially useful during development or initial setup.

---

## Example Project Structure

### Standalone Hooks Repository

```
claude-hooks-rust/
├── Cargo.toml
├── src/
│   ├── bin/
│   │   ├── skill-activation-prompt.rs
│   │   ├── file-analyzer.rs
│   │   └── post-tool-tracker.rs
│   └── lib.rs                    # Shared code
├── README.md
├── install.sh
└── .github/
    └── workflows/
        └── release.yml           # Auto-build releases
```

### Cargo.toml for Multiple Binaries

```toml
[package]
name = "claude-hooks"
version = "1.0.0"
edition = "2021"

# Multiple binaries in one project
[[bin]]
name = "skill-activation-prompt"
path = "src/bin/skill-activation-prompt.rs"

[[bin]]
name = "file-analyzer"
path = "src/bin/file-analyzer.rs"

[[bin]]
name = "post-tool-tracker"
path = "src/bin/post-tool-tracker.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
clap = { version = "4.4", features = ["derive"] }  # For CLI args

# Optional: Share code between binaries
[lib]
name = "claude_hooks_common"
path = "src/lib.rs"
```

### Shared Library Code

```rust
// src/lib.rs - shared utilities
pub mod config;
pub mod file_analyzer;
pub mod skill_matcher;

// src/bin/skill-activation-prompt.rs
use claude_hooks_common::config::load_config;
use claude_hooks_common::skill_matcher::match_skills;

fn main() {
    let config = load_config();
    let skills = match_skills(&config);
    // ... rest of implementation
}
```

---

## Update Workflow

### For Hook Developer

```bash
# Make changes
cd ~/.claude-hooks/src
# edit code

# Test
cargo run --bin skill-activation-prompt

# Build and install
cargo build --release
cp target/release/* ~/.claude-hooks/bin/

# Test in real project
cd ~/test-project
echo '{"prompt":"test"}' | ./.claude/hooks/skill-activation-prompt.sh
```

### For Hook User

```bash
# Update to latest version
cd ~/.claude-hooks/src
git pull
cargo build --release
cp target/release/* ~/.claude-hooks/bin/

# All projects automatically use new version
```

---

## Comparison: Embedded vs Standalone

### Embedded (Original Approach)

```
my-project/.claude/hooks/RustHooks/
├── Cargo.toml
├── src/
│   ├── skill_activation_prompt.rs
│   └── ...
└── target/
    └── release/
        └── skill-activation-prompt  (2.1 MB)
```

**Per project:**
- 45s build time
- 2.1 MB binary
- Full source code
- Must rebuild for updates

**Total for 5 projects:**
- 225s build time
- 10.5 MB disk space
- Update 5 times

---

### Standalone (Recommended)

```
~/.claude-hooks/
└── bin/
    └── skill-activation-prompt  (2.1 MB)

my-project/.claude/hooks/
└── skill-activation-prompt.sh  (50 bytes)
```

**Per project:**
- 0s build time (just copy wrapper)
- 50 bytes
- No source needed
- Automatic updates

**Total for 5 projects:**
- 45s build time (once)
- 2.1 MB disk space (shared)
- Update once

---

## Recommendation

**For Personal Use:**
- ✅ Build once in `~/.claude-hooks/`
- ✅ Per-project wrappers only
- ✅ Update centrally

**For Team Distribution:**
- ✅ GitHub releases with pre-built binaries
- ✅ Installation script
- ✅ Version pinning option

**For Open Source Project:**
- ✅ Publish to crates.io
- ✅ `cargo install claude-hooks`
- ✅ Automatic updates with `cargo install --force`

---

## Migration from Embedded to Standalone

```bash
# 1. Build standalone version
cd /tmp
git clone your-rust-hooks
cd rust-hooks
cargo build --release

# 2. Install globally
mkdir -p ~/.claude-hooks/bin
cp target/release/* ~/.claude-hooks/bin/

# 3. Update each project
cd ~/project-1/.claude/hooks
# Replace RustHooks/ with thin wrapper
rm -rf RustHooks/
cat > skill-activation-prompt.sh << 'EOF'
#!/bin/bash
cat | ~/.claude-hooks/bin/skill-activation-prompt
EOF
chmod +x skill-activation-prompt.sh

# 4. Repeat for other projects (or script it)
```

---

## Conclusion

**Yes, standalone Rust binaries are the better approach:**

- ✅ Compile once, use everywhere
- ✅ Faster project setup (just copy wrapper)
- ✅ Centralized maintenance
- ✅ Smaller per-project footprint
- ✅ Easier to distribute

**Only use embedded approach if:**
- You need per-project customization in Rust code
- You're actively developing/debugging the hook
- You have conflicting version requirements

**For 99% of use cases, standalone is better.**

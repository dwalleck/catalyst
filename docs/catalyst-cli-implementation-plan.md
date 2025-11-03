# Catalyst CLI Implementation Plan

**Status:** In Progress
**Created:** 2025-01-02
**Estimated Duration:** 3-4 weeks
**Goal:** Transform Catalyst from a manual 3-step installation into a single-command setup with `catalyst init`

---

## Executive Summary

### Current State (Pain Points)
- **3 manual steps:** Install binaries → Create wrappers → Edit settings.json
- **15-30 minutes** setup time for first-time users
- **High error rate:** Heredoc syntax, JSON editing, permissions
- **Documentation overload:** 600+ lines of setup instructions

### Future State (After Implementation)
- **1 command:** `catalyst init`
- **30-60 seconds** setup time
- **Low error rate:** Automated, validated
- **Self-documenting:** Built-in help and validation

### Key Changes
1. Rename `settings-manager` → `catalyst` (unified CLI)
2. Add `init` command (automated setup)
3. Add `status` command (validation & diagnostics)
4. Add `update` command (maintenance)
5. Add interactive mode (guided setup)

---

## User Experience Goals

### Before
```bash
# Step 1: Install binaries (45+ seconds)
cd catalyst
./install.sh

# Step 2: Create wrapper scripts (manual, error-prone)
cd my-project/.claude/hooks/
cat > skill-activation-prompt.sh << 'EOF'
#!/bin/bash
cat | ~/.claude-hooks/bin/skill-activation-prompt
EOF
chmod +x skill-activation-prompt.sh

# Step 3: Edit settings.json (manual JSON editing)
vi .claude/settings.json
# ... add hook configuration ...
```

### After
```bash
# Single command does everything
catalyst init

# Or with options
catalyst init --interactive
catalyst init --backend --frontend
catalyst init --all

# Validate setup anytime
catalyst status

# Fix issues automatically
catalyst status --fix

# Update installation
catalyst update
```

---

## Implementation Phases

## Phase 1: Rename & Foundation (2-3 days)

### 1.1 Rename Binary
**Files to modify:**
- `catalyst-cli/src/bin/settings_manager.rs` → `catalyst.rs`
- `catalyst-cli/Cargo.toml` - Update binary definition
- All documentation mentioning `settings-manager`

**Backward compatibility:**
- Keep `settings` subcommand with all existing functionality
- Old commands still work: `catalyst settings add-hook ...`

### 1.2 Restructure CLI with Subcommands

**New command structure:**
```bash
catalyst init [path]              # New: Initialize project
catalyst status [path]            # New: Validate setup
catalyst update [path]            # New: Update installation
catalyst settings <subcommand>    # Existing: Manage settings.json
```

**Implementation:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "catalyst")]
#[command(about = "Catalyst - Claude Code hooks and automation")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Catalyst in a project
    Init {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Interactive mode with guided prompts
        #[arg(short, long)]
        interactive: bool,

        /// Minimal setup (hooks only, no skills)
        #[arg(long)]
        minimal: bool,

        /// Install backend-dev-guidelines skill
        #[arg(long)]
        backend: bool,

        /// Install frontend-dev-guidelines skill
        #[arg(long)]
        frontend: bool,

        /// Install all available skills
        #[arg(long)]
        all: bool,

        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Check installation status and validate setup
    Status {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Automatically fix common issues
        #[arg(long)]
        fix: bool,
    },

    /// Update hooks and wrappers to latest version
    Update {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Force update even if no changes detected
        #[arg(long)]
        force: bool,
    },

    /// Manage settings.json configuration
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
    },
}

#[derive(Subcommand)]
enum SettingsCommands {
    /// Display current settings
    Read {
        /// Path to settings file
        #[arg(default_value = ".claude/settings.json")]
        file: PathBuf,
    },

    /// Validate settings file structure
    Validate {
        /// Path to settings file
        file: PathBuf,
    },

    /// Add a hook configuration
    AddHook {
        /// Hook event type (UserPromptSubmit, PostToolUse, etc.)
        #[arg(long)]
        event: String,

        /// Command to execute
        #[arg(long)]
        command: String,

        /// Matcher regex (optional)
        #[arg(long)]
        matcher: Option<String>,

        /// Show changes without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove hooks matching pattern
    RemoveHook {
        /// Pattern to match hook commands
        #[arg(long)]
        pattern: String,

        /// Show changes without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Merge two settings files
    Merge {
        /// Base settings file
        base: PathBuf,

        /// Additional settings to merge
        additional: PathBuf,

        /// Output file (default: base file)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
```

### 1.3 Create Core Init Framework

**Binary validation check:**
```rust
fn check_binaries_installed() -> Result<()> {
    let home_dir = get_home_dir();
    let bin_dir = home_dir.join(".claude-hooks").join("bin");

    let required_binaries = vec![
        "skill-activation-prompt",
        "post-tool-use-tracker-sqlite",
        "file-analyzer",
    ];

    let mut missing = Vec::new();
    for binary in required_binaries {
        let binary_path = bin_dir.join(binary);
        if !binary_path.exists() {
            missing.push(binary);
        }
    }

    if !missing.is_empty() {
        return Err(CatalystError::BinariesNotInstalled {
            missing,
            install_path: bin_dir.display().to_string(),
        });
    }

    Ok(())
}
```

**Platform detection:**
```rust
#[derive(Debug, Clone, Copy)]
enum Platform {
    Unix,
    Windows,
}

impl Platform {
    fn detect() -> Self {
        if cfg!(windows) {
            Platform::Windows
        } else {
            Platform::Unix
        }
    }

    fn wrapper_extension(&self) -> &str {
        match self {
            Platform::Unix => ".sh",
            Platform::Windows => ".ps1",
        }
    }

    fn home_dir_var(&self) -> &str {
        match self {
            Platform::Unix => "HOME",
            Platform::Windows => "USERPROFILE",
        }
    }
}
```

---

## Phase 2: Directory & File Creation (1-2 days)

### 2.1 Directory Structure

**Target structure:**
```
.claude/
├── hooks/           # Create with wrappers
├── skills/          # Create, optionally populate
├── agents/          # Create (future-proofing)
├── commands/        # Create (future-proofing)
└── settings.json    # Create or merge
```

**Implementation:**
```rust
fn create_directory_structure(project_dir: &Path) -> Result<()> {
    let claude_dir = project_dir.join(".claude");

    // Create main directory
    fs::create_dir_all(&claude_dir)
        .context("Failed to create .claude directory")?;

    // Create subdirectories
    for subdir in &["hooks", "skills", "agents", "commands"] {
        fs::create_dir_all(claude_dir.join(subdir))
            .with_context(|| format!("Failed to create .claude/{} directory", subdir))?;
    }

    Ok(())
}
```

### 2.2 Wrapper Script Generation

**Wrapper templates (embedded):**
```rust
// Embed wrapper templates at compile time
const UNIX_WRAPPER_TEMPLATE: &str = include_str!("../resources/wrapper-template.sh");
const WINDOWS_WRAPPER_TEMPLATE: &str = include_str!("../resources/wrapper-template.ps1");

fn create_wrapper_script(
    hooks_dir: &Path,
    binary_name: &str,
    platform: Platform,
) -> Result<()> {
    let wrapper_name = format!("{}{}", binary_name, platform.wrapper_extension());
    let wrapper_path = hooks_dir.join(&wrapper_name);

    let template = match platform {
        Platform::Unix => UNIX_WRAPPER_TEMPLATE,
        Platform::Windows => WINDOWS_WRAPPER_TEMPLATE,
    };

    // Replace placeholders in template
    let content = template.replace("{{BINARY_NAME}}", binary_name);

    fs::write(&wrapper_path, content)
        .with_context(|| format!("Failed to write wrapper script: {}", wrapper_name))?;

    // Set executable permissions on Unix
    if let Platform::Unix = platform {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&wrapper_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&wrapper_path, perms)?;
        }
    }

    Ok(())
}
```

**Wrapper template (Unix):**
```bash
#!/bin/bash
# Auto-generated by catalyst init
# Binary: {{BINARY_NAME}}

set -e

STANDALONE_BIN="$HOME/.claude-hooks/bin/{{BINARY_NAME}}"
PROJECT_BIN="$CLAUDE_PROJECT_DIR/target/release/{{BINARY_NAME}}"

if [ -f "$STANDALONE_BIN" ]; then
    cat | "$STANDALONE_BIN"
elif [ -f "$PROJECT_BIN" ]; then
    cat | "$PROJECT_BIN"
else
    echo "❌ {{BINARY_NAME}} not found!" >&2
    echo "Expected: $STANDALONE_BIN" >&2
    echo "Or: $PROJECT_BIN" >&2
    echo "Run: catalyst status --fix" >&2
    exit 1
fi
```

**Wrapper template (Windows):**
```powershell
#!/usr/bin/env pwsh
# Auto-generated by catalyst init
# Binary: {{BINARY_NAME}}

$standaloneBin = "$env:USERPROFILE\.claude-hooks\bin\{{BINARY_NAME}}.exe"
$projectBin = "$env:CLAUDE_PROJECT_DIR\target\release\{{BINARY_NAME}}.exe"

if (Test-Path $standaloneBin) {
    $input | & $standaloneBin
} elseif (Test-Path $projectBin) {
    $input | & $projectBin
} else {
    Write-Error "{{BINARY_NAME}} not found!"
    Write-Error "Expected: $standaloneBin"
    Write-Error "Or: $projectBin"
    Write-Error "Run: catalyst status --fix"
    exit 1
}
```

### 2.3 Settings.json Management

**Hook configuration templates:**
```rust
const HOOK_CONFIGS: &[HookConfig] = &[
    HookConfig {
        event: "UserPromptSubmit",
        command: "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt{{EXT}}",
        matcher: None,
    },
    HookConfig {
        event: "PostToolUse",
        command: "$CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use-tracker{{EXT}}",
        matcher: Some(r"Edit|MultiEdit|Write|NotebookEdit"),
    },
];

fn create_or_merge_settings(
    claude_dir: &Path,
    platform: Platform,
) -> Result<()> {
    let settings_path = claude_dir.join("settings.json");

    if settings_path.exists() {
        // Merge with existing
        merge_hook_configs(&settings_path, HOOK_CONFIGS, platform)?;
    } else {
        // Create new
        create_settings_file(&settings_path, HOOK_CONFIGS, platform)?;
    }

    // Validate
    validate_settings_file(&settings_path)?;

    Ok(())
}
```

---

## Phase 3: Skill Installation (2 days)

### 3.1 Skill Metadata

**Skill definitions:**
```rust
#[derive(Debug, Clone)]
struct SkillMetadata {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    tech_stack: Vec<&'static str>,
    recommended_for: SkillRecommendation,
}

#[derive(Debug, Clone, Copy)]
enum SkillRecommendation {
    Everyone,
    Backend,
    Frontend,
    Optional,
}

const AVAILABLE_SKILLS: &[SkillMetadata] = &[
    SkillMetadata {
        id: "skill-developer",
        name: "Skill Developer",
        description: "Create and manage Claude Code skills (recommended for all)",
        tech_stack: vec!["any"],
        recommended_for: SkillRecommendation::Everyone,
    },
    SkillMetadata {
        id: "backend-dev-guidelines",
        name: "Backend Development Guidelines",
        description: "Node.js, Express, TypeScript, Prisma patterns",
        tech_stack: vec!["node", "express", "prisma", "typescript"],
        recommended_for: SkillRecommendation::Backend,
    },
    SkillMetadata {
        id: "frontend-dev-guidelines",
        name: "Frontend Development Guidelines",
        description: "React, MUI v7, TanStack Query & Router patterns",
        tech_stack: vec!["react", "mui", "tanstack", "typescript"],
        recommended_for: SkillRecommendation::Frontend,
    },
    SkillMetadata {
        id: "route-tester",
        name: "Route Tester",
        description: "Test authenticated routes with cookie-based auth",
        tech_stack: vec!["node", "express", "jwt"],
        recommended_for: SkillRecommendation::Optional,
    },
    SkillMetadata {
        id: "error-tracking",
        name: "Error Tracking",
        description: "Sentry v8 integration patterns",
        tech_stack: vec!["sentry"],
        recommended_for: SkillRecommendation::Optional,
    },
];
```

### 3.2 Embedding Skills at Compile Time

```rust
use include_dir::{include_dir, Dir};

// Embed skills directory at compile time
static SKILLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../.claude/skills");

fn install_skill(
    skills_dir: &Path,
    skill_id: &str,
) -> Result<()> {
    // Extract embedded skill directory
    let skill_source = SKILLS_DIR
        .get_dir(skill_id)
        .ok_or_else(|| CatalystError::SkillNotFound(skill_id.to_string()))?;

    let skill_dest = skills_dir.join(skill_id);

    // Create skill directory
    fs::create_dir_all(&skill_dest)?;

    // Copy all files
    copy_dir_recursive(skill_source, &skill_dest)?;

    Ok(())
}
```

### 3.3 Project Structure Detection

**Interactive prompts:**
```rust
use dialoguer::{Select, MultiSelect, Confirm, Input};

fn detect_project_structure_interactive() -> Result<ProjectStructure> {
    let structures = vec![
        "Single application (src/)",
        "Monorepo (packages/*/src/)",
        "Multi-service (services/*/src/)",
        "Custom",
    ];

    let selection = Select::new()
        .with_prompt("Project structure?")
        .items(&structures)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(ProjectStructure::SingleApp),
        1 => Ok(ProjectStructure::Monorepo),
        2 => Ok(ProjectStructure::MultiService),
        3 => {
            let pattern = Input::<String>::new()
                .with_prompt("Enter path pattern (e.g., apps/*/src/**/*.ts)")
                .interact()?;
            Ok(ProjectStructure::Custom(pattern))
        },
        _ => unreachable!(),
    }
}

enum ProjectStructure {
    SingleApp,
    Monorepo,
    MultiService,
    Custom(String),
}

impl ProjectStructure {
    fn path_patterns(&self) -> Vec<String> {
        match self {
            Self::SingleApp => vec!["src/**/*.ts".to_string()],
            Self::Monorepo => vec!["packages/*/src/**/*.ts".to_string()],
            Self::MultiService => vec!["services/*/src/**/*.ts".to_string()],
            Self::Custom(pattern) => vec![pattern.clone()],
        }
    }
}
```

### 3.4 skill-rules.json Generation

```rust
fn generate_skill_rules(
    skills_dir: &Path,
    installed_skills: &[String],
    project_structure: &ProjectStructure,
) -> Result<()> {
    let rules_path = skills_dir.join("skill-rules.json");

    let mut rules = SkillRules {
        version: "1.0".to_string(),
        skills: HashMap::new(),
    };

    for skill_id in installed_skills {
        let skill_meta = AVAILABLE_SKILLS
            .iter()
            .find(|s| s.id == skill_id)
            .ok_or_else(|| CatalystError::SkillNotFound(skill_id.clone()))?;

        let rule = SkillRule {
            r#type: "UserPromptSubmit".to_string(),
            enforcement: "suggest".to_string(),
            priority: match skill_meta.recommended_for {
                SkillRecommendation::Everyone => "high".to_string(),
                _ => "medium".to_string(),
            },
            path_patterns: Some(project_structure.path_patterns()),
            // ... other rule fields
        };

        rules.skills.insert(skill_id.clone(), rule);
    }

    // Write rules file
    let json = serde_json::to_string_pretty(&rules)?;
    fs::write(rules_path, json)?;

    Ok(())
}
```

---

## Phase 4: Validation & Status (1-2 days)

### 4.1 Status Command Structure

```rust
struct StatusReport {
    overall: StatusLevel,
    binaries: Vec<BinaryStatus>,
    hooks: Vec<HookStatus>,
    skills: Vec<SkillStatus>,
    issues: Vec<Issue>,
}

enum StatusLevel {
    Healthy,
    Warning,
    Error,
}

struct BinaryStatus {
    name: String,
    found: bool,
    version: Option<String>,
    path: Option<PathBuf>,
}

struct HookStatus {
    event: String,
    configured: bool,
    wrapper_exists: bool,
    wrapper_executable: bool,
    binary_accessible: bool,
}

struct SkillStatus {
    name: String,
    installed: bool,
    rules_valid: bool,
}

struct Issue {
    severity: IssueSeverity,
    description: String,
    fix_suggestion: String,
    auto_fixable: bool,
}

enum IssueSeverity {
    Error,
    Warning,
    Info,
}
```

### 4.2 Validation Checks

```rust
fn validate_installation(project_dir: &Path) -> Result<StatusReport> {
    let mut report = StatusReport::default();

    // Check binaries
    report.binaries = check_binaries();

    // Check hooks
    report.hooks = check_hooks(project_dir)?;

    // Check skills
    report.skills = check_skills(project_dir)?;

    // Collect issues
    report.issues = collect_issues(&report);

    // Determine overall status
    report.overall = determine_overall_status(&report.issues);

    Ok(report)
}

fn check_binaries() -> Vec<BinaryStatus> {
    let home_dir = get_home_dir();
    let bin_dir = home_dir.join(".claude-hooks").join("bin");

    vec![
        "skill-activation-prompt",
        "post-tool-use-tracker-sqlite",
        "file-analyzer",
    ]
    .into_iter()
    .map(|name| {
        let path = bin_dir.join(name);
        BinaryStatus {
            name: name.to_string(),
            found: path.exists(),
            version: get_binary_version(&path),
            path: Some(path),
        }
    })
    .collect()
}

fn check_hooks(project_dir: &Path) -> Result<Vec<HookStatus>> {
    let settings_path = project_dir.join(".claude/settings.json");
    let hooks_dir = project_dir.join(".claude/hooks");

    if !settings_path.exists() {
        return Ok(Vec::new());
    }

    let settings = read_settings(&settings_path)?;

    let mut statuses = Vec::new();

    for hook in settings.hooks.unwrap_or_default() {
        let wrapper_name = extract_wrapper_name(&hook.command);
        let wrapper_path = hooks_dir.join(&wrapper_name);

        let status = HookStatus {
            event: hook.event,
            configured: true,
            wrapper_exists: wrapper_path.exists(),
            wrapper_executable: is_executable(&wrapper_path),
            binary_accessible: check_binary_accessible(&wrapper_path),
        };

        statuses.push(status);
    }

    Ok(statuses)
}
```

### 4.3 Auto-Fix Implementation

```rust
fn auto_fix_issues(project_dir: &Path, issues: &[Issue]) -> Result<FixReport> {
    let mut fixed = Vec::new();
    let mut failed = Vec::new();

    for issue in issues.iter().filter(|i| i.auto_fixable) {
        match fix_issue(project_dir, issue) {
            Ok(_) => fixed.push(issue.description.clone()),
            Err(e) => failed.push((issue.description.clone(), e)),
        }
    }

    Ok(FixReport { fixed, failed })
}

fn fix_issue(project_dir: &Path, issue: &Issue) -> Result<()> {
    // Example: Fix permissions
    if issue.description.contains("not executable") {
        let wrapper_path = extract_path_from_description(&issue.description);
        set_executable(&wrapper_path)?;
    }

    // Example: Recreate missing wrapper
    if issue.description.contains("wrapper not found") {
        let binary_name = extract_binary_name(&issue.description);
        create_wrapper_script(
            &project_dir.join(".claude/hooks"),
            &binary_name,
            Platform::detect(),
        )?;
    }

    Ok(())
}
```

### 4.4 Status Output Format

```rust
fn print_status_report(report: &StatusReport, use_color: bool) {
    // Header
    let status_icon = match report.overall {
        StatusLevel::Healthy => "✅",
        StatusLevel::Warning => "⚠️",
        StatusLevel::Error => "❌",
    };

    println!("\n{} Catalyst Status: {}\n", status_icon,
             format!("{:?}", report.overall).to_uppercase());

    // Binaries section
    println!("Binaries:");
    for binary in &report.binaries {
        let status = if binary.found { "✓" } else { "✗" };
        println!("  {} {} {}",
                 status,
                 binary.name,
                 binary.version.as_deref().unwrap_or("(version unknown)"));
    }
    println!();

    // Hooks section
    if !report.hooks.is_empty() {
        println!("Hooks:");
        for hook in &report.hooks {
            let status = if hook.wrapper_exists && hook.wrapper_executable {
                "✓"
            } else {
                "✗"
            };
            println!("  {} {} → wrapper", status, hook.event);
        }
        println!();
    }

    // Skills section
    if !report.skills.is_empty() {
        println!("Skills:");
        for skill in &report.skills {
            let status = if skill.installed { "✓" } else { "✗" };
            println!("  {} {}", status, skill.name);
        }
        println!();
    }

    // Issues section
    if !report.issues.is_empty() {
        println!("Issues:");
        for issue in &report.issues {
            let icon = match issue.severity {
                IssueSeverity::Error => "✗",
                IssueSeverity::Warning => "⚠",
                IssueSeverity::Info => "ℹ",
            };
            println!("  {} {}", icon, issue.description);
            println!("    Fix: {}", issue.fix_suggestion);
        }
        println!();

        let fixable = report.issues.iter().filter(|i| i.auto_fixable).count();
        if fixable > 0 {
            println!("Run 'catalyst status --fix' to automatically repair {} issues", fixable);
        }
    } else {
        println!("Issues: None");
    }
}
```

---

## Phase 5: Interactive Mode (1-2 days)

### 5.1 Dependencies

```toml
[dependencies]
# Existing...

# Interactive prompts
dialoguer = { version = "0.11", features = ["completion"] }
```

### 5.2 Interactive Flow

```rust
fn run_interactive_init(project_dir: &Path) -> Result<InitConfig> {
    use dialoguer::*;

    println!("\n🎯 Catalyst Interactive Setup\n");

    // Confirm directory
    let confirmed = Confirm::new()
        .with_prompt(format!("Initialize Catalyst in '{}'?",
                            project_dir.display()))
        .default(true)
        .interact()?;

    if !confirmed {
        return Err(CatalystError::Cancelled);
    }

    // Install hooks?
    let install_hooks = Confirm::new()
        .with_prompt("Install skill auto-activation hooks?")
        .default(true)
        .interact()?;

    let install_tracker = if install_hooks {
        Confirm::new()
            .with_prompt("Install post-tool-use tracker?")
            .default(true)
            .interact()?
    } else {
        false
    };

    // Select skills
    let skill_selection = MultiSelect::new()
        .with_prompt("Which skills to install?")
        .items(&AVAILABLE_SKILLS.iter().map(|s| {
            format!("{} - {}", s.name, s.description)
        }).collect::<Vec<_>>())
        .defaults(&[true, false, false, false, false]) // skill-developer checked
        .interact()?;

    let selected_skills: Vec<String> = skill_selection
        .into_iter()
        .map(|i| AVAILABLE_SKILLS[i].id.to_string())
        .collect();

    // Project structure
    let structure = if !selected_skills.is_empty() {
        Some(detect_project_structure_interactive()?)
    } else {
        None
    };

    Ok(InitConfig {
        install_hooks,
        install_tracker,
        skills: selected_skills,
        project_structure: structure,
    })
}
```

### 5.3 Progress Indicators

```rust
use indicatif::{ProgressBar, ProgressStyle};

fn show_progress<F>(total: u64, message: &str, f: F) -> Result<()>
where
    F: FnOnce(&ProgressBar) -> Result<()>,
{
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());

    let result = f(&pb);
    pb.finish_with_message("Done");

    result
}

// Usage
show_progress(5, "Installing skills", |pb| {
    for (i, skill) in skills.iter().enumerate() {
        install_skill(skills_dir, skill)?;
        pb.inc(1);
        pb.set_message(format!("Installed {}", skill));
    }
    Ok(())
})?;
```

---

## Phase 6: Update Command (1 day)

### 6.1 Version Tracking

```rust
const CATALYST_VERSION: &str = env!("CARGO_PKG_VERSION");

fn check_for_updates(project_dir: &Path) -> Result<UpdateInfo> {
    let claude_dir = project_dir.join(".claude");
    let version_file = claude_dir.join(".catalyst-version");

    let installed_version = if version_file.exists() {
        fs::read_to_string(version_file)?
    } else {
        "unknown".to_string()
    };

    Ok(UpdateInfo {
        current: CATALYST_VERSION.to_string(),
        installed: installed_version,
        updates_available: CATALYST_VERSION != installed_version,
    })
}

fn write_version_file(project_dir: &Path) -> Result<()> {
    let version_file = project_dir.join(".claude/.catalyst-version");
    fs::write(version_file, CATALYST_VERSION)?;
    Ok(())
}
```

### 6.2 Update Logic

```rust
fn update_installation(project_dir: &Path, force: bool) -> Result<UpdateReport> {
    let mut report = UpdateReport::default();

    let update_info = check_for_updates(project_dir)?;

    if !force && !update_info.updates_available {
        println!("Already up to date (v{})", update_info.current);
        return Ok(report);
    }

    println!("Updating from {} to {}",
             update_info.installed,
             update_info.current);

    // Update wrapper scripts
    report.wrappers_updated = update_wrappers(project_dir)?;

    // Update settings.json hooks (preserve customizations)
    report.settings_updated = update_settings_hooks(project_dir)?;

    // Update skills (with backup)
    report.skills_updated = update_skills(project_dir)?;

    // Write new version file
    write_version_file(project_dir)?;

    Ok(report)
}

fn update_wrappers(project_dir: &Path) -> Result<usize> {
    let hooks_dir = project_dir.join(".claude/hooks");
    let platform = Platform::detect();

    let mut updated = 0;

    for binary in &["skill-activation-prompt", "post-tool-use-tracker"] {
        let wrapper_name = format!("{}{}", binary, platform.wrapper_extension());
        let wrapper_path = hooks_dir.join(&wrapper_name);

        if wrapper_path.exists() {
            // Backup old wrapper
            let backup = wrapper_path.with_extension(
                format!("{}. bak", platform.wrapper_extension().trim_start_matches('.'))
            );
            fs::copy(&wrapper_path, &backup)?;

            // Write new wrapper
            create_wrapper_script(&hooks_dir, binary, platform)?;
            updated += 1;
        }
    }

    Ok(updated)
}
```

---

## Phase 7: Polish & UX (1-2 days)

### 7.1 Error Types

```rust
#[derive(Error, Debug)]
enum CatalystError {
    #[error("Catalyst binaries not installed at {install_path}\nMissing: {}\nRun the installation script first: ./install.sh", missing.join(", "))]
    BinariesNotInstalled {
        missing: Vec<String>,
        install_path: String,
    },

    #[error("Skill not found: {0}\nAvailable skills: {}", AVAILABLE_SKILLS.iter().map(|s| s.id).collect::<Vec<_>>().join(", "))]
    SkillNotFound(String),

    #[error("Directory already initialized\nUse --force to reinitialize or 'catalyst status' to check current setup")]
    AlreadyInitialized,

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Invalid project structure pattern: {0}\nExpected format: 'path/pattern/**/*.ext'")]
    InvalidPathPattern(String),
}
```

### 7.2 Colored Output

```rust
fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message);
}

fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message);
}

fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
}

fn print_section_header(title: &str) {
    println!("\n{}\n", title.bright_cyan().bold());
}
```

### 7.3 Final Output Summary

```rust
fn print_init_summary(config: &InitConfig, report: &InitReport) {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    println!("{}", "✅ Catalyst initialized successfully!".green().bold());
    println!("{}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    println!("Created:");
    for dir in &report.created_dirs {
        println!("  ✓ {}", dir);
    }

    if !report.installed_hooks.is_empty() {
        println!("\nInstalled hooks:");
        for hook in &report.installed_hooks {
            println!("  ✓ {}", hook);
        }
    }

    if !config.skills.is_empty() {
        println!("\nInstalled skills:");
        for skill in &config.skills {
            let meta = AVAILABLE_SKILLS.iter().find(|s| s.id == skill).unwrap();
            println!("  ✓ {}", meta.name);
        }
    }

    println!("\nNext steps:");
    println!("  1. Review .claude/settings.json");

    if !config.skills.is_empty() {
        println!("  2. Customize .claude/skills/skill-rules.json for your project structure");
        println!("  3. Try editing a file - skills should activate automatically");
    }

    println!("  4. Run 'catalyst status' to validate setup");

    println!("\nDocumentation: {}", "https://github.com/dwalleck/catalyst".bright_blue());
}
```

---

## Phase 8: Testing (1-2 days)

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_directory_structure() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();

        create_directory_structure(project_dir).unwrap();

        assert!(project_dir.join(".claude").exists());
        assert!(project_dir.join(".claude/hooks").exists());
        assert!(project_dir.join(".claude/skills").exists());
    }

    #[test]
    fn test_wrapper_generation() {
        let temp = TempDir::new().unwrap();
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        create_wrapper_script(
            &hooks_dir,
            "skill-activation-prompt",
            Platform::Unix,
        ).unwrap();

        let wrapper = hooks_dir.join("skill-activation-prompt.sh");
        assert!(wrapper.exists());

        let content = fs::read_to_string(&wrapper).unwrap();
        assert!(content.contains("#!/bin/bash"));
        assert!(content.contains("skill-activation-prompt"));
    }

    #[test]
    fn test_settings_creation() {
        let temp = TempDir::new().unwrap();
        let claude_dir = temp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        create_or_merge_settings(&claude_dir, Platform::Unix).unwrap();

        let settings_path = claude_dir.join("settings.json");
        assert!(settings_path.exists());

        // Validate JSON
        let content = fs::read_to_string(&settings_path).unwrap();
        let _: serde_json::Value = serde_json::from_str(&content).unwrap();
    }
}
```

### 8.2 Integration Tests

```rust
#[test]
fn test_full_init_flow() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    let config = InitConfig {
        install_hooks: true,
        install_tracker: true,
        skills: vec!["skill-developer".to_string()],
        project_structure: Some(ProjectStructure::SingleApp),
    };

    let result = run_init(project_dir, &config);
    assert!(result.is_ok());

    // Verify directory structure
    assert!(project_dir.join(".claude/hooks").exists());
    assert!(project_dir.join(".claude/skills").exists());
    assert!(project_dir.join(".claude/settings.json").exists());

    // Verify skill installed
    assert!(project_dir.join(".claude/skills/skill-developer").exists());

    // Verify wrappers
    let wrapper = if cfg!(windows) {
        "skill-activation-prompt.ps1"
    } else {
        "skill-activation-prompt.sh"
    };
    assert!(project_dir.join(".claude/hooks").join(wrapper).exists());
}

#[test]
fn test_status_command() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Initialize first
    let config = InitConfig::minimal();
    run_init(project_dir, &config).unwrap();

    // Run status
    let report = validate_installation(project_dir).unwrap();

    assert_eq!(report.overall, StatusLevel::Healthy);
    assert!(report.issues.is_empty());
}
```

---

## Phase 9: Documentation (1 day)

### 9.1 README.md Updates

**Add prominent quick start:**
```markdown
## Quick Start

### 1. Install Catalyst binaries (one-time, 45 seconds)
```bash
cd catalyst
./install.sh
# Or with SQLite support: ./install.sh --sqlite
```

### 2. Initialize in your project (30 seconds)
```bash
cd your-project
catalyst init

# Or with interactive setup
catalyst init --interactive

# Or with specific skills
catalyst init --backend --frontend
```

### 3. Verify installation
```bash
catalyst status
```

That's it! Skills will now auto-activate when you edit files.
```

### 9.2 New docs/catalyst-cli.md

**Comprehensive CLI reference:**
```markdown
# Catalyst CLI Reference

## Commands

### catalyst init

Initialize Catalyst in a project directory.

**Usage:**
```bash
catalyst init [OPTIONS] [PATH]
```

**Options:**
- `-i, --interactive` - Interactive mode with guided prompts
- `--minimal` - Minimal setup (hooks only, no skills)
- `--backend` - Install backend-dev-guidelines skill
- `--frontend` - Install frontend-dev-guidelines skill
- `--all` - Install all available skills
- `--dry-run` - Show what would be done without making changes

**Examples:**
```bash
# Initialize in current directory
catalyst init

# Interactive setup
catalyst init --interactive

# Install backend and frontend skills
catalyst init --backend --frontend

# Preview changes
catalyst init --dry-run
```

### catalyst status

Check installation status and validate setup.

**Usage:**
```bash
catalyst status [OPTIONS] [PATH]
```

**Options:**
- `--fix` - Automatically fix common issues

**Examples:**
```bash
# Check status
catalyst status

# Check and fix issues
catalyst status --fix
```

[... etc for all commands ...]
```

### 9.3 Update all docs mentioning manual setup

**Files to update:**
- README.md
- docs/standalone-installation.md
- CLAUDE_INTEGRATION_GUIDE.md
- docs/rust-hooks.md

**Add migration note:**
```markdown
## Migrating from Manual Setup

If you previously set up Catalyst manually:

1. Your existing installation will continue to work
2. Run `catalyst status` to validate your setup
3. Optionally run `catalyst init` to update to the new structure
4. The init command will detect and preserve your existing configuration
```

---

## File Structure After Implementation

```
catalyst-cli/src/
├── bin/
│   ├── catalyst.rs                         # Main CLI (renamed from settings_manager)
│   ├── skill_activation_prompt.rs
│   ├── file_analyzer.rs
│   └── post_tool_use_tracker_sqlite.rs
├── commands/
│   ├── mod.rs
│   ├── init.rs                             # Init command logic
│   ├── status.rs                           # Status command logic
│   ├── update.rs                           # Update command logic
│   └── settings.rs                         # Settings management (existing)
├── resources/
│   ├── wrapper-template.sh                 # Unix wrapper template
│   └── wrapper-template.ps1                # Windows wrapper template
└── lib.rs                                  # Shared utilities

catalyst-cli/Cargo.toml                     # Updated dependencies
docs/
├── catalyst-cli-implementation-plan.md     # This file
└── catalyst-cli.md                         # CLI reference (new)
```

---

## Dependencies to Add

```toml
[dependencies]
# Existing
clap = { workspace = true }
colored = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

# New for Phase 5 (Interactive)
dialoguer = { version = "0.11", features = ["completion"] }
indicatif = "0.17"                          # Progress bars

# New for Phase 3 (Embedded skills)
include_dir = "0.7"                         # Embed directories at compile time

# Development
tempfile = "3.8"                            # For tests
```

---

## Success Metrics

### Quantitative
- **Setup time:** 15-30 min → 30-60 seconds
- **Error rate:** 40-50% → <5%
- **Documentation needed:** 600+ lines → 100 lines quick start
- **User steps:** 3 manual → 1 automated
- **CLI commands:** 1 binary (settings-manager) → 1 unified (catalyst)

### Qualitative
- First-time users can set up without reading docs
- Existing users can validate their setup
- Issues are automatically detected and fixed
- Skills can be easily added/removed
- Updates are safe and preserve customizations

---

## Risk Mitigation

### Risk 1: Breaking existing installations
**Mitigation:**
- Detect existing setup before making changes
- Offer to merge rather than replace
- Backup files before updating
- Make init command idempotent

### Risk 2: Platform-specific bugs
**Mitigation:**
- CI testing on Windows/Linux/macOS
- Platform detection with fallbacks
- Clear error messages for unsupported scenarios

### Risk 3: Embedded resources bloat binary size
**Mitigation:**
- Skills are text files (~500KB total)
- Binary size increase acceptable for UX gain
- Alternative: Download skills on demand (future)

### Risk 4: Complex interactive flow confuses users
**Mitigation:**
- Sensible defaults for non-interactive mode
- Clear prompts with examples
- Dry-run mode to preview changes
- Comprehensive help text

---

## Timeline

### Week 1: Foundation
- **Days 1-2:** Phase 1 (Rename, subcommands, structure)
- **Days 3-5:** Phase 2 (Directories, wrappers, settings)

### Week 2: Core Features
- **Days 1-2:** Phase 3 (Skill installation)
- **Days 3-4:** Phase 4 (Validation, status)
- **Day 5:** Phase 5 start (Interactive mode)

### Week 3: Enhancement
- **Day 1:** Phase 5 finish (Interactive mode)
- **Day 2:** Phase 6 (Update command)
- **Days 3-4:** Phase 7 (Polish & UX)
- **Day 5:** Phase 8 start (Testing)

### Week 4: Release
- **Days 1-2:** Phase 8 finish (Testing)
- **Day 3:** Phase 9 (Documentation)
- **Days 4-5:** Bug fixes, final testing, release

---

## Open Questions

1. **Binary distribution:** Continue standalone installation or publish to crates.io?
2. **Skill updates:** How to update skills when changes are made upstream?
3. **Custom skills:** Should users be able to install their own skills?
4. **Telemetry:** Should we track usage (opt-in) to improve UX?
5. **Auto-updates:** Should catalyst check for updates automatically?

---

## Future Enhancements (Post-MVP)

- **catalyst add-skill:** Install skills after initialization
- **catalyst remove-skill:** Uninstall skills
- **catalyst template:** Create custom project templates
- **catalyst doctor:** Deep diagnostic tool
- **catalyst upgrade:** In-place binary upgrades
- **Custom skill support:** Install from URLs or local paths
- **Skill marketplace:** Browse and install community skills
- **Configuration profiles:** Save and reuse initialization configs

---

## Appendix: Command Examples

### Complete Usage Examples

**First-time setup:**
```bash
# Interactive setup with all prompts
catalyst init --interactive

# Quick setup with defaults
catalyst init

# Specific skills
catalyst init --backend --frontend

# Preview before committing
catalyst init --all --dry-run
```

**Maintaining installation:**
```bash
# Check status
catalyst status

# Fix issues
catalyst status --fix

# Update to latest
catalyst update
```

**Advanced usage:**
```bash
# Initialize in different directory
catalyst init ~/projects/my-app

# Force reinitialize
catalyst init --force

# Minimal setup for testing
catalyst init --minimal

# Settings management (backward compatible)
catalyst settings read
catalyst settings validate .claude/settings.json
catalyst settings add-hook --event UserPromptSubmit --command "./hook.sh"
```

---

**End of Implementation Plan**

This plan will be continuously updated as implementation progresses.

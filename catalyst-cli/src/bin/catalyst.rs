//! Catalyst CLI
//!
//! Unified CLI tool for Claude Code project setup and management.
//!
//! # Commands
//!
//! - `init` - Initialize a Claude Code project with hooks and skills
//! - `status` - Validate installation and report issues
//! - `update` - Update hooks and skills to latest version
//! - `settings` - Manage settings.json files (legacy commands)
//!
//! # Examples
//!
//! ```bash
//! # Initialize a new project
//! catalyst init
//!
//! # Initialize with interactive mode
//! catalyst init --interactive
//!
//! # Check status of current installation
//! catalyst status
//!
//! # Auto-fix common issues
//! catalyst status --fix
//!
//! # Update to latest version
//! catalyst update
//! ```

use anyhow::Result;
use catalyst_cli::init;
use catalyst_cli::types::InitConfig;
use catalyst_cli::validation::check_binaries_installed;
use catalyst_core::settings::*;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "catalyst")]
#[command(version, about = "Catalyst - Claude Code project setup and management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a Claude Code project with hooks and skills
    Init {
        /// Directory to initialize (defaults to current directory)
        #[arg(short, long, value_name = "DIR")]
        path: Option<PathBuf>,

        /// Interactive mode with prompts
        #[arg(short, long)]
        interactive: bool,

        /// Force re-initialization (overwrite existing files)
        #[arg(short, long)]
        force: bool,

        /// Install all available skills
        #[arg(long)]
        all: bool,
    },

    /// Validate installation and report issues
    Status {
        /// Directory to check (defaults to current directory)
        #[arg(short, long, value_name = "DIR")]
        path: Option<PathBuf>,

        /// Auto-fix common issues
        #[arg(short, long)]
        fix: bool,
    },

    /// Update hooks and skills to latest version
    Update {
        /// Directory to update (defaults to current directory)
        #[arg(short, long, value_name = "DIR")]
        path: Option<PathBuf>,

        /// Force update even if files were modified locally
        #[arg(short, long)]
        force: bool,
    },

    /// Manage settings.json files (legacy commands)
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
    },
}

#[derive(Subcommand)]
enum SettingsCommands {
    /// Read and display settings file
    Read {
        /// Path to settings.json
        #[arg(default_value = ".claude/settings.json")]
        path: String,
    },

    /// Validate settings file structure
    Validate {
        /// Path to settings.json
        #[arg(default_value = ".claude/settings.json")]
        path: String,
    },

    /// Add a hook to settings
    AddHook {
        /// Path to settings.json
        #[arg(short, long, default_value = ".claude/settings.json")]
        path: String,

        /// Hook event type (UserPromptSubmit, PostToolUse, Stop)
        #[arg(short, long)]
        event: String,

        /// Hook command to execute
        #[arg(short, long)]
        command: String,

        /// Optional matcher pattern (regex)
        #[arg(short, long)]
        matcher: Option<String>,

        /// Dry run - preview changes without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove hooks matching a command pattern
    RemoveHook {
        /// Path to settings.json
        #[arg(short, long, default_value = ".claude/settings.json")]
        path: String,

        /// Hook event type
        #[arg(short, long)]
        event: String,

        /// Command pattern to match for removal
        #[arg(short = 'P', long)]
        pattern: String,

        /// Dry run - preview changes without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Merge two settings files
    Merge {
        /// Base settings file
        base: String,

        /// Settings file to merge in
        merge: String,

        /// Output file (defaults to base file)
        #[arg(short, long)]
        output: Option<String>,

        /// Dry run - preview merge without writing
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    // Check for NO_COLOR environment variable and TTY
    let use_color = env::var("NO_COLOR").is_err() && io::stdout().is_terminal();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            path,
            interactive,
            force,
            all,
        } => {
            let target_dir =
                path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            // Check if binaries are installed
            let platform = catalyst_cli::types::Platform::detect();
            if let Err(e) = check_binaries_installed(platform) {
                if use_color {
                    eprintln!("{}", format!("❌ {}", e).red().bold());
                } else {
                    eprintln!("❌ {}", e);
                }
                std::process::exit(1);
            }

            // Build skill list based on flags
            let mut skills = Vec::new();
            if all {
                skills.extend_from_slice(catalyst_cli::types::AVAILABLE_SKILLS);
            } else {
                // Default: install skill-developer
                // TODO Phase 3: Implement skill selection logic
                skills.push("skill-developer");
            }

            // Build config
            let config = InitConfig {
                directory: target_dir.clone(),
                install_hooks: true,   // Always install hooks
                install_tracker: true, // Always install tracker
                skills: skills.iter().map(|s| s.to_string()).collect(),
                force,
            };

            // Handle interactive mode
            if interactive {
                if use_color {
                    println!("{}", "🔧 Interactive mode not yet implemented".yellow());
                } else {
                    println!("🔧 Interactive mode not yet implemented");
                }
                println!("Proceeding with default configuration...\n");
            }

            // Run initialization
            if use_color {
                println!("{}", "🚀 Initializing Catalyst...".cyan().bold());
            } else {
                println!("🚀 Initializing Catalyst...");
            }
            println!();

            match init::initialize(&config) {
                Ok(report) => {
                    // Display success report
                    if use_color {
                        println!("{}", "━".repeat(60).bright_cyan());
                        println!("{}", "✅ Catalyst initialized successfully!".green().bold());
                        println!("{}", "━".repeat(60).bright_cyan());
                    } else {
                        println!("{}", "=".repeat(60));
                        println!("✅ Catalyst initialized successfully!");
                        println!("{}", "=".repeat(60));
                    }
                    println!();

                    // Created directories
                    if !report.created_dirs.is_empty() {
                        if use_color {
                            println!("{}", "Created directories:".cyan().bold());
                        } else {
                            println!("Created directories:");
                        }
                        for dir in &report.created_dirs {
                            println!("  ✓ {}", dir);
                        }
                        println!();
                    }

                    // Installed hooks
                    if !report.installed_hooks.is_empty() {
                        if use_color {
                            println!("{}", "Installed hooks:".cyan().bold());
                        } else {
                            println!("Installed hooks:");
                        }
                        for hook in &report.installed_hooks {
                            println!("  ✓ {}", hook);
                        }
                        println!();
                    }

                    // Installed skills
                    if !report.installed_skills.is_empty() {
                        if use_color {
                            println!("{}", "Installed skills:".cyan().bold());
                        } else {
                            println!("Installed skills:");
                        }
                        for skill in &report.installed_skills {
                            println!("  ✓ {}", skill);
                        }
                        println!();
                    }

                    // Settings file
                    if report.settings_created {
                        if use_color {
                            println!("{}", "Configuration:".cyan().bold());
                        } else {
                            println!("Configuration:");
                        }
                        println!("  ✓ .claude/settings.json");
                        println!();
                    }

                    // Next steps
                    if use_color {
                        println!("{}", "Next steps:".yellow().bold());
                    } else {
                        println!("Next steps:");
                    }
                    println!("  1. Review .claude/settings.json");
                    println!("  2. Try editing a file - hooks should activate automatically");
                    println!("  3. Run 'catalyst status' to validate setup");
                    println!();

                    if use_color {
                        println!(
                            "{}",
                            "📖 Documentation: https://github.com/dwalleck/catalyst".bright_blue()
                        );
                    } else {
                        println!("📖 Documentation: https://github.com/dwalleck/catalyst");
                    }
                }
                Err(e) => {
                    if use_color {
                        eprintln!(
                            "{}",
                            format!("❌ Initialization failed: {}", e).red().bold()
                        );
                    } else {
                        eprintln!("❌ Initialization failed: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Status { path, fix } => {
            let target_dir =
                path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            // Detect platform
            let platform = catalyst_cli::types::Platform::detect();

            // Validate installation
            match catalyst_cli::status::validate_installation(&target_dir, platform) {
                Ok(report) => {
                    // If --fix flag provided and there are auto-fixable issues, attempt fixes
                    let mut fixed_issues = Vec::new();
                    if fix && report.issues.iter().any(|i| i.auto_fixable) {
                        match catalyst_cli::status::auto_fix(&target_dir, platform, &report) {
                            Ok(fixes) => {
                                fixed_issues = fixes;
                            }
                            Err(e) => {
                                if use_color {
                                    eprintln!(
                                        "{}",
                                        format!("❌ Auto-fix failed: {}", e).red().bold()
                                    );
                                } else {
                                    eprintln!("❌ Auto-fix failed: {}", e);
                                }
                            }
                        }
                    }

                    // Display status report
                    display_status_report(&report, use_color, &fixed_issues);

                    // Exit with error code if status is not ok
                    if report.level != catalyst_cli::types::StatusLevel::Ok {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    if use_color {
                        eprintln!("{}", format!("❌ Status check failed: {}", e).red().bold());
                    } else {
                        eprintln!("❌ Status check failed: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Update { path, force } => {
            let target_dir =
                path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            if use_color {
                println!("{}", "⚠️  Not implemented yet".yellow().bold());
            } else {
                println!("⚠️  Not implemented yet");
            }
            println!("Would update: {:?}", target_dir);
            println!("  Force: {}", force);
        }

        Commands::Settings { command } => {
            match command {
                SettingsCommands::Read { path } => {
                    let settings = ClaudeSettings::read(&path)?;
                    let json = serde_json::to_string_pretty(&settings)?;
                    println!("{}", json);
                }

                SettingsCommands::Validate { path } => {
                    let settings = ClaudeSettings::read(&path)?;
                    settings.validate()?;

                    if use_color {
                        println!("{}", "✅ Settings file is valid".green().bold());
                    } else {
                        println!("✅ Settings file is valid");
                    }
                }

                SettingsCommands::AddHook {
                    path,
                    event,
                    command,
                    matcher,
                    dry_run,
                } => {
                    // Load existing settings or create new
                    // Only create defaults for missing files, not for other errors (permissions, invalid JSON, etc.)
                    let (mut settings, file_existed) = match ClaudeSettings::read(&path) {
                        Ok(s) => (s, true),
                        Err(e) => {
                            // Check if the underlying error is io::ErrorKind::NotFound
                            // Use downcast_ref to check the root cause
                            let is_not_found = e.chain().any(|cause| {
                                cause
                                    .downcast_ref::<std::io::Error>()
                                    .map(|io_err| io_err.kind() == std::io::ErrorKind::NotFound)
                                    .unwrap_or(false)
                            });

                            if is_not_found {
                                (ClaudeSettings::default(), false)
                            } else {
                                // Propagate other errors (permissions, invalid JSON, etc.)
                                return Err(e);
                            }
                        }
                    };

                    // Parse event string into HookEvent enum
                    let hook_event = HookEvent::from_str(&event)?;

                    let hook_config = HookConfig {
                        matcher: matcher.clone(),
                        hooks: vec![Hook {
                            r#type: "command".to_string(),
                            command: command.clone(),
                        }],
                    };

                    settings.add_hook(hook_event, hook_config)?;

                    if dry_run {
                        if use_color {
                            println!("{}", "🔍 Dry run - would write:".yellow().bold());
                        } else {
                            println!("🔍 Dry run - would write:");
                        }
                        println!("{}", serde_json::to_string_pretty(&settings)?);
                    } else {
                        settings.write(&path)?;

                        if use_color {
                            if file_existed {
                                println!(
                                    "{} {}",
                                    "✅ Hook added to existing file:".green().bold(),
                                    path
                                );
                            } else {
                                println!(
                                    "{} {}",
                                    "✅ Created new settings file:".green().bold(),
                                    path
                                );
                            }
                            println!("  {} {}", "Event:".cyan(), event);
                            println!("  {} {}", "Command:".cyan(), command);
                            if let Some(m) = matcher {
                                println!("  {} {}", "Matcher:".cyan(), m);
                            }
                        } else {
                            if file_existed {
                                println!("✅ Hook added to existing file: {}", path);
                            } else {
                                println!("✅ Created new settings file: {}", path);
                            }
                            println!("  Event: {}", event);
                            println!("  Command: {}", command);
                            if let Some(m) = matcher {
                                println!("  Matcher: {}", m);
                            }
                        }
                    }
                }

                SettingsCommands::RemoveHook {
                    path,
                    event,
                    pattern,
                    dry_run,
                } => {
                    let mut settings = ClaudeSettings::read(&path)?;

                    // Parse event string into HookEvent enum
                    let hook_event = HookEvent::from_str(&event)?;

                    settings.remove_hook(hook_event, &pattern);

                    if dry_run {
                        if use_color {
                            println!("{}", "🔍 Dry run - would write:".yellow().bold());
                        } else {
                            println!("🔍 Dry run - would write:");
                        }
                        println!("{}", serde_json::to_string_pretty(&settings)?);
                    } else {
                        settings.write(&path)?;
                        if use_color {
                            println!("{} {}", "✅ Hooks removed from".green().bold(), path);
                        } else {
                            println!("✅ Hooks removed from {}", path);
                        }
                    }
                }

                SettingsCommands::Merge {
                    base,
                    merge,
                    output,
                    dry_run,
                } => {
                    let mut base_settings = ClaudeSettings::read(&base)?;
                    let merge_settings = ClaudeSettings::read(&merge)?;

                    base_settings.merge(merge_settings);

                    // Validate merged result
                    base_settings.validate()?;

                    let output_path = output.as_deref().unwrap_or(&base);

                    if dry_run {
                        if use_color {
                            println!(
                                "{} {}:",
                                "🔍 Dry run - would write to".yellow().bold(),
                                output_path
                            );
                        } else {
                            println!("🔍 Dry run - would write to {}:", output_path);
                        }
                        println!("{}", serde_json::to_string_pretty(&base_settings)?);
                    } else {
                        base_settings.write(output_path)?;
                        if use_color {
                            println!("{}", "✅ Settings merged successfully".green().bold());
                            println!("  {} {}", "Base file:".cyan(), base);
                            println!("  {} {}", "Merged from:".cyan(), merge);
                            println!("  {} {}", "Output:".cyan(), output_path);
                        } else {
                            println!("✅ Settings merged successfully");
                            println!("  Base file: {}", base);
                            println!("  Merged from: {}", merge);
                            println!("  Output: {}", output_path);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Display a formatted status report
fn display_status_report(
    report: &catalyst_cli::types::StatusReport,
    use_color: bool,
    fixed_issues: &[String],
) {
    use catalyst_cli::types::{IssueSeverity, StatusLevel};

    // Show fixed issues first if any
    if !fixed_issues.is_empty() {
        if use_color {
            println!("\n{}", "🔧 Auto-Fix Results:".cyan().bold());
        } else {
            println!("\n🔧 Auto-Fix Results:");
        }
        for fix in fixed_issues {
            if use_color {
                println!("  {}", format!("✓ {}", fix).green());
            } else {
                println!("  ✓ {}", fix);
            }
        }
        println!();
    }

    // Overall status header
    let (status_icon, status_text) = match report.level {
        StatusLevel::Ok => ("✅", "HEALTHY"),
        StatusLevel::Warning => ("⚠️", "WARNING"),
        StatusLevel::Error => ("❌", "ERROR"),
    };

    if use_color {
        match report.level {
            StatusLevel::Ok => {
                println!(
                    "{} {}",
                    status_icon,
                    format!("Catalyst Status: {}", status_text).green().bold()
                );
            }
            StatusLevel::Warning => {
                println!(
                    "{} {}",
                    status_icon,
                    format!("Catalyst Status: {}", status_text).yellow().bold()
                );
            }
            StatusLevel::Error => {
                println!(
                    "{} {}",
                    status_icon,
                    format!("Catalyst Status: {}", status_text).red().bold()
                );
            }
        }
    } else {
        println!("{} Catalyst Status: {}", status_icon, status_text);
    }
    println!();

    // Binaries section
    if !report.binaries.is_empty() {
        if use_color {
            println!("{}", "Binaries:".cyan().bold());
        } else {
            println!("Binaries:");
        }
        for binary in &report.binaries {
            let status_icon = if binary.exists && binary.executable {
                "✓"
            } else {
                "✗"
            };
            let status_text = if binary.exists {
                if binary.executable {
                    "found"
                } else {
                    "not executable"
                }
            } else {
                "not found"
            };

            let variant_text = if let Some(ref v) = binary.variant {
                format!(" ({})", v)
            } else {
                String::new()
            };

            if use_color {
                if binary.exists && binary.executable {
                    println!(
                        "  {} {}{}",
                        status_icon,
                        format!("{} ({})", binary.name, status_text).green(),
                        variant_text
                    );
                } else {
                    println!(
                        "  {} {}{}",
                        status_icon,
                        format!("{} ({})", binary.name, status_text).red(),
                        variant_text
                    );
                }
            } else {
                println!(
                    "  {} {} ({}){}",
                    status_icon, binary.name, status_text, variant_text
                );
            }
        }
        println!();
    }

    // Hooks section
    if !report.hooks.is_empty() {
        if use_color {
            println!("{}", "Hooks:".cyan().bold());
        } else {
            println!("Hooks:");
        }
        for hook in &report.hooks {
            let status_icon = if hook.exists && hook.executable && hook.calls_correct_binary {
                "✓"
            } else {
                "✗"
            };
            let event = hook.event.as_deref().unwrap_or("unknown");

            if use_color {
                if hook.exists && hook.executable && hook.calls_correct_binary {
                    println!("  {} {} → {}", status_icon, event.green(), hook.name);
                } else {
                    println!("  {} {} → {}", status_icon, event.red(), hook.name);
                }
            } else {
                println!("  {} {} → {}", status_icon, event, hook.name);
            }
        }
        println!();
    }

    // Skills section
    if !report.skills.is_empty() {
        if use_color {
            println!("{}", "Skills:".cyan().bold());
        } else {
            println!("Skills:");
        }
        for skill in &report.skills {
            let status_icon = if skill.has_main_file { "✓" } else { "✗" };
            let status_text = if skill.has_main_file {
                "installed"
            } else {
                "incomplete"
            };

            if use_color {
                if skill.has_main_file {
                    println!("  {} {} ({})", status_icon, skill.name.green(), status_text);
                } else {
                    println!("  {} {} ({})", status_icon, skill.name.red(), status_text);
                }
            } else {
                println!("  {} {} ({})", status_icon, skill.name, status_text);
            }
        }
        println!();
    }

    // Issues section
    if !report.issues.is_empty() {
        if use_color {
            println!("{}", "Issues:".cyan().bold());
        } else {
            println!("Issues:");
        }
        for issue in &report.issues {
            let severity_icon = match issue.severity {
                IssueSeverity::Error => "❌",
                IssueSeverity::Warning => "⚠️",
                IssueSeverity::Info => "ℹ️",
            };

            if use_color {
                let colored_desc = match issue.severity {
                    IssueSeverity::Error => issue.description.red(),
                    IssueSeverity::Warning => issue.description.yellow(),
                    IssueSeverity::Info => issue.description.blue(),
                };
                println!("  {} [{}] {}", severity_icon, issue.component, colored_desc);
            } else {
                println!(
                    "  {} [{}] {}",
                    severity_icon, issue.component, issue.description
                );
            }

            if let Some(ref fix) = issue.suggested_fix {
                if use_color {
                    println!("     {}", format!("→ {}", fix).cyan());
                } else {
                    println!("     → {}", fix);
                }
            }
        }
        println!();
    } else {
        if use_color {
            println!("{}", "Issues: None".green());
        } else {
            println!("Issues: None");
        }
        println!();
    }

    // Final message
    if report.level == StatusLevel::Ok {
        if use_color {
            println!("{}", "All systems operational! 🚀".green().bold());
        } else {
            println!("All systems operational! 🚀");
        }
    } else if report.issues.iter().any(|i| i.auto_fixable) && fixed_issues.is_empty() {
        if use_color {
            println!(
                "{}",
                "Run 'catalyst status --fix' to auto-repair fixable issues.".yellow()
            );
        } else {
            println!("Run 'catalyst status --fix' to auto-repair fixable issues.");
        }
    }
}

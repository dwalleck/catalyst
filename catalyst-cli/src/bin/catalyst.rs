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

        /// Install backend development skills
        #[arg(long)]
        backend: bool,

        /// Install frontend development skills
        #[arg(long)]
        frontend: bool,
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
            backend,
            frontend,
        } => {
            let target_dir =
                path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            if use_color {
                println!("{}", "⚠️  Not implemented yet".yellow().bold());
            } else {
                println!("⚠️  Not implemented yet");
            }
            println!("Would initialize: {:?}", target_dir);
            println!("  Interactive: {}", interactive);
            println!("  Force: {}", force);
            println!("  All skills: {}", all);
            println!("  Backend: {}", backend);
            println!("  Frontend: {}", frontend);
        }

        Commands::Status { path, fix } => {
            let target_dir =
                path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            if use_color {
                println!("{}", "⚠️  Not implemented yet".yellow().bold());
            } else {
                println!("⚠️  Not implemented yet");
            }
            println!("Would check status: {:?}", target_dir);
            println!("  Auto-fix: {}", fix);
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
                    // Load existing settings or create new (check Result to avoid TOCTOU race)
                    let (mut settings, file_existed) = match ClaudeSettings::read(&path) {
                        Ok(s) => (s, true),
                        Err(_) => (ClaudeSettings::default(), false),
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

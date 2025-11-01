//! Claude Code Settings Manager
//!
//! CLI tool for managing `.claude/settings.json` files programmatically.
//!
//! # Commands
//!
//! - `read` - Display settings file
//! - `validate` - Validate settings structure
//! - `add-hook` - Add a hook to settings
//! - `remove-hook` - Remove hooks matching pattern
//! - `merge` - Merge two settings files
//!
//! # Examples
//!
//! ```bash
//! # Read settings
//! settings-manager read .claude/settings.json
//!
//! # Add hook with dry run
//! settings-manager add-hook \
//!   --event UserPromptSubmit \
//!   --command '$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh' \
//!   --dry-run
//!
//! # Validate settings
//! settings-manager validate .claude/settings.json
//! ```

use anyhow::Result;
use catalyst_core::settings::*;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;

#[derive(Parser)]
#[command(name = "settings-manager")]
#[command(version, about = "Manage Claude Code settings.json files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    // Check for NO_COLOR environment variable
    let use_color = env::var("NO_COLOR").is_err();

    let cli = Cli::parse();

    match cli.command {
        Commands::Read { path } => {
            let settings = ClaudeSettings::read(&path)?;
            let json = serde_json::to_string_pretty(&settings)?;
            println!("{}", json);
        }

        Commands::Validate { path } => {
            let settings = ClaudeSettings::read(&path)?;
            settings.validate()?;

            if use_color {
                println!("{}", "✅ Settings file is valid".green().bold());
            } else {
                println!("✅ Settings file is valid");
            }
        }

        Commands::AddHook {
            path,
            event,
            command,
            matcher,
            dry_run,
        } => {
            // Load existing settings or create new
            let mut settings = ClaudeSettings::read(&path).unwrap_or_default();

            let hook_config = HookConfig {
                matcher,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command,
                }],
            };

            settings.add_hook(&event, hook_config);

            // Validate before writing
            settings.validate()?;

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
                    println!("{} {}", "✅ Hook added to".green().bold(), path);
                } else {
                    println!("✅ Hook added to {}", path);
                }
            }
        }

        Commands::RemoveHook {
            path,
            event,
            pattern,
            dry_run,
        } => {
            let mut settings = ClaudeSettings::read(&path)?;
            settings.remove_hook(&event, &pattern);

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

        Commands::Merge {
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
                    println!("{} {}", "✅ Settings merged to".green().bold(), output_path);
                } else {
                    println!("✅ Settings merged to {}", output_path);
                }
            }
        }
    }

    Ok(())
}

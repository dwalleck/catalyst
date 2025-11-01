//! Claude Code Settings Management
//!
//! Provides typesafe parsing, validation, and manipulation of `.claude/settings.json` files.
//!
//! # Example
//!
//! ```no_run
//! use catalyst_core::settings::*;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Read settings
//! let mut settings = ClaudeSettings::read(".claude/settings.json")?;
//!
//! // Add a hook
//! settings.add_hook("UserPromptSubmit", HookConfig {
//!     matcher: None,
//!     hooks: vec![Hook {
//!         r#type: "command".to_string(),
//!         command: "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh".to_string(),
//!     }],
//! });
//!
//! // Validate and write
//! settings.validate()?;
//! settings.write(".claude/settings.json")?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Constants for Claude Code settings validation
pub mod constants {
    /// Hook type: command
    pub const HOOK_TYPE_COMMAND: &str = "command";

    /// All valid hook types
    pub const VALID_HOOK_TYPES: &[&str] = &[HOOK_TYPE_COMMAND];

    /// Hook event: UserPromptSubmit
    pub const EVENT_USER_PROMPT_SUBMIT: &str = "UserPromptSubmit";

    /// Hook event: PostToolUse
    pub const EVENT_POST_TOOL_USE: &str = "PostToolUse";

    /// Hook event: Stop
    pub const EVENT_STOP: &str = "Stop";

    /// All valid hook events (from Claude Code documentation)
    pub const VALID_EVENTS: &[&str] = &[EVENT_USER_PROMPT_SUBMIT, EVENT_POST_TOOL_USE, EVENT_STOP];
}

/// Root settings structure for Claude Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettings {
    /// Enable all project MCP servers
    #[serde(default)]
    pub enable_all_project_mcp_servers: bool,

    /// List of enabled MCP JSON servers
    #[serde(default)]
    pub enabled_mcpjson_servers: Vec<String>,

    /// Permission configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,

    /// Hook configurations by event type
    #[serde(default)]
    pub hooks: HashMap<String, Vec<HookConfig>>,
}

/// Permission settings for tool usage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    /// List of allowed tool patterns (e.g., "Edit:*", "Write:*")
    #[serde(default)]
    pub allow: Vec<String>,

    /// Default permission mode
    #[serde(default)]
    pub default_mode: String,
}

/// Hook configuration for a specific event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookConfig {
    /// Optional matcher pattern (regex) for filtering when hook runs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,

    /// List of hooks to execute
    pub hooks: Vec<Hook>,
}

/// Individual hook definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hook {
    /// Hook type (typically "command")
    pub r#type: String,

    /// Command to execute
    pub command: String,
}

impl ClaudeSettings {
    /// Read settings from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to settings.json file
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or JSON is invalid
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).context("Failed to read settings file")?;

        let settings: ClaudeSettings =
            serde_json::from_str(&content).context("Failed to parse settings JSON")?;

        Ok(settings)
    }

    /// Write settings to a JSON file with pretty formatting
    ///
    /// Uses atomic write (temp file + rename) to prevent corruption if write fails.
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where settings.json will be written
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails, parent directory cannot be created,
    /// or file cannot be written
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self).context("Failed to serialize settings")?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create parent directories")?;
        }

        // Atomic write using temp file + rename
        let temp_path = path.with_extension("tmp");

        // Write to temp file first
        let mut temp_file =
            fs::File::create(&temp_path).context("Failed to create temporary file")?;

        temp_file
            .write_all(json.as_bytes())
            .context("Failed to write to temporary file")?;

        // Ensure data is flushed to disk
        temp_file
            .sync_all()
            .context("Failed to sync temporary file")?;

        // Drop the file handle before rename
        drop(temp_file);

        // Atomic rename (POSIX guarantees atomicity)
        fs::rename(&temp_path, path).context("Failed to rename temporary file")?;

        Ok(())
    }

    /// Add a hook configuration to a specific event
    ///
    /// # Arguments
    ///
    /// * `event` - Event name (e.g., "UserPromptSubmit", "PostToolUse", "Stop")
    /// * `hook_config` - Hook configuration to add
    pub fn add_hook(&mut self, event: &str, hook_config: HookConfig) {
        self.hooks
            .entry(event.to_string())
            .or_default()
            .push(hook_config);
    }

    /// Remove hooks matching a command pattern
    ///
    /// # Arguments
    ///
    /// * `event` - Event name to remove hooks from
    /// * `command_pattern` - Pattern to match in hook commands
    pub fn remove_hook(&mut self, event: &str, command_pattern: &str) {
        if let Some(configs) = self.hooks.get_mut(event) {
            configs.retain(|config| {
                config
                    .hooks
                    .iter()
                    .all(|h| !h.command.contains(command_pattern))
            });
        }
    }

    /// Merge another settings object into this one
    ///
    /// This preserves existing settings and adds new ones from the other settings.
    /// For collections (MCP servers, permissions, hooks), items are appended without duplication.
    ///
    /// # Arguments
    ///
    /// * `other` - Settings to merge in
    pub fn merge(&mut self, other: ClaudeSettings) {
        // Merge MCP flag (other takes precedence if true)
        if other.enable_all_project_mcp_servers {
            self.enable_all_project_mcp_servers = true;
        }

        // Merge MCP servers (deduplicate)
        for server in other.enabled_mcpjson_servers {
            if !self.enabled_mcpjson_servers.contains(&server) {
                self.enabled_mcpjson_servers.push(server);
            }
        }

        // Merge permissions
        if let Some(other_perms) = other.permissions {
            if let Some(ref mut perms) = self.permissions {
                // Merge allow patterns (deduplicate)
                for allow in other_perms.allow {
                    if !perms.allow.contains(&allow) {
                        perms.allow.push(allow);
                    }
                }
                // Other's default_mode takes precedence if non-empty
                if !other_perms.default_mode.is_empty() {
                    perms.default_mode = other_perms.default_mode;
                }
            } else {
                self.permissions = Some(other_perms);
            }
        }

        // Merge hooks (append all from other)
        for (event, configs) in other.hooks {
            self.hooks.entry(event).or_default().extend(configs);
        }
    }

    /// Validate the settings structure
    ///
    /// Checks:
    /// - Hook event names are valid Claude Code events
    /// - Hook matcher patterns are valid regex
    /// - Hook arrays are not empty
    /// - Hook types are supported
    ///
    /// # Errors
    ///
    /// Returns error if validation fails, with helpful messages showing valid options
    pub fn validate(&self) -> Result<()> {
        use constants::*;

        // Validate hooks
        for (event, configs) in &self.hooks {
            // Validate event name
            if !VALID_EVENTS.contains(&event.as_str()) {
                anyhow::bail!(
                    "Unknown event '{}'. Valid events: {}",
                    event,
                    VALID_EVENTS.join(", ")
                );
            }

            for config in configs {
                // Validate matcher is valid regex if present
                if let Some(ref matcher) = config.matcher {
                    regex::Regex::new(matcher).context(format!(
                        "Invalid matcher regex in {} hook: {}",
                        event, matcher
                    ))?;
                }

                // Validate hooks array not empty
                if config.hooks.is_empty() {
                    anyhow::bail!("Empty hooks array in {} event", event);
                }

                // Validate hook types
                for hook in &config.hooks {
                    if !VALID_HOOK_TYPES.contains(&hook.r#type.as_str()) {
                        anyhow::bail!(
                            "Unknown hook type '{}' in {} event. Valid types: {}",
                            hook.r#type,
                            event,
                            VALID_HOOK_TYPES.join(", ")
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_settings() {
        let json = r#"{
            "hooks": {}
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert!(!settings.enable_all_project_mcp_servers);
        assert!(settings.enabled_mcpjson_servers.is_empty());
        assert!(settings.hooks.is_empty());
    }

    #[test]
    fn test_parse_full_settings() {
        let json = r#"{
            "enableAllProjectMcpServers": true,
            "enabledMcpjsonServers": ["mysql", "playwright"],
            "permissions": {
                "allow": ["Edit:*", "Write:*"],
                "defaultMode": "acceptEdits"
            },
            "hooks": {
                "UserPromptSubmit": [{
                    "hooks": [{
                        "type": "command",
                        "command": "test.sh"
                    }]
                }]
            }
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert!(settings.enable_all_project_mcp_servers);
        assert_eq!(settings.enabled_mcpjson_servers.len(), 2);
        assert!(settings.permissions.is_some());
        assert_eq!(settings.hooks.len(), 1);
    }

    #[test]
    fn test_add_hook() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "test.sh".to_string(),
                }],
            },
        );

        assert_eq!(settings.hooks.len(), 1);
        assert_eq!(settings.hooks.get("UserPromptSubmit").unwrap().len(), 1);
    }

    #[test]
    fn test_remove_hook() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "skill-activation-prompt.sh".to_string(),
                }],
            },
        );

        settings.remove_hook("UserPromptSubmit", "skill-activation");
        assert!(settings.hooks.get("UserPromptSubmit").unwrap().is_empty());
    }

    #[test]
    fn test_merge_mcp_servers() {
        let mut base = ClaudeSettings::default();
        base.enabled_mcpjson_servers.push("mysql".to_string());

        let mut other = ClaudeSettings::default();
        other.enabled_mcpjson_servers.push("playwright".to_string());
        other.enabled_mcpjson_servers.push("mysql".to_string()); // Duplicate

        base.merge(other);

        assert_eq!(base.enabled_mcpjson_servers.len(), 2);
        assert!(base.enabled_mcpjson_servers.contains(&"mysql".to_string()));
        assert!(base
            .enabled_mcpjson_servers
            .contains(&"playwright".to_string()));
    }

    #[test]
    fn test_merge_permissions() {
        let mut base = ClaudeSettings::default();
        base.permissions = Some(Permissions {
            allow: vec!["Edit:*".to_string()],
            default_mode: "ask".to_string(),
        });

        let mut other = ClaudeSettings::default();
        other.permissions = Some(Permissions {
            allow: vec!["Write:*".to_string()],
            default_mode: "acceptEdits".to_string(),
        });

        base.merge(other);

        let perms = base.permissions.unwrap();
        assert_eq!(perms.allow.len(), 2);
        assert_eq!(perms.default_mode, "acceptEdits");
    }

    #[test]
    fn test_merge_hooks() {
        let mut base = ClaudeSettings::default();
        base.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "hook1.sh".to_string(),
                }],
            },
        );

        let mut other = ClaudeSettings::default();
        other.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "hook2.sh".to_string(),
                }],
            },
        );

        base.merge(other);

        assert_eq!(base.hooks.get("UserPromptSubmit").unwrap().len(), 2);
    }

    #[test]
    fn test_validation_success() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: Some("Edit|Write".to_string()),
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "test.sh".to_string(),
                }],
            },
        );

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_regex() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: Some("[invalid regex".to_string()),
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "test.sh".to_string(),
                }],
            },
        );

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validation_empty_hooks_array() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![],
            },
        );

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_hook_type() {
        let mut settings = ClaudeSettings::default();
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![Hook {
                    r#type: "invalid_type".to_string(),
                    command: "test.sh".to_string(),
                }],
            },
        );

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut settings = ClaudeSettings::default();
        settings.enable_all_project_mcp_servers = true;
        settings.enabled_mcpjson_servers.push("mysql".to_string());
        settings.add_hook(
            "UserPromptSubmit",
            HookConfig {
                matcher: None,
                hooks: vec![Hook {
                    r#type: "command".to_string(),
                    command: "test.sh".to_string(),
                }],
            },
        );

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: ClaudeSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, parsed);
    }

    // Integration tests for file I/O
    mod integration {
        use super::*;
        use tempfile::TempDir;

        #[test]
        fn test_write_read_roundtrip() {
            let temp_dir = TempDir::new().unwrap();
            let settings_path = temp_dir.path().join("settings.json");

            let mut settings = ClaudeSettings::default();
            settings.enable_all_project_mcp_servers = true;
            settings.enabled_mcpjson_servers.push("mysql".to_string());
            settings.add_hook(
                "UserPromptSubmit",
                HookConfig {
                    matcher: Some("Edit|Write".to_string()),
                    hooks: vec![Hook {
                        r#type: "command".to_string(),
                        command: "test.sh".to_string(),
                    }],
                },
            );

            // Write settings
            settings.write(&settings_path).unwrap();

            // Read back
            let loaded = ClaudeSettings::read(&settings_path).unwrap();

            assert_eq!(settings, loaded);
        }

        #[test]
        fn test_parent_directory_creation() {
            let temp_dir = TempDir::new().unwrap();
            let nested_path = temp_dir.path().join("nested/deep/path/settings.json");

            let settings = ClaudeSettings::default();

            // Should create all parent directories
            settings.write(&nested_path).unwrap();

            assert!(nested_path.exists());

            // Verify it can be read back
            let loaded = ClaudeSettings::read(&nested_path).unwrap();
            assert_eq!(settings, loaded);
        }

        #[test]
        fn test_read_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let nonexistent = temp_dir.path().join("does-not-exist.json");

            let result = ClaudeSettings::read(&nonexistent);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Failed to read"));
        }

        #[test]
        fn test_read_invalid_json() {
            let temp_dir = TempDir::new().unwrap();
            let invalid_json_path = temp_dir.path().join("invalid.json");

            // Write invalid JSON
            fs::write(&invalid_json_path, "{ this is not valid json }").unwrap();

            let result = ClaudeSettings::read(&invalid_json_path);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Failed to parse"));
        }

        #[test]
        fn test_overwrite_existing_file() {
            let temp_dir = TempDir::new().unwrap();
            let settings_path = temp_dir.path().join("settings.json");

            // Write initial settings
            let mut settings1 = ClaudeSettings::default();
            settings1.enabled_mcpjson_servers.push("mysql".to_string());
            settings1.write(&settings_path).unwrap();

            // Overwrite with new settings
            let mut settings2 = ClaudeSettings::default();
            settings2
                .enabled_mcpjson_servers
                .push("playwright".to_string());
            settings2.write(&settings_path).unwrap();

            // Verify new settings were written
            let loaded = ClaudeSettings::read(&settings_path).unwrap();
            assert_eq!(loaded.enabled_mcpjson_servers.len(), 1);
            assert_eq!(loaded.enabled_mcpjson_servers[0], "playwright");
        }

        #[test]
        fn test_atomic_write_no_partial_files() {
            let temp_dir = TempDir::new().unwrap();
            let settings_path = temp_dir.path().join("settings.json");

            let settings = ClaudeSettings::default();
            settings.write(&settings_path).unwrap();

            // Check no temp files left behind
            let entries: Vec<_> = fs::read_dir(temp_dir.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();

            // Should only be settings.json, no .tmp files
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].file_name(), "settings.json");
        }
    }
}

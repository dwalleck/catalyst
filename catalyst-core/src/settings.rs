//! Claude Code Settings Management
//!
//! Provides typesafe parsing, validation, and manipulation of `.claude/settings.json` files.
//!
//! # Example
//!
//! ```rust
//! use catalyst_core::settings::*;
//!
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
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    /// # Arguments
    ///
    /// * `path` - Path where settings.json will be written
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails or file cannot be written
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("Failed to serialize settings")?;

        fs::write(path.as_ref(), json).context("Failed to write settings file")?;

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
    /// - Hook matcher patterns are valid regex
    /// - Hook arrays are not empty
    /// - Hook types are supported
    ///
    /// # Errors
    ///
    /// Returns error if validation fails
    pub fn validate(&self) -> Result<()> {
        // Validate hooks
        for (event, configs) in &self.hooks {
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
                    if hook.r#type != "command" {
                        anyhow::bail!("Unknown hook type '{}' in {} event", hook.r#type, event);
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
}

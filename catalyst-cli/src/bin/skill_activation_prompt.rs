use colored::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
enum SkillActivationError {
    #[error("[SA001] Failed to read input from stdin")]
    StdinRead(#[from] io::Error),

    #[error("[SA002] Invalid JSON input from hook: {0}\nCheck that the hook is passing valid JSON format")]
    InvalidHookInput(#[source] serde_json::Error),

    #[error("[SA003] Skill rules file not found at {}\nMake sure the file exists and CLAUDE_PROJECT_DIR is set correctly\nTry: mkdir -p $(dirname {}) && touch {}", path.display(), path.display(), path.display())]
    RulesNotFound { path: PathBuf },

    #[error("[SA004] Failed to read skill rules from {}: {source}\nCheck file permissions\nTry: chmod 644 {}", path.display(), path.display())]
    RulesReadFailed {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("[SA005] Invalid JSON in skill rules file: {0}\nCheck the syntax in .claude/skills/skill-rules.json\nTry: cat {} | jq .", path.display())]
    InvalidRulesJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

/// Input data from Claude Code's UserPromptSubmit hook
///
/// Note: Fields prefixed with underscore are part of the hook's JSON schema
/// but not currently used by this binary. They're kept in the struct to:
/// 1. Maintain complete schema compatibility with Claude Code
/// 2. Enable future features (e.g., session-aware caching, permission checks)
/// 3. Ensure deserialization succeeds even if Claude Code adds more fields
///
/// If these fields are needed in the future, remove the underscore prefix.
#[derive(Debug, Deserialize)]
struct HookInput {
    /// Session ID for the current Claude Code session (reserved for future use)
    #[serde(rename = "session_id")]
    _session_id: String,

    /// Path to the conversation transcript (reserved for future use)
    #[serde(rename = "transcript_path")]
    _transcript_path: String,

    /// Current working directory when the hook was triggered
    #[serde(rename = "cwd")]
    cwd: String,

    /// Permission mode from Claude Code settings (reserved for future use)
    #[serde(rename = "permission_mode")]
    _permission_mode: String,

    /// The user's prompt text to analyze for skill activation
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct PromptTriggers {
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default, rename = "intentPatterns")]
    intent_patterns: Vec<String>,
}

// Compiled version of PromptTriggers with pre-compiled regexes and lowercased keywords
struct CompiledTriggers {
    keywords_lower: Vec<String>, // Pre-lowercased for efficient substring matching
    intent_regexes: Vec<Regex>,
}

impl CompiledTriggers {
    fn from_triggers(triggers: &PromptTriggers) -> Self {
        let intent_regexes = triggers
            .intent_patterns
            .iter()
            .filter_map(|pattern| match Regex::new(pattern) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    tracing::warn!(
                        pattern = %pattern,
                        error = %e,
                        "Failed to compile intent pattern regex, skipping"
                    );
                    None
                }
            })
            .collect();

        // Pre-lowercase keywords once during compilation (eliminates N allocations per check)
        // Note: Duplicate keywords (including case variations like "Backend" and "backend")
        // are intentionally NOT deduplicated for these reasons:
        // 1. Simplicity - avoids additional HashSet allocation and deduplication logic
        // 2. Performance - keyword lists are typically small (<10 items), so duplicate checks
        //    have negligible impact on matching performance (still O(n) substring checks)
        // 3. Correctness - preserves user's original configuration without modification
        let keywords_lower = triggers
            .keywords
            .iter()
            .map(|kw| kw.to_lowercase())
            .collect();

        Self {
            keywords_lower,
            intent_regexes,
        }
    }
}

/// Priority levels for skill activation (PR feedback - extracted magic strings)
///
/// These priority levels determine the order and prominence of skill suggestions
/// in the activation output. Higher priorities appear first and with more emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl Priority {
    /// Parse priority from string (case-insensitive)
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => {
                tracing::warn!(
                    priority = %s,
                    "Unknown priority level, defaulting to Medium"
                );
                Priority::Medium
            }
        }
    }

    /// Convert to string for display (reserved for future use)
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }
}

#[derive(Debug, Deserialize)]
struct SkillRule {
    #[serde(rename = "type")]
    r#_type: String,
    #[serde(rename = "enforcement")]
    _enforcement: String,
    #[serde(deserialize_with = "deserialize_priority")]
    priority: Priority,
    #[serde(rename = "promptTriggers")]
    prompt_triggers: Option<PromptTriggers>,
}

/// Custom deserializer for Priority enum from string
fn deserialize_priority<'de, D>(deserializer: D) -> Result<Priority, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Priority::from_str(&s))
}

struct CompiledSkillRule {
    priority: Priority,
    compiled_triggers: Option<CompiledTriggers>,
}

impl CompiledSkillRule {
    fn from_rule(rule: &SkillRule) -> Self {
        Self {
            priority: rule.priority,
            compiled_triggers: rule
                .prompt_triggers
                .as_ref()
                .map(CompiledTriggers::from_triggers),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SkillRules {
    #[serde(rename = "version")]
    _version: String,
    skills: HashMap<String, SkillRule>,
}

/// Maps io::Error to SkillActivationError for file reading operations
fn map_file_read_error(path: PathBuf, error: io::Error) -> SkillActivationError {
    if error.kind() == io::ErrorKind::NotFound {
        error!(
            error_code = "SA003",
            error_kind = "RulesNotFound",
            path = %path.display(),
            "Skill rules file not found"
        );
        SkillActivationError::RulesNotFound { path }
    } else {
        error!(
            error_code = "SA004",
            error_kind = "RulesReadFailed",
            path = %path.display(),
            io_error = %error,
            "Failed to read skill rules file"
        );
        SkillActivationError::RulesReadFailed {
            path,
            source: error,
        }
    }
}

#[derive(Debug)]
struct MatchedSkill {
    name: String,
    _match_type: String,
    priority: Priority,
}

fn run() -> Result<(), SkillActivationError> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|e| {
        error!(
            error_code = "SA001",
            error_kind = "StdinRead",
            io_error = %e,
            "Failed to read input from stdin"
        );
        SkillActivationError::StdinRead(e)
    })?;

    let data: HookInput = serde_json::from_str(&input).map_err(|e| {
        error!(
            error_code = "SA002",
            error_kind = "InvalidHookInput",
            json_error = %e,
            "Invalid JSON input from hook"
        );
        SkillActivationError::InvalidHookInput(e)
    })?;

    // Phase 2.5: Lowercase prompt once for efficient substring matching
    let prompt = &data.prompt;
    let prompt_lower = prompt.to_lowercase();

    // Load skill rules with multi-directory support
    //
    // Path Resolution Priority (PR feedback - detailed explanation):
    // 1. cwd/.claude/skills/skill-rules.json (HIGHEST priority)
    //    - Supports Claude Code's /add-dir command where users work with multiple projects
    //    - Each directory can have its own skill configuration
    //    - Example: Main project uses backend skills, added dir uses frontend skills
    //
    // 2. $CLAUDE_PROJECT_DIR/.claude/skills/skill-rules.json (MEDIUM priority)
    //    - Falls back to the primary project directory when set
    //    - Useful when hooks are invoked from nested directories
    //    - Ensures consistent skill rules across the main project
    //
    // 3. cwd/.claude/skills/skill-rules.json (LOWEST priority, same as #1)
    //    - If CLAUDE_PROJECT_DIR is not set, uses current directory
    //    - This is the default behavior for single-directory workflows
    //
    // Why this order matters:
    // - /add-dir workflows: User has catalyst/ and mental-health-bar-rs/ both open
    // - When in mental-health-bar-rs/, we should use THAT directory's skill rules
    // - Not the catalyst/ directory's rules, even if CLAUDE_PROJECT_DIR=catalyst
    // - This enables polyglot workflows (Rust + TypeScript) with appropriate skills per dir
    let rules_path = {
        let cwd_path = PathBuf::from(&data.cwd)
            .join(".claude")
            .join("skills")
            .join("skill-rules.json");

        if cwd_path.exists() {
            debug!("Using skill-rules.json from cwd: {}", cwd_path.display());
            cwd_path
        } else {
            let project_dir = env::var("CLAUDE_PROJECT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(&data.cwd));

            let fallback_path = project_dir
                .join(".claude")
                .join("skills")
                .join("skill-rules.json");

            debug!(
                "Using skill-rules.json from project dir: {}",
                fallback_path.display()
            );
            fallback_path
        }
    };

    let rules_content =
        fs::read_to_string(&rules_path).map_err(|e| map_file_read_error(rules_path.clone(), e))?;
    let rules: SkillRules = serde_json::from_str(&rules_content).map_err(|source| {
        error!(
            error_code = "SA005",
            error_kind = "InvalidRulesJson",
            path = %rules_path.display(),
            json_error = %source,
            "Invalid JSON in skill rules file"
        );
        SkillActivationError::InvalidRulesJson {
            path: rules_path.clone(),
            source,
        }
    })?;

    debug!("Loaded {} skills from rules", rules.skills.len());

    // Pre-compile all regex patterns (CRITICAL PERFORMANCE IMPROVEMENT)
    let compiled_rules: HashMap<String, CompiledSkillRule> = rules
        .skills
        .iter()
        .map(|(name, rule)| (name.clone(), CompiledSkillRule::from_rule(rule)))
        .collect();

    let mut matched_skills = Vec::new();

    // Check each skill for matches using pre-compiled regexes
    for (skill_name, compiled_rule) in &compiled_rules {
        if let Some(triggers) = &compiled_rule.compiled_triggers {
            // Case-insensitive keyword matching using pre-lowercased keywords
            let keyword_match = triggers
                .keywords_lower
                .iter()
                .any(|kw_lower| prompt_lower.contains(kw_lower));

            if keyword_match {
                debug!(skill = %skill_name, match_type = "keyword", "Skill matched");
                matched_skills.push(MatchedSkill {
                    name: skill_name.clone(),
                    _match_type: "keyword".to_string(),
                    priority: compiled_rule.priority,
                });
                continue;
            }

            // Intent pattern matching with pre-compiled regexes
            // Note: Regex matching is already case-insensitive if patterns use (?i)
            let intent_match = triggers
                .intent_regexes
                .iter()
                .any(|regex| regex.is_match(prompt));

            if intent_match {
                debug!(skill = %skill_name, match_type = "intent", "Skill matched");
                matched_skills.push(MatchedSkill {
                    name: skill_name.clone(),
                    _match_type: "intent".to_string(),
                    priority: compiled_rule.priority,
                });
            }
        }
    }

    // Generate output if matches found
    if !matched_skills.is_empty() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 SKILL ACTIVATION CHECK");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Group by priority (using enum for type safety - PR feedback)
        let critical: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == Priority::Critical)
            .collect();
        let high: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == Priority::High)
            .collect();
        let medium: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == Priority::Medium)
            .collect();
        let low: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == Priority::Low)
            .collect();

        if !critical.is_empty() {
            println!("{}", "⚠️ CRITICAL SKILLS (REQUIRED):".red().bold());
            for skill in critical {
                println!("  → {}", skill.name.yellow());
            }
            println!();
        }

        if !high.is_empty() {
            println!("{}", "📚 RECOMMENDED SKILLS:".blue().bold());
            for skill in high {
                println!("  → {}", skill.name.cyan());
            }
            println!();
        }

        if !medium.is_empty() {
            println!("{}", "💡 SUGGESTED SKILLS:".green().bold());
            for skill in medium {
                println!("  → {}", skill.name.bright_green());
            }
            println!();
        }

        if !low.is_empty() {
            println!("{}", "📌 OPTIONAL SKILLS:".white().bold());
            for skill in low {
                println!("  → {}", skill.name.white());
            }
            println!();
        }

        println!(
            "{}",
            "ACTION: Use Skill tool BEFORE responding"
                .bright_yellow()
                .bold()
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matching_case_insensitive() {
        let triggers = PromptTriggers {
            keywords: vec!["backend".to_string(), "API".to_string()],
            intent_patterns: vec![],
        };

        let compiled = CompiledTriggers::from_triggers(&triggers);

        // Test lowercase keyword
        assert!(compiled
            .keywords_lower
            .iter()
            .any(|kw| "create a backend service".to_lowercase().contains(kw)));

        // Test uppercase keyword match
        assert!(compiled
            .keywords_lower
            .iter()
            .any(|kw| "BUILD AN API ENDPOINT".to_lowercase().contains(kw)));

        // Test mixed case
        assert!(compiled
            .keywords_lower
            .iter()
            .any(|kw| "Add Backend logic".to_lowercase().contains(kw)));

        // Test non-match
        assert!(!compiled
            .keywords_lower
            .iter()
            .any(|kw| "frontend component".to_lowercase().contains(kw)));
    }

    #[test]
    fn test_intent_pattern_matching() {
        let triggers = PromptTriggers {
            keywords: vec![],
            intent_patterns: vec![
                r"(?i)create.*controller".to_string(),
                r"(?i)add.*route".to_string(),
            ],
        };

        let compiled = CompiledTriggers::from_triggers(&triggers);

        // Test first pattern
        assert!(compiled
            .intent_regexes
            .iter()
            .any(|r| r.is_match("create a new controller")));

        // Test case insensitivity
        assert!(compiled
            .intent_regexes
            .iter()
            .any(|r| r.is_match("CREATE USER CONTROLLER")));

        // Test second pattern
        assert!(compiled
            .intent_regexes
            .iter()
            .any(|r| r.is_match("add a new route for users")));

        // Test non-match
        assert!(!compiled
            .intent_regexes
            .iter()
            .any(|r| r.is_match("delete a component")));
    }

    #[test]
    fn test_compiled_triggers_from_triggers() {
        let triggers = PromptTriggers {
            keywords: vec!["Backend".to_string(), "API".to_string()],
            intent_patterns: vec![r"test.*pattern".to_string()],
        };

        let compiled = CompiledTriggers::from_triggers(&triggers);

        // Verify keywords are lowercased
        assert_eq!(compiled.keywords_lower.len(), 2);
        assert_eq!(compiled.keywords_lower[0], "backend");
        assert_eq!(compiled.keywords_lower[1], "api");

        // Verify regex compiled
        assert_eq!(compiled.intent_regexes.len(), 1);
    }

    #[test]
    fn test_invalid_regex_patterns_are_skipped() {
        let triggers = PromptTriggers {
            keywords: vec![],
            intent_patterns: vec![
                r"(?i)valid.*pattern".to_string(),
                r"[invalid(".to_string(), // Invalid regex
                r"(?i)another.*valid".to_string(),
            ],
        };

        let compiled = CompiledTriggers::from_triggers(&triggers);

        // Should only have 2 valid regexes (invalid one skipped)
        assert_eq!(compiled.intent_regexes.len(), 2);
    }

    #[test]
    fn test_duplicate_keywords_case_insensitive() {
        let triggers = PromptTriggers {
            keywords: vec![
                "backend".to_string(),
                "Backend".to_string(),
                "BACKEND".to_string(),
                "api".to_string(),
            ],
            intent_patterns: vec![],
        };

        let compiled = CompiledTriggers::from_triggers(&triggers);

        // All keywords are lowercased, so duplicates remain (no deduplication)
        assert_eq!(compiled.keywords_lower.len(), 4);
        assert_eq!(compiled.keywords_lower[0], "backend");
        assert_eq!(compiled.keywords_lower[1], "backend");
        assert_eq!(compiled.keywords_lower[2], "backend");
        assert_eq!(compiled.keywords_lower[3], "api");

        // Matching still works correctly with duplicates
        let prompt = "create a BACKEND service";
        assert!(compiled
            .keywords_lower
            .iter()
            .any(|kw| prompt.to_lowercase().contains(kw)));
    }

    #[test]
    fn test_empty_triggers() {
        let triggers = PromptTriggers {
            keywords: vec![],
            intent_patterns: vec![],
        };

        let compiled = CompiledTriggers::from_triggers(&triggers);

        assert_eq!(compiled.keywords_lower.len(), 0);
        assert_eq!(compiled.intent_regexes.len(), 0);
    }

    #[test]
    fn test_compiled_skill_rule_creation() {
        // Test priority deserialization and compilation
        let json = r#"{
            "type": "UserPromptSubmit",
            "enforcement": "suggest",
            "priority": "high",
            "promptTriggers": {
                "keywords": ["test"],
                "intentPatterns": []
            }
        }"#;

        let rule: SkillRule = serde_json::from_str(json).unwrap();
        let compiled = CompiledSkillRule::from_rule(&rule);

        assert_eq!(compiled.priority, Priority::High);
        assert!(compiled.compiled_triggers.is_some());
    }

    #[test]
    fn test_compiled_skill_rule_without_triggers() {
        let json = r#"{
            "type": "UserPromptSubmit",
            "enforcement": "suggest",
            "priority": "medium"
        }"#;

        let rule: SkillRule = serde_json::from_str(json).unwrap();
        let compiled = CompiledSkillRule::from_rule(&rule);

        assert_eq!(compiled.priority, Priority::Medium);
        assert!(compiled.compiled_triggers.is_none());
    }

    #[test]
    fn test_priority_enum_parsing() {
        // Test case-insensitive priority parsing
        assert_eq!(Priority::from_str("critical"), Priority::Critical);
        assert_eq!(Priority::from_str("CRITICAL"), Priority::Critical);
        assert_eq!(Priority::from_str("High"), Priority::High);
        assert_eq!(Priority::from_str("high"), Priority::High);
        assert_eq!(Priority::from_str("medium"), Priority::Medium);
        assert_eq!(Priority::from_str("low"), Priority::Low);
        // Unknown priority defaults to Medium
        assert_eq!(Priority::from_str("unknown"), Priority::Medium);
    }

    #[test]
    fn test_hook_input_deserialization() {
        let json = r#"{
            "session_id": "test-123",
            "transcript_path": "/path/to/transcript",
            "cwd": "/project",
            "permission_mode": "normal",
            "prompt": "create a backend service"
        }"#;

        let result: Result<HookInput, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.prompt, "create a backend service");
    }

    #[test]
    fn test_malformed_json_input() {
        let json = r#"{
            "session_id": "test-123",
            "invalid_field_structure
        }"#;

        let result: Result<HookInput, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_rules_deserialization() {
        let json = r#"{
            "version": "1.0",
            "skills": {
                "backend-dev-guidelines": {
                    "type": "UserPromptSubmit",
                    "enforcement": "suggest",
                    "priority": "high",
                    "promptTriggers": {
                        "keywords": ["backend", "API"],
                        "intentPatterns": ["(?i)create.*controller"]
                    }
                }
            }
        }"#;

        let result: Result<SkillRules, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let rules = result.unwrap();
        assert_eq!(rules.skills.len(), 1);
        assert!(rules.skills.contains_key("backend-dev-guidelines"));
    }

    #[test]
    fn test_error_message_rules_not_found() {
        let path = PathBuf::from("/nonexistent/.claude/skills/skill-rules.json");
        let error = SkillActivationError::RulesNotFound { path };

        let error_msg = error.to_string();
        assert!(error_msg.contains("[SA003]"));
        assert!(error_msg.contains("Skill rules file not found"));
        assert!(error_msg.contains("/nonexistent/.claude/skills/skill-rules.json"));
        assert!(error_msg.contains("Make sure the file exists"));
        assert!(error_msg.contains("CLAUDE_PROJECT_DIR"));
        assert!(error_msg.contains("Try: mkdir -p"));
        assert!(error_msg.contains("touch"));
    }

    #[test]
    fn test_error_message_rules_read_failed() {
        let path = PathBuf::from("/test/skill-rules.json");
        let io_err = io::Error::other("disk error");
        let error = SkillActivationError::RulesReadFailed {
            path,
            source: io_err,
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("[SA004]"));
        assert!(error_msg.contains("Failed to read skill rules"));
        assert!(error_msg.contains("/test/skill-rules.json"));
        assert!(error_msg.contains("disk error"));
        assert!(error_msg.contains("Check file permissions"));
        assert!(error_msg.contains("Try: chmod 644"));
    }

    #[test]
    fn test_error_message_invalid_hook_input() {
        let json_err = serde_json::from_str::<HookInput>("invalid").unwrap_err();
        let error = SkillActivationError::InvalidHookInput(json_err);

        let error_msg = error.to_string();
        assert!(error_msg.contains("[SA002]"));
        assert!(error_msg.contains("Invalid JSON input from hook"));
        assert!(error_msg.contains("Check that the hook is passing valid JSON format"));
    }

    #[test]
    fn test_error_message_invalid_rules_json() {
        let path = PathBuf::from(".claude/skills/skill-rules.json");
        let json_err = serde_json::from_str::<SkillRules>("invalid").unwrap_err();
        let error = SkillActivationError::InvalidRulesJson {
            path,
            source: json_err,
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("[SA005]"));
        assert!(error_msg.contains("Invalid JSON in skill rules file"));
        assert!(error_msg.contains("Check the syntax"));
        assert!(error_msg.contains(".claude/skills/skill-rules.json"));
        assert!(error_msg.contains("Try: cat"));
        assert!(error_msg.contains("jq"));
    }

    #[test]
    fn test_map_file_read_error_not_found() {
        let path = PathBuf::from("/test/path");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = map_file_read_error(path.clone(), io_err);

        match error {
            SkillActivationError::RulesNotFound { path: err_path } => {
                assert_eq!(err_path, path);
            }
            _ => panic!("Expected RulesNotFound error"),
        }
    }

    #[test]
    fn test_map_file_read_error_other() {
        let path = PathBuf::from("/test/path");
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let error = map_file_read_error(path.clone(), io_err);

        match error {
            SkillActivationError::RulesReadFailed {
                path: err_path,
                source,
            } => {
                assert_eq!(err_path, path);
                assert_eq!(source.to_string(), "access denied");
            }
            _ => panic!("Expected RulesReadFailed error"),
        }
    }
}

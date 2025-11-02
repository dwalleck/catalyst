use anyhow::{Context, Result};
use colored::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct HookInput {
    #[serde(rename = "session_id")]
    _session_id: String,
    #[serde(rename = "transcript_path")]
    _transcript_path: String,
    #[serde(rename = "cwd")]
    _cwd: String,
    #[serde(rename = "permission_mode")]
    _permission_mode: String,
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
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        // Pre-lowercase keywords once during compilation (eliminates N allocations per check)
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

#[derive(Debug, Deserialize)]
struct SkillRule {
    #[serde(rename = "type")]
    r#_type: String,
    #[serde(rename = "enforcement")]
    _enforcement: String,
    priority: String,
    #[serde(rename = "promptTriggers")]
    prompt_triggers: Option<PromptTriggers>,
}

struct CompiledSkillRule {
    priority: String,
    compiled_triggers: Option<CompiledTriggers>,
}

impl CompiledSkillRule {
    fn from_rule(rule: &SkillRule) -> Self {
        Self {
            priority: rule.priority.clone(),
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

#[derive(Debug)]
struct MatchedSkill {
    name: String,
    _match_type: String,
    priority: String,
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let data: HookInput = serde_json::from_str(&input).context("Failed to parse hook input")?;

    // Phase 2.5: Lowercase prompt once for efficient substring matching
    let prompt = &data.prompt;
    let prompt_lower = prompt.to_lowercase();

    // Load skill rules (cross-platform path handling)
    let project_dir = env::var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let rules_path = project_dir
        .join(".claude")
        .join("skills")
        .join("skill-rules.json");

    let rules_content =
        fs::read_to_string(&rules_path).context("Failed to read skill-rules.json")?;
    let rules: SkillRules =
        serde_json::from_str(&rules_content).context("Failed to parse skill-rules.json")?;

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
                    priority: compiled_rule.priority.clone(),
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
                    priority: compiled_rule.priority.clone(),
                });
            }
        }
    }

    // Generate output if matches found
    if !matched_skills.is_empty() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 SKILL ACTIVATION CHECK");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Group by priority
        let critical: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == "critical")
            .collect();
        let high: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == "high")
            .collect();
        let medium: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == "medium")
            .collect();
        let low: Vec<_> = matched_skills
            .iter()
            .filter(|s| s.priority == "low")
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
        let rule = SkillRule {
            r#_type: "UserPromptSubmit".to_string(),
            _enforcement: "suggest".to_string(),
            priority: "high".to_string(),
            prompt_triggers: Some(PromptTriggers {
                keywords: vec!["test".to_string()],
                intent_patterns: vec![],
            }),
        };

        let compiled = CompiledSkillRule::from_rule(&rule);

        assert_eq!(compiled.priority, "high");
        assert!(compiled.compiled_triggers.is_some());
    }

    #[test]
    fn test_compiled_skill_rule_without_triggers() {
        let rule = SkillRule {
            r#_type: "UserPromptSubmit".to_string(),
            _enforcement: "suggest".to_string(),
            priority: "medium".to_string(),
            prompt_triggers: None,
        };

        let compiled = CompiledSkillRule::from_rule(&rule);

        assert_eq!(compiled.priority, "medium");
        assert!(compiled.compiled_triggers.is_none());
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
}

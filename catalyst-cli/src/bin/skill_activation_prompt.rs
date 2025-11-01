use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct HookInput {
    _session_id: String,
    _transcript_path: String,
    _cwd: String,
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

// Compiled version of PromptTriggers with pre-compiled regexes
struct CompiledTriggers {
    keywords: Vec<String>,
    intent_regexes: Vec<Regex>,
}

impl CompiledTriggers {
    fn from_triggers(triggers: &PromptTriggers) -> Self {
        let intent_regexes = triggers
            .intent_patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        Self {
            keywords: triggers.keywords.clone(),
            intent_regexes,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SkillRule {
    #[serde(rename = "type")]
    r#_type: String,
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
    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let data: HookInput = serde_json::from_str(&input).context("Failed to parse hook input")?;

    let prompt = data.prompt.to_lowercase();

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
            // Keyword matching
            let keyword_match = triggers
                .keywords
                .iter()
                .any(|kw| prompt.contains(&kw.to_lowercase()));

            if keyword_match {
                matched_skills.push(MatchedSkill {
                    name: skill_name.clone(),
                    _match_type: "keyword".to_string(),
                    priority: compiled_rule.priority.clone(),
                });
                continue;
            }

            // Intent pattern matching with pre-compiled regexes
            let intent_match = triggers
                .intent_regexes
                .iter()
                .any(|regex| regex.is_match(&prompt));

            if intent_match {
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
            println!("⚠️ CRITICAL SKILLS (REQUIRED):");
            for skill in critical {
                println!("  → {}", skill.name);
            }
            println!();
        }

        if !high.is_empty() {
            println!("📚 RECOMMENDED SKILLS:");
            for skill in high {
                println!("  → {}", skill.name);
            }
            println!();
        }

        if !medium.is_empty() {
            println!("💡 SUGGESTED SKILLS:");
            for skill in medium {
                println!("  → {}", skill.name);
            }
            println!();
        }

        if !low.is_empty() {
            println!("📌 OPTIONAL SKILLS:");
            for skill in low {
                println!("  → {}", skill.name);
            }
            println!();
        }

        println!("ACTION: Use Skill tool BEFORE responding");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    Ok(())
}

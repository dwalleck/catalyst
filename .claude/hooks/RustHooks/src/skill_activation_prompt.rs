use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use regex::Regex;

#[derive(Debug, Deserialize)]
struct HookInput {
    session_id: String,
    transcript_path: String,
    cwd: String,
    permission_mode: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct PromptTriggers {
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default, rename = "intentPatterns")]
    intent_patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SkillRule {
    r#type: String,
    enforcement: String,
    priority: String,
    #[serde(rename = "promptTriggers")]
    prompt_triggers: Option<PromptTriggers>,
}

#[derive(Debug, Deserialize)]
struct SkillRules {
    version: String,
    skills: HashMap<String, SkillRule>,
}

#[derive(Debug)]
struct MatchedSkill {
    name: String,
    match_type: String,
    priority: String,
}

fn main() -> io::Result<()> {
    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let data: HookInput = serde_json::from_str(&input)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let prompt = data.prompt.to_lowercase();

    // Load skill rules
    let project_dir = env::var("CLAUDE_PROJECT_DIR")
        .unwrap_or_else(|_| String::from("/home/project"));
    let rules_path = format!("{project_dir}/.claude/skills/skill-rules.json");

    let rules_content = fs::read_to_string(&rules_path)?;
    let rules: SkillRules = serde_json::from_str(&rules_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut matched_skills = Vec::new();

    // Check each skill for matches
    for (skill_name, config) in &rules.skills {
        if let Some(triggers) = &config.prompt_triggers {
            // Keyword matching
            let keyword_match = triggers.keywords.iter()
                .any(|kw| prompt.contains(&kw.to_lowercase()));

            if keyword_match {
                matched_skills.push(MatchedSkill {
                    name: skill_name.clone(),
                    match_type: "keyword".to_string(),
                    priority: config.priority.clone(),
                });
                continue;
            }

            // Intent pattern matching
            let intent_match = triggers.intent_patterns.iter()
                .filter_map(|pattern| Regex::new(pattern).ok())
                .any(|regex| regex.is_match(&prompt));

            if intent_match {
                matched_skills.push(MatchedSkill {
                    name: skill_name.clone(),
                    match_type: "intent".to_string(),
                    priority: config.priority.clone(),
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
        let critical: Vec<_> = matched_skills.iter()
            .filter(|s| s.priority == "critical")
            .collect();
        let high: Vec<_> = matched_skills.iter()
            .filter(|s| s.priority == "high")
            .collect();
        let medium: Vec<_> = matched_skills.iter()
            .filter(|s| s.priority == "medium")
            .collect();
        let low: Vec<_> = matched_skills.iter()
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

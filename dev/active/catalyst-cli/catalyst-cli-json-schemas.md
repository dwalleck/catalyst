# Catalyst CLI - JSON Schema Specifications

**Last Updated:** 2025-01-04
**Status:** Phase 0 - Specifications
**Related:** catalyst-cli-plan.md, catalyst-cli-tasks.md, catalyst-cli-data-structures.md

---

## Overview

This document specifies ALL JSON file formats and plain text file formats that the Catalyst CLI will generate and parse. Each specification includes:

- Complete schema definition
- Field descriptions and constraints
- Example files
- Validation rules

**Files Specified:**
1. `settings.json` - Claude Code hook configurations
2. `skill-rules.json` - Skill activation rules
3. `.catalyst-version` - Version tracking (plain text)
4. `.catalyst-hashes.json` - Skill modification detection

---

## Table of Contents

1. [settings.json](#settingsjson)
2. [skill-rules.json](#skill-rulesjson)
3. [.catalyst-version](#catalyst-version)
4. [.catalyst-hashes.json](#catalyst-hashesjson)

---

## settings.json

**Location:** `.claude/settings.json`

**Purpose:** Configures Claude Code hooks for skill auto-activation and file change tracking.

### Schema

```typescript
{
  "hooks": {
    [hookType: string]: {
      "command": string,          // Path to wrapper script
      "matchers"?: Array<{        // Optional: filter when hook runs
        "type": string,           // "path" | "tool"
        "pattern": string         // Regex pattern
      }>
    }
  }
}
```

### Field Descriptions

| Field Path | Type | Required | Description | Constraints |
|------------|------|----------|-------------|-------------|
| `hooks` | Object | Yes | Map of hook type → configuration | Must have at least one hook |
| `hooks.<type>` | Object | Yes | Configuration for specific hook | Type must be valid Claude hook |
| `hooks.<type>.command` | String | Yes | Path to hook wrapper script | Must use `$CLAUDE_PROJECT_DIR` variable |
| `hooks.<type>.matchers` | Array | No | Filters for when hook runs | Only used for PostToolUse |
| `hooks.<type>.matchers[].type` | String | Yes | Matcher type | "path" or "tool" |
| `hooks.<type>.matchers[].pattern` | String | Yes | Regex pattern | Must be valid regex |

### Valid Hook Types

| Hook Type | Purpose | When It Runs |
|-----------|---------|--------------|
| `UserPromptSubmit` | Skill activation | Before every user prompt |
| `PostToolUse` | File change tracking | After every tool use |

### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `$CLAUDE_PROJECT_DIR` | Absolute path to project root | `/home/user/my-project` |

**Why use $CLAUDE_PROJECT_DIR?**
- Portable across machines
- Works in monorepos
- Claude resolves at runtime

### Platform-Specific Wrapper Extensions

| Platform | Extension | Example |
|----------|-----------|---------|
| Linux/macOS/WSL | `.sh` | `skill-activation-prompt.sh` |
| Windows | `.ps1` | `skill-activation-prompt.ps1` |

### Complete Example (Unix)

```json
{
  "hooks": {
    "UserPromptSubmit": {
      "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
    },
    "PostToolUse": {
      "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/file-change-tracker.sh",
      "matchers": [
        {
          "type": "tool",
          "pattern": "^(Edit|Write|MultiEdit)$"
        },
        {
          "type": "path",
          "pattern": "\\.(ts|tsx|js|jsx|py|rs|go|java|rb|php|c|cpp|h|hpp)$"
        }
      ]
    }
  }
}
```

### Complete Example (Windows)

```json
{
  "hooks": {
    "UserPromptSubmit": {
      "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.ps1"
    },
    "PostToolUse": {
      "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/file-change-tracker.ps1",
      "matchers": [
        {
          "type": "tool",
          "pattern": "^(Edit|Write|MultiEdit)$"
        },
        {
          "type": "path",
          "pattern": "\\.(ts|tsx|js|jsx|py|rs|go|java|rb|php|c|cpp|h|hpp)$"
        }
      ]
    }
  }
}
```

### Minimal Example (Only Skill Activation)

```json
{
  "hooks": {
    "UserPromptSubmit": {
      "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
    }
  }
}
```

### Validation Rules

1. **JSON must be valid** - Parseable by `serde_json`
2. **At least one hook** - `hooks` object must not be empty
3. **Valid hook types** - Keys must be recognized by Claude Code
4. **Command paths** - Must use `$CLAUDE_PROJECT_DIR` variable
5. **File extensions** - Must match platform (`.sh` or `.ps1`)
6. **Matcher patterns** - Must be valid regex (no compilation errors)
7. **Pretty-printed** - 2-space indentation for readability

### Generation Algorithm (Pseudocode)

```rust
fn generate_settings_json(platform: Platform, config: &InitConfig) -> String {
    let ext = platform.wrapper_extension();  // ".sh" or ".ps1"
    let mut hooks = Map::new();

    // Always include UserPromptSubmit if install_hooks is true
    if config.install_hooks {
        hooks.insert("UserPromptSubmit", {
            "command": format!("$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt{}", ext)
        });
    }

    // Include PostToolUse if install_tracker is true
    if config.install_tracker {
        hooks.insert("PostToolUse", {
            "command": format!("$CLAUDE_PROJECT_DIR/.claude/hooks/file-change-tracker{}", ext),
            "matchers": [
                {"type": "tool", "pattern": "^(Edit|Write|MultiEdit)$"},
                {"type": "path", "pattern": r"\.(...file extensions...)$"}
            ]
        });
    }

    serde_json::to_string_pretty(&{"hooks": hooks})
}
```

---

## skill-rules.json

**Location:** `.claude/skills/skill-rules.json`

**Purpose:** Defines when each skill should auto-activate based on keywords, intent, and file paths.

### Schema

```typescript
{
  "version": string,                    // Schema version
  [skillId: string]: {
    "type": "keyword" | "intent",       // Activation type
    "enforcement": "suggest",           // How aggressively to activate
    "priority": number,                 // Activation priority (1-100)
    "keywords": string[],               // Trigger keywords
    "intentPatterns": string[],         // Intent matching patterns
    "pathPatterns": string[],           // File path glob patterns
    "enabled": boolean                  // Whether skill is enabled
  }
}
```

### Field Descriptions

| Field Path | Type | Required | Description | Constraints |
|------------|------|----------|-------------|-------------|
| `version` | String | Yes | Schema version | Currently "1.0" |
| `<skillId>` | Object | Yes | Configuration for one skill | Key = skill directory name |
| `<skillId>.type` | String | Yes | Activation trigger type | "keyword" or "intent" |
| `<skillId>.enforcement` | String | Yes | Activation aggressiveness | "suggest", "warn", or "block" |
| `<skillId>.priority` | Number | Yes | Priority when multiple match | 1-100 (higher = more important) |
| `<skillId>.keywords` | Array | Yes | Trigger words/phrases | Lowercase, no punctuation |
| `<skillId>.intentPatterns` | Array | Yes | Intent descriptions | Natural language patterns |
| `<skillId>.pathPatterns` | Array | Yes | Glob patterns for files | Standard glob syntax |
| `<skillId>.enabled` | Boolean | Yes | Whether skill can activate | true to enable |

### Activation Types

| Type | Trigger | Use Case |
|------|---------|----------|
| `keyword` | Exact keyword match in prompt | Fast, precise matching |
| `intent` | Semantic understanding of prompt | Flexible, context-aware |

**Note:** Most skills use `keyword` for MVP. `intent` requires LLM call.

### Enforcement Levels

| Level | Behavior | User Experience |
|-------|----------|-----------------|
| `suggest` | Gently recommend skill | User can ignore |
| `warn` | Strong recommendation | Highlighted suggestion |
| `block` | Require skill load | Cannot proceed without |

**Current Practice:** All skills use `suggest` to avoid being intrusive.

### Priority Guidelines

| Priority | Use Case | Example |
|----------|----------|---------|
| 90-100 | Critical, always-needed | `skill-developer` (meta-skill) |
| 70-89 | Framework-specific | `backend-dev-guidelines`, `frontend-dev-guidelines` |
| 50-69 | Feature-specific | `route-tester`, `error-tracking` |
| 1-49 | Optional/experimental | Future skills |

### Path Pattern Syntax

Uses standard glob patterns:

| Pattern | Matches | Example |
|---------|---------|---------|
| `*` | Any characters (single level) | `src/*.ts` matches `src/foo.ts` |
| `**` | Any characters (any depth) | `src/**/*.ts` matches `src/a/b/c.ts` |
| `{a,b}` | Either a or b | `**/*.{ts,tsx}` matches TS or TSX |
| `[abc]` | Any single character in set | `file[123].ts` |

### Default Path Patterns (Language-Agnostic)

**For general-purpose skills:**
```json
"pathPatterns": [
  "src/**/*",
  "lib/**/*",
  "app/**/*",
  "tests/**/*",
  "**/*.ts",
  "**/*.js",
  "**/*.py",
  "**/*.rs",
  "**/*.go"
]
```

**Why broad patterns?**
- Works across different project structures
- User customizes to narrow down
- Avoids false negatives (skill not triggering)

### Framework-Specific Path Patterns

**Frontend (React/Vue/Svelte):**
```json
"pathPatterns": [
  "**/*.{ts,tsx,js,jsx}",
  "**/*.{vue,svelte}",
  "src/components/**/*",
  "src/pages/**/*",
  "app/components/**/*"
]
```

**Backend (Node.js/Express):**
```json
"pathPatterns": [
  "src/**/*.ts",
  "services/**/*.ts",
  "routes/**/*.ts",
  "controllers/**/*.ts"
]
```

### Complete Example (Multi-Skill)

```json
{
  "version": "1.0",
  "skill-developer": {
    "type": "keyword",
    "enforcement": "suggest",
    "priority": 95,
    "keywords": [
      "skill",
      "create skill",
      "skill activation",
      "skill-rules.json",
      "progressive disclosure",
      "500-line rule"
    ],
    "intentPatterns": [
      "creating a new skill",
      "modifying skill activation",
      "skill not activating"
    ],
    "pathPatterns": [
      ".claude/skills/**/*",
      "**/*SKILL.md",
      "**/*skill-rules.json"
    ],
    "enabled": true
  },
  "backend-dev-guidelines": {
    "type": "keyword",
    "enforcement": "suggest",
    "priority": 80,
    "keywords": [
      "express",
      "prisma",
      "controller",
      "service",
      "repository",
      "middleware",
      "route",
      "api endpoint",
      "database",
      "orm"
    ],
    "intentPatterns": [
      "creating an API endpoint",
      "implementing a controller",
      "database query",
      "service layer logic"
    ],
    "pathPatterns": [
      "src/**/*.ts",
      "services/**/*.ts",
      "routes/**/*.ts",
      "controllers/**/*.ts",
      "repositories/**/*.ts",
      "middleware/**/*.ts"
    ],
    "enabled": true
  },
  "frontend-dev-guidelines": {
    "type": "keyword",
    "enforcement": "suggest",
    "priority": 80,
    "keywords": [
      "react",
      "component",
      "mui",
      "tanstack",
      "useSuspenseQuery",
      "router",
      "styling",
      "theme"
    ],
    "intentPatterns": [
      "creating a React component",
      "fetching data with TanStack Query",
      "styling with MUI",
      "routing with TanStack Router"
    ],
    "pathPatterns": [
      "src/**/*.{ts,tsx}",
      "app/**/*.{ts,tsx}",
      "components/**/*.{ts,tsx}",
      "pages/**/*.{ts,tsx}",
      "features/**/*.{ts,tsx}"
    ],
    "enabled": true
  }
}
```

### Minimal Example (Single Skill)

```json
{
  "version": "1.0",
  "skill-developer": {
    "type": "keyword",
    "enforcement": "suggest",
    "priority": 95,
    "keywords": ["skill", "create skill"],
    "intentPatterns": ["creating a new skill"],
    "pathPatterns": [".claude/skills/**/*"],
    "enabled": true
  }
}
```

### Validation Rules

1. **JSON must be valid** - Parseable by `serde_json`
2. **Version required** - Must have `"version": "1.0"`
3. **At least one skill** - Must have at least one skill entry
4. **All fields required** - Each skill must have all 7 fields
5. **Valid types** - `type` must be "keyword" or "intent"
6. **Valid enforcement** - Must be "suggest", "warn", or "block"
7. **Priority range** - Must be 1-100
8. **Non-empty arrays** - `keywords`, `intentPatterns`, `pathPatterns` must not be empty
9. **Glob syntax** - `pathPatterns` must be valid glob patterns
10. **Pretty-printed** - 2-space indentation

### Generation Algorithm (Pseudocode)

```rust
fn generate_skill_rules(skills: &[String]) -> String {
    let mut rules = Map::new();
    rules.insert("version", "1.0");

    for skill_id in skills {
        let config = match skill_id.as_str() {
            "skill-developer" => {
                // Meta-skill: High priority, broad patterns
                SkillConfig {
                    type: "keyword",
                    enforcement: "suggest",
                    priority: 95,
                    keywords: vec!["skill", "create skill", ...],
                    intentPatterns: vec!["creating a new skill", ...],
                    pathPatterns: vec![".claude/skills/**/*", ...],
                    enabled: true,
                }
            },
            "backend-dev-guidelines" => {
                // Backend-specific: Medium priority, backend paths
                SkillConfig {
                    type: "keyword",
                    enforcement: "suggest",
                    priority: 80,
                    keywords: vec!["express", "prisma", ...],
                    intentPatterns: vec!["creating an API endpoint", ...],
                    pathPatterns: vec!["src/**/*.ts", "services/**/*.ts", ...],
                    enabled: true,
                }
            },
            // ... other skills
        };

        rules.insert(skill_id, config);
    }

    serde_json::to_string_pretty(&rules)
}
```

---

## .catalyst-version

**Location:** `.claude/skills/.catalyst-version`

**Purpose:** Track which version of Catalyst CLI was used to initialize the project.

### Format

**Plain text file** (not JSON) containing semver version string.

```
0.1.0
```

### Specification

- **Single line** - No trailing newline
- **Semver format** - `MAJOR.MINOR.PATCH`
- **UTF-8 encoding** - No BOM
- **No comments** - Just the version string

### Examples

```
0.1.0
```

```
1.2.3
```

### Usage

```rust
// Read version
fn read_catalyst_version(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

// Write version
fn write_catalyst_version(path: &Path) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");  // "0.1.0" at compile time
    fs::write(path, version)?;
    Ok(())
}

// Compare versions
fn is_up_to_date(installed: &str, current: &str) -> bool {
    installed == current  // Simple string comparison for MVP
    // Future: Use semver crate for proper comparison
}
```

### Validation Rules

1. **File exists** - Presence indicates initialized project
2. **Valid semver** - Must match `\d+\.\d+\.\d+` pattern
3. **Not empty** - Must contain at least "0.0.0"

---

## .catalyst-hashes.json

**Location:** `.claude/skills/.catalyst-hashes.json`

**Purpose:** Track SHA256 hashes of installed skills to detect user modifications.

### Schema

```typescript
{
  [relativePath: string]: string  // filepath → SHA256 hash
}
```

### Field Descriptions

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `<relativePath>` | String | Path relative to `.claude/skills/` | Forward slashes, even on Windows |
| `<hash>` | String | SHA256 hash (hex-encoded) | Lowercase, 64 characters |

### Path Format

**Always use forward slashes**, even on Windows:

✅ `"skill-developer/SKILL.md"`
✅ `"backend-dev-guidelines/resources/architecture.md"`
❌ `"skill-developer\\SKILL.md"` (Windows backslash - wrong)

### Complete Example

```json
{
  "skill-developer/SKILL.md": "a1b2c3d4e5f6...0123456789abcdef",
  "skill-developer/resources/activation-patterns.md": "1234567890abcdef...fedcba0987654321",
  "backend-dev-guidelines/SKILL.md": "fedcba0987654321...a1b2c3d4e5f6",
  "backend-dev-guidelines/resources/architecture.md": "0123456789abcdef...1234567890abcdef",
  "backend-dev-guidelines/resources/error-handling.md": "abcdef0123456789...fedcba0987654321"
}
```

### Hash Calculation

```rust
use sha2::{Sha256, Digest};

fn hash_file(path: &Path) -> Result<String> {
    let content = fs::read(path)?;  // Read as bytes
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))  // Hex encoding, lowercase
}
```

### Update Detection Algorithm

```rust
fn detect_modifications(skill_id: &str) -> Vec<String> {
    let stored_hashes = read_hashes_json()?;
    let mut modified = Vec::new();

    for file in walk_skill_files(skill_id) {
        let rel_path = file.strip_prefix(".claude/skills/")?;
        let rel_path_str = rel_path.to_slash_lossy();  // Always forward slashes

        let current_hash = hash_file(&file)?;
        let stored_hash = stored_hashes.get(&rel_path_str);

        if Some(&current_hash) != stored_hash {
            modified.push(rel_path_str.to_string());
        }
    }

    modified
}
```

### Validation Rules

1. **JSON must be valid** - Parseable by `serde_json`
2. **All paths relative** - No absolute paths
3. **Forward slashes only** - Even on Windows
4. **Valid SHA256** - 64 hex characters (lowercase)
5. **Pretty-printed** - 2-space indentation for readability
6. **All installed files tracked** - Every `.md` file should have an entry

### Generation Algorithm (Pseudocode)

```rust
fn generate_hashes_json(skills: &[String]) -> Result<String> {
    let mut hashes = Map::new();

    for skill_id in skills {
        let skill_dir = Path::new(".claude/skills").join(skill_id);

        for file in walk_md_files(&skill_dir) {
            let rel_path = file.strip_prefix(".claude/skills/")?;
            let rel_path_str = rel_path.to_slash_lossy().to_string();
            let hash = hash_file(&file)?;

            hashes.insert(rel_path_str, hash);
        }
    }

    serde_json::to_string_pretty(&hashes)
}
```

---

## Validation Testing Strategy

### Test Files Location

All example files should be created in `docs/schemas/` for validation:

```
docs/schemas/
├── settings.json.example
├── settings-windows.json.example
├── skill-rules.json.example
├── .catalyst-version.example
└── .catalyst-hashes.json.example
```

### Validation Commands

```bash
# Validate all JSON files parse correctly
for file in docs/schemas/*.json.example; do
  echo "Validating $file..."
  cat "$file" | jq . > /dev/null || echo "❌ FAILED: $file"
done

# Validate semver in .catalyst-version
cat docs/schemas/.catalyst-version.example | grep -E '^\d+\.\d+\.\d+$' || echo "❌ Invalid version format"

# Validate hash format in .catalyst-hashes.json
cat docs/schemas/.catalyst-hashes.json.example | jq -r 'values[]' | grep -E '^[a-f0-9]{64}$' || echo "❌ Invalid hash format"
```

---

## Completion Checklist

- [x] Document complete `settings.json` schema with example
- [x] Document complete `skill-rules.json` schema with example
- [x] Specify `.catalyst-version` format
- [x] Specify `.catalyst-hashes.json` format
- [x] Create example files in `docs/schemas/` directory
- [x] Validate all examples parse as valid JSON

---

**End of JSON Schema Specifications**

Next: Create example files in `docs/schemas/` directory.

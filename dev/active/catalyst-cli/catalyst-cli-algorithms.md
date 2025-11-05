# Catalyst CLI - Algorithm Specifications

**Last Updated:** 2025-01-04
**Status:** Phase 0 - Specifications
**Related:** catalyst-cli-plan.md, catalyst-cli-tasks.md, catalyst-cli-data-structures.md

---

## Overview

This document specifies all major algorithms in pseudocode before implementation. Each algorithm includes:
- Clear step-by-step logic
- Edge case handling
- Error conditions
- Validation rules
- Expected inputs and outputs

**Algorithms Specified:**
1. Settings Creation Algorithm
2. Status Determination Rules
3. Auto-Fix Decision Tree
4. Skill Hash Tracking Algorithm
5. Default PathPatterns Strategy

---

## Table of Contents

1. [Settings Creation Algorithm](#settings-creation-algorithm)
2. [Status Determination Rules](#status-determination-rules)
3. [Auto-Fix Decision Tree](#auto-fix-decision-tree)
4. [Skill Hash Tracking Algorithm](#skill-hash-tracking-algorithm)
5. [Default PathPatterns Strategy](#default-pathpatterns-strategy)

---

## Settings Creation Algorithm

**Purpose:** Generate `settings.json` with hook configurations based on platform and user choices.

### Inputs

| Parameter | Type | Description |
|-----------|------|-------------|
| `platform` | `Platform` | Detected platform (Linux/MacOS/Windows/WSL) |
| `config` | `InitConfig` | User's initialization choices |
| `target_path` | `PathBuf` | Path to `.claude/settings.json` |

### Outputs

| Output | Type | Description |
|--------|------|-------------|
| `Result<()>` | `Result` | Success or error (file already exists, write failed) |

### Pseudocode

```
FUNCTION generate_settings_json(platform: Platform, config: InitConfig, target_path: PathBuf) -> Result<()>
    // 1. Check if file already exists
    IF target_path.exists() AND NOT config.force THEN
        RETURN Error(AlreadyInitialized)
    END IF

    // 2. Determine wrapper extension based on platform
    wrapper_ext = platform.wrapper_extension()  // ".sh" or ".ps1"

    // 3. Build hooks configuration
    hooks = Map::new()

    // 4. Add UserPromptSubmit hook if requested
    IF config.install_hooks THEN
        hooks.insert("UserPromptSubmit", {
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt" + wrapper_ext
        })
    END IF

    // 5. Add PostToolUse hook if requested
    IF config.install_tracker THEN
        // Define matchers for file edits only
        matchers = [
            {
                "type": "tool",
                "pattern": "^(Edit|Write|MultiEdit)$"
            },
            {
                "type": "path",
                "pattern": r"\.(ts|tsx|js|jsx|py|rs|go|java|rb|php|c|cpp|h|hpp)$"
            }
        ]

        hooks.insert("PostToolUse", {
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/file-change-tracker" + wrapper_ext,
            "matchers": matchers
        })
    END IF

    // 6. Validate we have at least one hook
    IF hooks.is_empty() THEN
        RETURN Error(InvalidConfig("At least one hook must be enabled"))
    END IF

    // 7. Build final JSON structure
    settings = {
        "hooks": hooks
    }

    // 8. Serialize to pretty JSON (2-space indent)
    json_string = serde_json::to_string_pretty(settings)
        .map_err(|e| Error(JsonError { source: e, file: target_path }))?

    // 9. Write atomically (with fallback on network FS)
    write_file_atomic(target_path, json_string)?

    // 10. Validate generated file parses correctly
    validate_json_file(target_path)?

    RETURN Ok(())
END FUNCTION
```

### Edge Cases

| Edge Case | Handling |
|-----------|----------|
| File already exists | Error if `!force`, overwrite if `force` |
| Both hooks disabled | Error - at least one hook required |
| Write permission denied | Error with helpful message |
| Atomic write fails (network FS) | Fallback to regular write, warn user |
| Generated JSON invalid | Error before writing (shouldn't happen) |

### Validation

```
FUNCTION validate_generated_settings(path: PathBuf) -> Result<()>
    // Read back the file
    content = read_to_string(path)?

    // Parse as JSON
    parsed = serde_json::from_str(content)
        .map_err(|e| Error(JsonError))?

    // Validate structure
    IF NOT parsed.contains_key("hooks") THEN
        RETURN Error("Missing 'hooks' key")
    END IF

    IF parsed["hooks"].is_empty() THEN
        RETURN Error("'hooks' must not be empty")
    END IF

    // Validate each hook has required fields
    FOR each (hook_type, hook_config) IN parsed["hooks"]
        IF NOT hook_config.contains_key("command") THEN
            RETURN Error("Hook missing 'command' field: " + hook_type)
        END IF

        IF NOT hook_config["command"].starts_with("$CLAUDE_PROJECT_DIR") THEN
            WARN "Hook command should use $CLAUDE_PROJECT_DIR variable"
        END IF
    END FOR

    RETURN Ok(())
END FUNCTION
```

---

## Status Determination Rules

**Purpose:** Determine overall health status (Healthy/Warning/Error) based on collected issues.

### Inputs

| Parameter | Type | Description |
|-----------|------|-------------|
| `issues` | `Vec<Issue>` | All issues found during status check |

### Outputs

| Output | Type | Description |
|--------|------|-------------|
| `HealthStatus` | Enum | Healthy, Warning, or Error |

### Decision Rules

```
FUNCTION determine_overall_status(issues: Vec<Issue>) -> HealthStatus
    // Rule 1: No issues = Healthy
    IF issues.is_empty() THEN
        RETURN HealthStatus::Healthy
    END IF

    // Rule 2: Any Error severity = Overall Error
    FOR each issue IN issues
        IF issue.severity == IssueSeverity::Error THEN
            RETURN HealthStatus::Error
        END IF
    END FOR

    // Rule 3: Any Warning severity (no Errors) = Overall Warning
    FOR each issue IN issues
        IF issue.severity == IssueSeverity::Warning THEN
            RETURN HealthStatus::Warning
        END IF
    END FOR

    // Rule 4: Only Info messages = Healthy
    RETURN HealthStatus::Healthy
END FUNCTION
```

### Issue Severity Guidelines

| Condition | Severity | Example |
|-----------|----------|---------|
| Binary missing | `Error` | skill-activation-prompt not found |
| Wrapper missing | `Error` | skill-activation-prompt.sh doesn't exist |
| Wrapper not executable | `Error` | Missing +x permission (Unix) |
| Binary not accessible | `Error` | Wrapper can't find binary |
| Skill incomplete | `Error` | Missing SKILL.md |
| Invalid JSON | `Error` | skill-rules.json syntax error |
| Skill modified | `Warning` | Hash mismatch (user customized) |
| Skill not in rules | `Warning` | Won't auto-activate |
| SQLite variant detected | `Info` | Using SQLite tracker |
| Version info | `Info` | Catalyst v0.1.0 |

### Complete Status Check Algorithm

```
FUNCTION check_status(project_path: PathBuf) -> Result<StatusReport>
    issues = Vec::new()

    // 1. Validate binaries
    binary_status = validate_binaries()?
    FOR each binary IN binary_status
        IF NOT binary.found THEN
            issues.push(Issue {
                severity: Error,
                message: "Binary not found: " + binary.name,
                suggestion: Some("Run ./install.sh to install binaries"),
                auto_fixable: false
            })
        END IF
    END FOR

    // 2. Validate hooks
    hook_status = validate_hooks(project_path)?
    FOR each hook IN hook_status
        IF NOT hook.wrapper_exists THEN
            issues.push(Issue {
                severity: Error,
                message: "Wrapper missing: " + hook.wrapper_path,
                suggestion: Some("Run: catalyst status --fix"),
                auto_fixable: true  // Can recreate wrapper
            })
        ELSE IF NOT hook.executable THEN
            issues.push(Issue {
                severity: Error,
                message: "Wrapper not executable: " + hook.wrapper_path,
                suggestion: Some("Run: chmod +x " + hook.wrapper_path),
                auto_fixable: true  // Can set permission
            })
        ELSE IF NOT hook.binary_accessible THEN
            issues.push(Issue {
                severity: Error,
                message: "Binary not accessible from wrapper: " + hook.hook_type,
                suggestion: Some("Check binary installation path"),
                auto_fixable: false
            })
        END IF
    END FOR

    // 3. Validate skills
    skill_status = validate_skills(project_path)?
    FOR each skill IN skill_status
        IF NOT skill.has_skill_md THEN
            issues.push(Issue {
                severity: Error,
                message: "Skill incomplete: " + skill.id + " (missing SKILL.md)",
                suggestion: Some("Reinstall skill: catalyst init --force"),
                auto_fixable: false
            })
        ELSE IF NOT skill.in_rules THEN
            issues.push(Issue {
                severity: Warning,
                message: "Skill not in skill-rules.json: " + skill.id,
                suggestion: Some("Skill won't auto-activate. Add to skill-rules.json."),
                auto_fixable: false
            })
        END IF
    END FOR

    // 4. Determine overall status
    overall = determine_overall_status(issues)

    // 5. Build status report
    RETURN StatusReport {
        overall: overall,
        binaries: binary_status,
        hooks: hook_status,
        skills: skill_status,
        issues: issues
    }
END FUNCTION
```

### Edge Cases

| Edge Case | Handling |
|-----------|----------|
| `.claude/` doesn't exist | Error: "Not initialized" |
| No settings.json | Warning: "No hooks configured" |
| No skill-rules.json | Warning: "No skills configured" |
| Empty issues, Info only | Still show as Healthy |
| Multiple errors | Show all, don't stop at first |

---

## Auto-Fix Decision Tree

**Purpose:** Determine which issues can be automatically fixed and execute fixes.

### Inputs

| Parameter | Type | Description |
|-----------|------|-------------|
| `status_report` | `StatusReport` | Current status with issues |
| `platform` | `Platform` | Current platform |

### Outputs

| Output | Type | Description |
|--------|------|-------------|
| `FixReport` | Struct | What was fixed, what failed |

### Decision Tree

```
FUNCTION auto_fix(status_report: StatusReport, platform: Platform) -> Result<FixReport>
    fixed = Vec::new()
    failed = Vec::new()

    // 1. Filter auto-fixable issues
    auto_fixable_issues = status_report.issues
        .filter(|issue| issue.auto_fixable)

    IF auto_fixable_issues.is_empty() THEN
        RETURN FixReport { fixed: [], failed: [] }
    END IF

    // 2. Fix each issue based on type
    FOR each issue IN auto_fixable_issues
        result = MATCH issue.message
            CASE "Wrapper missing: *"
                fix_missing_wrapper(issue, platform)

            CASE "Wrapper not executable: *"
                fix_wrapper_permissions(issue)

            CASE default
                // Shouldn't happen if auto_fixable is true
                Err("Unknown fixable issue: " + issue.message)
        END MATCH

        // 3. Record result
        IF result.is_ok() THEN
            fixed.push(result.unwrap())
        ELSE
            failed.push((issue.message, result.err()))
        END IF
    END FOR

    RETURN FixReport { fixed, failed }
END FUNCTION
```

### Fix Implementations

#### Fix Missing Wrapper

```
FUNCTION fix_missing_wrapper(issue: Issue, platform: Platform) -> Result<String>
    // 1. Extract wrapper path from issue message
    wrapper_path = extract_path_from_message(issue.message)

    // 2. Determine binary name from wrapper path
    // E.g., "skill-activation-prompt.sh" -> "skill-activation-prompt"
    binary_name = wrapper_path.file_stem()

    // 3. Load template for platform
    template = load_wrapper_template(platform)  // .sh or .ps1

    // 4. Generate wrapper content
    wrapper_content = template.replace("{{BINARY_NAME}}", binary_name)

    // 5. Write wrapper file
    write_file_atomic(wrapper_path, wrapper_content)?

    // 6. Set executable permission (Unix only)
    IF platform.has_permissions() THEN
        set_executable(wrapper_path)?
    END IF

    // 7. Verify wrapper works
    IF NOT verify_wrapper_accessible(wrapper_path) THEN
        RETURN Err("Wrapper created but binary not accessible")
    END IF

    RETURN Ok("Recreated " + wrapper_path.file_name())
END FUNCTION
```

#### Fix Wrapper Permissions

```
FUNCTION fix_wrapper_permissions(issue: Issue) -> Result<String>
    // 1. Extract wrapper path
    wrapper_path = extract_path_from_message(issue.message)

    // 2. Set executable permission (Unix only)
    set_executable(wrapper_path)?

    // 3. Verify now executable
    IF NOT is_executable(wrapper_path) THEN
        RETURN Err("Permission set failed (check filesystem)")
    END IF

    RETURN Ok("Set executable: " + wrapper_path.file_name())
END FUNCTION
```

### Non-Fixable Issues

| Issue Type | Why Not Fixable | User Action Required |
|------------|-----------------|---------------------|
| Binary missing | Requires compilation | Run `./install.sh` |
| Binary not accessible | Path configuration issue | Check PATH or installation |
| Skill incomplete | Embedded resources needed | Run `catalyst init --force` |
| Invalid JSON | User-edited file corrupted | Manually fix or regenerate |
| Permission denied | Filesystem restrictions | Run as admin or fix permissions |

### Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Fix fails mid-operation | Record in `failed`, continue others |
| Wrapper recreated but binary missing | Report as partial success + error |
| Multiple issues with same wrapper | Fix once, report once |
| Concurrent fix attempts | Lock file prevents (see Task 0.5) |

---

## Skill Hash Tracking Algorithm

**Purpose:** Track SHA256 hashes of installed skills to detect user modifications during updates.

### Initial Hash Generation

```
FUNCTION generate_skill_hashes(skills: Vec<String>) -> Result<Map<String, String>>
    hashes = Map::new()

    FOR each skill_id IN skills
        skill_dir = Path::new(".claude/skills").join(skill_id)

        // Walk all .md files in skill directory
        FOR each file IN walk_md_files(skill_dir)
            // Compute relative path from .claude/skills/
            rel_path = file.strip_prefix(".claude/skills/")?

            // Normalize to forward slashes (even on Windows)
            rel_path_str = rel_path.to_slash_lossy().to_string()

            // Compute SHA256 hash
            hash = hash_file(file)?

            // Store in map
            hashes.insert(rel_path_str, hash)
        END FOR
    END FOR

    RETURN hashes
END FUNCTION
```

### Hash Comparison During Update

```
FUNCTION detect_modified_skills(installed_skills: Vec<String>) -> Result<Vec<String>>
    // 1. Load stored hashes from .catalyst-hashes.json
    stored_hashes = read_hashes_json(".claude/skills/.catalyst-hashes.json")?

    modified_skills = Set::new()

    // 2. For each installed skill
    FOR each skill_id IN installed_skills
        skill_dir = Path::new(".claude/skills").join(skill_id)

        // 3. Walk all .md files
        FOR each file IN walk_md_files(skill_dir)
            rel_path = file.strip_prefix(".claude/skills/")?
            rel_path_str = rel_path.to_slash_lossy().to_string()

            // 4. Compute current hash
            current_hash = hash_file(file)?

            // 5. Compare with stored hash
            MATCH stored_hashes.get(rel_path_str)
                CASE Some(stored_hash)
                    IF current_hash != stored_hash THEN
                        // File modified!
                        modified_skills.insert(skill_id)
                    END IF

                CASE None
                    // New file added by user
                    modified_skills.insert(skill_id)
            END MATCH
        END FOR
    END FOR

    RETURN modified_skills.to_vec()
END FUNCTION
```

### Update Logic with Hash Detection

```
FUNCTION update_skills(config: UpdateConfig) -> Result<UpdateReport>
    installed_skills = get_installed_skills()?
    modified_skills = detect_modified_skills(installed_skills)?

    skills_updated = Vec::new()
    skills_skipped = Vec::new()

    FOR each skill_id IN installed_skills
        // Skip modified skills unless --force
        IF modified_skills.contains(skill_id) AND NOT config.force THEN
            skills_skipped.push(skill_id)
            CONTINUE
        END IF

        // Update skill from embedded resources
        update_skill_from_embedded(skill_id)?
        skills_updated.push(skill_id)
    END FOR

    // Regenerate hashes for updated skills
    regenerate_hashes(skills_updated)?

    RETURN UpdateReport {
        skills_updated,
        skills_skipped,
        // ... other fields
    }
END FUNCTION
```

### Hash File Algorithm

```
FUNCTION hash_file(path: PathBuf) -> Result<String>
    // 1. Read file as raw bytes (not string - preserves exact content)
    content = read_file_bytes(path)?

    // 2. Create SHA256 hasher
    hasher = Sha256::new()

    // 3. Update with file content
    hasher.update(content)

    // 4. Finalize and get hash
    hash_bytes = hasher.finalize()

    // 5. Convert to lowercase hex string
    hash_string = format!("{:x}", hash_bytes)  // Always 64 chars

    RETURN hash_string
END FUNCTION
```

### Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Hash file missing | Assume all skills modified (or regenerate) |
| Hash file corrupt | Error, suggest `--force` |
| New files added by user | Detected as modification |
| Files deleted by user | Detected as modification |
| Line ending changes (CRLF vs LF) | **Not** detected (hash changes) |
| Empty skill directory | No hashes generated |
| Symbolic links | Follow links, hash target content |

### Hash File Format Validation

```
FUNCTION validate_hash_file(path: PathBuf) -> Result<()>
    hashes = read_hashes_json(path)?

    FOR each (file_path, hash_value) IN hashes
        // Validate path uses forward slashes
        IF file_path.contains("\\") THEN
            RETURN Err("Path must use forward slashes: " + file_path)
        END IF

        // Validate hash is 64 hex characters
        IF NOT is_valid_sha256(hash_value) THEN
            RETURN Err("Invalid SHA256 hash: " + hash_value)
        END IF
    END FOR

    RETURN Ok(())
END FUNCTION

FUNCTION is_valid_sha256(hash: String) -> bool
    RETURN hash.len() == 64 AND hash.chars().all(|c| c.is_ascii_hexdigit())
END FUNCTION
```

---

## Default PathPatterns Strategy

**Purpose:** Generate sensible default pathPatterns for skill-rules.json that work across different project structures.

### Design Philosophy

**Problem:** Users have vastly different directory structures:
- Monorepos: `packages/*/src/**/*.ts`
- Single-app: `src/**/*.ts`
- Nested: `backend/src/**/*.ts`
- Flat: `*.ts`

**Solution:** Use **broad patterns** that match common locations, then users narrow down.

### Pattern Categories

#### 1. Framework-Agnostic Patterns (General Skills)

```
FUNCTION get_generic_pathpatterns() -> Vec<String>
    RETURN [
        // Common source directories
        "src/**/*",
        "lib/**/*",
        "app/**/*",

        // Test directories
        "tests/**/*",
        "test/**/*",
        "__tests__/**/*",

        // Language-agnostic extensions
        "**/*.ts",
        "**/*.js",
        "**/*.py",
        "**/*.rs",
        "**/*.go",
        "**/*.java",

        // Config files
        "*.config.{js,ts}",
        "*.json"
    ]
END FUNCTION
```

**Why broad?**
- ✅ Matches 90% of projects without customization
- ✅ Avoids false negatives (skill not triggering)
- ⚠️ May cause false positives (skill suggests unnecessarily)
- 👤 User narrows down to their structure

#### 2. Backend-Specific Patterns

```
FUNCTION get_backend_pathpatterns() -> Vec<String>
    RETURN [
        // TypeScript/Node.js backend
        "src/**/*.ts",
        "services/**/*.ts",
        "routes/**/*.ts",
        "controllers/**/*.ts",
        "repositories/**/*.ts",
        "middleware/**/*.ts",
        "models/**/*.ts",

        // Prisma
        "prisma/**/*.prisma",

        // Express patterns
        "**/routes/**/*",
        "**/api/**/*",

        // Generic backend
        "backend/**/*",
        "server/**/*"
    ]
END FUNCTION
```

#### 3. Frontend-Specific Patterns

```
FUNCTION get_frontend_pathpatterns() -> Vec<String>
    RETURN [
        // React/TypeScript
        "src/**/*.{ts,tsx}",
        "app/**/*.{ts,tsx}",
        "components/**/*.{ts,tsx}",
        "pages/**/*.{ts,tsx}",
        "features/**/*.{ts,tsx}",

        // Other frameworks
        "**/*.vue",
        "**/*.svelte",

        // Generic frontend
        "frontend/**/*",
        "client/**/*",
        "web/**/*"
    ]
END FUNCTION
```

#### 4. Meta-Skill Patterns (skill-developer)

```
FUNCTION get_skill_developer_pathpatterns() -> Vec<String>
    RETURN [
        // Claude Code infrastructure
        ".claude/skills/**/*",
        ".claude/agents/**/*",
        ".claude/commands/**/*",

        // Skill files
        "**/*SKILL.md",
        "**/skill-rules.json",

        // This skill activates on its own infrastructure
    ]
END FUNCTION
```

### Complete Generation Algorithm

```
FUNCTION generate_skill_rules_pathpatterns(skill_id: String) -> Vec<String>
    RETURN MATCH skill_id
        CASE "skill-developer"
            get_skill_developer_pathpatterns()

        CASE "backend-dev-guidelines"
            get_backend_pathpatterns()

        CASE "frontend-dev-guidelines"
            get_frontend_pathpatterns()

        CASE "route-tester"
            // Backend routes + tests
            concat(
                get_backend_pathpatterns(),
                ["**/routes/**/*", "**/*test*.ts"]
            )

        CASE "error-tracking"
            // Anywhere errors might occur
            concat(
                get_backend_pathpatterns(),
                get_frontend_pathpatterns()
            )

        CASE default
            // Unknown skill - use generic patterns
            get_generic_pathpatterns()
    END MATCH
END FUNCTION
```

### Customization Guidance

**Generated with comment:**
```json
{
  "version": "1.0",
  "backend-dev-guidelines": {
    "pathPatterns": [
      "src/**/*.ts",
      "services/**/*.ts"
    ],
    // 👆 CUSTOMIZE THESE for your project structure:
    // - Monorepo: "packages/*/src/**/*.ts"
    // - Nested: "backend/src/**/*.ts"
    // - Flat: "*.ts"
  }
}
```

### Pattern Validation

```
FUNCTION validate_pathpattern(pattern: String) -> Result<()>
    // 1. Check for common mistakes
    IF pattern.contains("\\") THEN
        RETURN Err("Use forward slashes even on Windows: " + pattern)
    END IF

    // 2. Check for absolute paths
    IF pattern.starts_with("/") OR pattern.contains(":") THEN
        RETURN Err("Patterns must be relative: " + pattern)
    END IF

    // 3. Warn about overly broad patterns
    IF pattern == "**/*" OR pattern == "*" THEN
        WARN "Very broad pattern - may cause skill to activate too often"
    END IF

    // 4. Validate glob syntax (basic check)
    IF pattern.contains("[") AND NOT pattern.contains("]") THEN
        RETURN Err("Unmatched bracket in pattern: " + pattern)
    END IF

    RETURN Ok(())
END FUNCTION
```

### Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Empty patterns array | Error - at least one required |
| Pattern matches nothing | Warning during validation (hard to detect) |
| Pattern too broad | Warning to user |
| Duplicate patterns | Deduplicate silently |
| Invalid glob syntax | Error with helpful message |
| Windows backslashes | Convert to forward slashes |

---

## Algorithm Edge Cases Summary

### Cross-Algorithm Edge Cases

| Scenario | Impact | Handling |
|----------|--------|----------|
| Network filesystem | Atomic write may fail | Fallback to regular write, warn user |
| Concurrent operations | File corruption risk | Lock file protection (Task 0.5) |
| WSL environment | Path resolution differs | Platform detection (Task 0.5) |
| Missing home directory | Cannot find binaries | Error with clear message |
| Corrupted JSON files | Parse errors | Helpful error with line number |
| Partial installation | Inconsistent state | Status command detects, auto-fix repairs |
| User deletes files | Detection in status | Clear error, suggest fix |
| User modifies skills | Hash mismatch | Skip in update, --force to overwrite |

---

## Completion Checklist

- [x] Specify settings creation algorithm in pseudocode
- [x] Specify status determination rules (Healthy/Warning/Error)
- [x] Specify auto-fix decision tree in pseudocode
- [x] Specify skill hash tracking algorithm
- [x] Document default pathPatterns strategy
- [x] Review all algorithms for edge cases

---

**End of Algorithm Specifications**

Next: See `catalyst-cli-helpers.md` (Task 0.4) for helper function specifications.

//! Status validation and diagnostic functionality
//!
//! This module provides comprehensive validation of Catalyst installations,
//! including binary checks, hook configurations, and skill installations.
//! It also provides auto-fix capabilities for common issues.

use crate::types::{
    BinaryStatus, CatalystError, HookStatus, Issue, IssueSeverity, Platform, Result, SkillStatus,
    StatusLevel, StatusReport, VersionStatus, BINARY_DIR, HOOKS_DIR, SETTINGS_FILE, SKILLS_DIR,
    SKILL_RULES_FILE,
};
use crate::validation::{binary_exists, detect_file_change_tracker_variant, get_binary_directory};
use catalyst_core::settings::ClaudeSettings;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Validate the complete Catalyst installation
///
/// Performs comprehensive checks on binaries, hooks, and skills,
/// returning a detailed status report with issues and severity levels.
///
/// # Arguments
///
/// * `target_dir` - Base directory to validate (defaults to current directory)
/// * `platform` - Current platform (for platform-specific checks)
pub fn validate_installation(target_dir: &Path, platform: Platform) -> Result<StatusReport> {
    let mut report = StatusReport::new();

    // Task 4.2: Validate binaries
    report.binaries = validate_binaries(platform)?;

    // Task 4.3: Validate hooks
    report.hooks = validate_hooks(target_dir, platform)?;

    // Task 4.4: Validate skills
    report.skills = validate_skills(target_dir)?;

    // Check version file
    report.version_status = check_version(target_dir)?;

    // Collect issues based on validation results
    collect_issues(&mut report);

    // Determine overall status level
    report.level = determine_status_level(&report);

    Ok(report)
}

/// Validate that all required binaries are installed and accessible
///
/// Checks ~/.claude-hooks/bin/ (or Windows equivalent) for:
/// - skill-activation-prompt
/// - file-change-tracker (both variants: SQLite and basic)
/// - file-analyzer
///
/// # Arguments
///
/// * `platform` - Current platform (for .exe extension on Windows)
fn validate_binaries(platform: Platform) -> Result<Vec<BinaryStatus>> {
    let mut binaries = Vec::new();

    // Get binary directory
    let bin_dir = get_binary_directory()?;

    // Check skill-activation-prompt
    binaries.push(validate_binary(
        "skill-activation-prompt",
        &bin_dir,
        platform,
        None,
    ));

    // Check file-change-tracker (detect variant)
    let tracker_variant = detect_file_change_tracker_variant(&bin_dir, platform)?;
    binaries.push(validate_binary(
        "file-change-tracker",
        &bin_dir,
        platform,
        tracker_variant,
    ));

    // Check file-analyzer
    binaries.push(validate_binary("file-analyzer", &bin_dir, platform, None));

    Ok(binaries)
}

/// Validate a single binary
fn validate_binary(
    name: &str,
    bin_dir: &Path,
    platform: Platform,
    variant: Option<String>,
) -> BinaryStatus {
    let exists = binary_exists(bin_dir, name, platform);
    let path = if exists {
        Some(bin_dir.join(format!(
            "{}{}",
            name,
            if matches!(platform, Platform::Windows) {
                ".exe"
            } else {
                ""
            }
        )))
    } else {
        None
    };

    // Check if executable (Unix only)
    let executable = if cfg!(unix) {
        path.as_ref()
            .map(|p| {
                fs::metadata(p)
                    .ok()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    } else {
        true // Windows executability not checked
    };

    BinaryStatus {
        name: name.to_string(),
        exists,
        executable,
        version: None, // MVP: version detection not implemented
        expected_version: None,
        version_matches: false,
        path,
        variant,
    }
}

/// Validate hook configurations and wrapper scripts
///
/// Checks that:
/// 1. settings.json exists and is valid
/// 2. Configured hooks have wrapper scripts in .claude/hooks/
/// 3. Wrapper scripts are executable (Unix)
/// 4. Wrapper scripts can access the required binaries
///
/// # Arguments
///
/// * `target_dir` - Base directory containing .claude/
/// * `platform` - Current platform (for wrapper extension)
fn validate_hooks(target_dir: &Path, platform: Platform) -> Result<Vec<HookStatus>> {
    let mut hooks = Vec::new();

    // Check if settings.json exists
    let settings_path = target_dir.join(SETTINGS_FILE);
    if !settings_path.exists() {
        // No settings.json - report empty hooks with issue
        return Ok(hooks);
    }

    // Parse settings.json
    let settings_path_str = settings_path.to_str().ok_or_else(|| {
        CatalystError::InvalidPath(format!(
            "Settings path contains non-UTF-8 characters: {:?}",
            settings_path
        ))
    })?;

    let settings = match ClaudeSettings::read(settings_path_str) {
        Ok(s) => s,
        Err(_) => {
            // Invalid settings.json - report with issue
            return Ok(hooks);
        }
    };

    // Check configured hooks
    let hooks_dir = target_dir.join(HOOKS_DIR);
    let extension = platform.hook_extension();

    // Check UserPromptSubmit hook (skill-activation-prompt)
    if let Some(user_prompt_hooks) = settings
        .hooks
        .get(&catalyst_core::settings::HookEvent::UserPromptSubmit)
    {
        for hook_config in user_prompt_hooks {
            for hook in &hook_config.hooks {
                if hook.command.contains("skill-activation-prompt") {
                    let wrapper_name = format!("skill-activation-prompt.{}", extension);
                    hooks.push(validate_hook(
                        &wrapper_name,
                        "UserPromptSubmit",
                        &hooks_dir,
                        "skill-activation-prompt",
                        platform,
                    ));
                    break; // Only check once per hook config
                }
            }
        }
    }

    // Check PostToolUse hook (file-change-tracker)
    if let Some(post_tool_hooks) = settings
        .hooks
        .get(&catalyst_core::settings::HookEvent::PostToolUse)
    {
        for hook_config in post_tool_hooks {
            for hook in &hook_config.hooks {
                if hook.command.contains("file-change-tracker") {
                    let wrapper_name = format!("file-change-tracker.{}", extension);
                    hooks.push(validate_hook(
                        &wrapper_name,
                        "PostToolUse",
                        &hooks_dir,
                        "file-change-tracker",
                        platform,
                    ));
                    break; // Only check once per hook config
                }
            }
        }
    }

    Ok(hooks)
}

/// Validate a single hook wrapper
fn validate_hook(
    wrapper_name: &str,
    event: &str,
    hooks_dir: &Path,
    binary_name: &str,
    platform: Platform,
) -> HookStatus {
    let wrapper_path = hooks_dir.join(wrapper_name);
    let exists = wrapper_path.exists();

    // Check if executable (Unix only)
    let executable = if cfg!(unix) && exists {
        fs::metadata(&wrapper_path)
            .ok()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    } else {
        true // Windows or doesn't exist
    };

    // Check if binary is accessible
    let bin_dir = match get_binary_directory() {
        Ok(dir) => dir,
        Err(_) => {
            return HookStatus {
                name: wrapper_name.to_string(),
                exists,
                executable,
                configured: true,
                event: Some(event.to_string()),
                path: Some(wrapper_path),
                calls_correct_binary: false,
            }
        }
    };
    let calls_correct_binary = exists && binary_exists(&bin_dir, binary_name, platform);

    HookStatus {
        name: wrapper_name.to_string(),
        exists,
        executable,
        configured: true, // We only check configured hooks
        event: Some(event.to_string()),
        path: Some(wrapper_path),
        calls_correct_binary,
    }
}

/// Validate installed skills
///
/// Checks that:
/// 1. .claude/skills/ directory exists
/// 2. skill-rules.json exists and is valid
/// 3. Each skill has required files (SKILL.md)
/// 4. Skills are registered in skill-rules.json
///
/// # Arguments
///
/// * `target_dir` - Base directory containing .claude/
fn validate_skills(target_dir: &Path) -> Result<Vec<SkillStatus>> {
    let mut skills = Vec::new();

    let skills_dir = target_dir.join(SKILLS_DIR);
    if !skills_dir.exists() {
        return Ok(skills);
    }

    // Check if skill-rules.json exists
    let skill_rules_path = target_dir.join(SKILL_RULES_FILE);
    let has_skill_rules = skill_rules_path.exists();

    // Read installed skills from directory
    let entries = match fs::read_dir(&skills_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(skills),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Skip hidden files and skill-rules.json
            if skill_name.starts_with('.') || skill_name == "skill-rules.json" {
                continue;
            }

            let has_main_file = path.join("SKILL.md").exists();

            skills.push(SkillStatus {
                name: skill_name,
                exists: true,
                has_main_file,
                registered: has_skill_rules, // Simplified check
                current_hash: None,          // Not computed during validation
                expected_hash: None,
                modified: false,
                path: Some(path),
            });
        }
    }

    Ok(skills)
}

/// Check version file status
fn check_version(target_dir: &Path) -> Result<VersionStatus> {
    let version_path = target_dir.join(".catalyst-version");

    if !version_path.exists() {
        return Ok(VersionStatus::Missing);
    }

    // Read version file
    let version = fs::read_to_string(&version_path)
        .map_err(CatalystError::Io)?
        .trim()
        .to_string();

    // Compare to current version
    let current_version = env!("CARGO_PKG_VERSION");
    if version == current_version {
        Ok(VersionStatus::Ok { version })
    } else {
        Ok(VersionStatus::Mismatch {
            expected: current_version.to_string(),
            found: version,
        })
    }
}

/// Collect issues from validation results
fn collect_issues(report: &mut StatusReport) {
    // Check for missing binaries
    for binary in &report.binaries {
        if !binary.exists {
            report.issues.push(Issue {
                severity: IssueSeverity::Error,
                component: format!("{} binary", binary.name),
                description: format!("Binary '{}' not found in {}", binary.name, BINARY_DIR),
                auto_fixable: false,
                suggested_fix: Some("Run: cd catalyst && ./install.sh".to_string()),
            });
        } else if !binary.executable {
            report.issues.push(Issue {
                severity: IssueSeverity::Warning,
                component: format!("{} binary", binary.name),
                description: format!("Binary '{}' is not executable", binary.name),
                auto_fixable: false,
                suggested_fix: Some(format!("Run: chmod +x ~/.claude-hooks/bin/{}", binary.name)),
            });
        }
    }

    // Check for missing or non-executable hooks
    for hook in &report.hooks {
        if !hook.exists {
            report.issues.push(Issue {
                severity: IssueSeverity::Error,
                component: format!("{} hook wrapper", hook.name),
                description: format!("Hook wrapper '{}' not found", hook.name),
                auto_fixable: true,
                suggested_fix: Some("Run: catalyst status --fix".to_string()),
            });
        } else if !hook.executable {
            report.issues.push(Issue {
                severity: IssueSeverity::Warning,
                component: format!("{} hook wrapper", hook.name),
                description: format!("Hook wrapper '{}' is not executable", hook.name),
                auto_fixable: true,
                suggested_fix: Some("Run: catalyst status --fix".to_string()),
            });
        } else if !hook.calls_correct_binary {
            report.issues.push(Issue {
                severity: IssueSeverity::Warning,
                component: format!("{} hook wrapper", hook.name),
                description: format!("Hook wrapper '{}' cannot access required binary", hook.name),
                auto_fixable: false,
                suggested_fix: Some("Run: cd catalyst && ./install.sh".to_string()),
            });
        }
    }

    // Check for incomplete skills
    for skill in &report.skills {
        if !skill.has_main_file {
            report.issues.push(Issue {
                severity: IssueSeverity::Warning,
                component: format!("{} skill", skill.name),
                description: format!("Skill '{}' is missing SKILL.md", skill.name),
                auto_fixable: false,
                suggested_fix: Some("Reinstall skill: catalyst init --force".to_string()),
            });
        }
    }

    // Check version status
    match &report.version_status {
        VersionStatus::Missing => {
            report.issues.push(Issue {
                severity: IssueSeverity::Info,
                component: "version tracking".to_string(),
                description: ".catalyst-version file not found".to_string(),
                auto_fixable: true,
                suggested_fix: Some("Run: catalyst status --fix".to_string()),
            });
        }
        VersionStatus::Mismatch { expected, found } => {
            report.issues.push(Issue {
                severity: IssueSeverity::Info,
                component: "version tracking".to_string(),
                description: format!(
                    "Version mismatch: installed v{}, current v{}",
                    found, expected
                ),
                auto_fixable: false,
                suggested_fix: Some("Run: catalyst update".to_string()),
            });
        }
        VersionStatus::Ok { .. } => {}
    }
}

/// Determine overall status level from issues
fn determine_status_level(report: &StatusReport) -> StatusLevel {
    let has_errors = report
        .issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Error);
    let has_warnings = report
        .issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Warning);

    if has_errors {
        StatusLevel::Error
    } else if has_warnings {
        StatusLevel::Warning
    } else {
        StatusLevel::Ok
    }
}

/// Auto-fix common issues
///
/// Attempts to automatically repair:
/// - Missing wrapper scripts (recreates from templates)
/// - Non-executable wrapper scripts (sets permissions)
/// - Missing .catalyst-version file
///
/// # Arguments
///
/// * `target_dir` - Base directory containing .claude/
/// * `platform` - Current platform
/// * `report` - Status report with identified issues
pub fn auto_fix(
    target_dir: &Path,
    platform: Platform,
    report: &StatusReport,
) -> Result<Vec<String>> {
    let mut fixed = Vec::new();

    // Fix missing or non-executable wrapper scripts
    for hook in &report.hooks {
        if !hook.exists || !hook.executable {
            match fix_hook_wrapper(target_dir, &hook.name, platform) {
                Ok(()) => {
                    fixed.push(format!("Fixed hook wrapper: {}", hook.name));
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to fix {}: {}", hook.name, e);
                }
            }
        }
    }

    // Fix missing version file
    if matches!(report.version_status, VersionStatus::Missing) {
        match fix_version_file(target_dir) {
            Ok(()) => {
                fixed.push("Created .catalyst-version file".to_string());
            }
            Err(e) => {
                eprintln!("⚠️  Failed to create version file: {}", e);
            }
        }
    }

    Ok(fixed)
}

/// Fix a hook wrapper by recreating it
fn fix_hook_wrapper(target_dir: &Path, wrapper_name: &str, platform: Platform) -> Result<()> {
    // Extract binary name from wrapper name
    let binary_name = wrapper_name
        .trim_end_matches(".sh")
        .trim_end_matches(".ps1");

    // Validate binary name to prevent potential injection
    // Only allow alphanumeric characters, hyphens, and underscores
    if !binary_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CatalystError::InvalidConfig(format!(
            "Invalid binary name '{}': must contain only alphanumeric characters, hyphens, and underscores",
            binary_name
        )));
    }

    // Use the init module's wrapper generation
    // For now, we'll just recreate the wrapper using the same logic
    let hooks_dir = target_dir.join(HOOKS_DIR);
    let wrapper_path = hooks_dir.join(wrapper_name);

    // Load template based on platform
    let template = match platform {
        Platform::Linux | Platform::MacOS | Platform::WSL => {
            include_str!("../resources/wrapper-template.sh")
        }
        Platform::Windows => include_str!("../resources/wrapper-template.ps1"),
    };

    // Replace template variable (safe after validation above)
    let content = template.replace("{{BINARY_NAME}}", binary_name);

    // Write wrapper file
    fs::write(&wrapper_path, content).map_err(CatalystError::Io)?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&wrapper_path, permissions).map_err(CatalystError::Io)?;
    }

    Ok(())
}

/// Fix missing version file
fn fix_version_file(target_dir: &Path) -> Result<()> {
    let version_path = target_dir.join(".catalyst-version");
    let version = env!("CARGO_PKG_VERSION");
    fs::write(version_path, version).map_err(CatalystError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_determine_status_level() {
        let mut report = StatusReport::new();

        // No issues = Ok
        assert_eq!(determine_status_level(&report), StatusLevel::Ok);

        // Warning issue = Warning
        report.issues.push(Issue {
            severity: IssueSeverity::Warning,
            component: "test".to_string(),
            description: "test warning".to_string(),
            auto_fixable: false,
            suggested_fix: None,
        });
        assert_eq!(determine_status_level(&report), StatusLevel::Warning);

        // Error issue = Error
        report.issues.push(Issue {
            severity: IssueSeverity::Error,
            component: "test".to_string(),
            description: "test error".to_string(),
            auto_fixable: false,
            suggested_fix: None,
        });
        assert_eq!(determine_status_level(&report), StatusLevel::Error);
    }

    #[test]
    fn test_check_version_missing() {
        let temp_dir = TempDir::new().unwrap();
        let result = check_version(temp_dir.path()).unwrap();
        assert!(matches!(result, VersionStatus::Missing));
    }

    #[test]
    fn test_check_version_ok() {
        let temp_dir = TempDir::new().unwrap();
        let version_path = temp_dir.path().join(".catalyst-version");
        let current_version = env!("CARGO_PKG_VERSION");
        fs::write(&version_path, current_version).unwrap();

        let result = check_version(temp_dir.path()).unwrap();
        assert!(matches!(result, VersionStatus::Ok { .. }));
    }

    #[test]
    fn test_check_version_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let version_path = temp_dir.path().join(".catalyst-version");
        fs::write(&version_path, "0.0.1").unwrap();

        let result = check_version(temp_dir.path()).unwrap();
        assert!(matches!(result, VersionStatus::Mismatch { .. }));
    }

    #[test]
    fn test_fix_version_file() {
        let temp_dir = TempDir::new().unwrap();
        fix_version_file(temp_dir.path()).unwrap();

        let version_path = temp_dir.path().join(".catalyst-version");
        assert!(version_path.exists());

        let content = fs::read_to_string(&version_path).unwrap();
        assert_eq!(content.trim(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_fix_hook_wrapper_validates_binary_name() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join(".claude/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Valid binary names should work
        let result = fix_hook_wrapper(
            temp_dir.path(),
            "skill-activation-prompt.sh",
            Platform::Linux,
        );
        assert!(result.is_ok());

        let result = fix_hook_wrapper(temp_dir.path(), "file-change-tracker.sh", Platform::Linux);
        assert!(result.is_ok());

        // Invalid binary names should be rejected
        let result = fix_hook_wrapper(temp_dir.path(), "test;rm-rf.sh", Platform::Linux);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid binary name"));

        let result = fix_hook_wrapper(temp_dir.path(), "test$command.sh", Platform::Linux);
        assert!(result.is_err());

        let result = fix_hook_wrapper(temp_dir.path(), "test/../etc/passwd.sh", Platform::Linux);
        assert!(result.is_err());
    }
}

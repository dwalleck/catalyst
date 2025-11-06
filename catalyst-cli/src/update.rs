//! Update logic for Catalyst CLI
//!
//! This module handles the `catalyst update` command, which updates an existing
//! Catalyst installation while preserving user customizations.

use crate::init::{generate_wrapper_scripts, read_version_file, write_version_file};
use crate::types::{
    CatalystError, CatalystHashes, Platform, Result, SkippedSkill, UpdateReport, CATALYST_VERSION,
    HASHES_FILE, SKILLS_DIR,
};
use include_dir::{include_dir, Dir};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

// Embed skills directory at compile time (same as in init.rs)
static SKILLS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../.claude/skills");

/// Update an existing Catalyst installation
///
/// This function:
/// 1. Checks the installed version
/// 2. Updates wrapper scripts (graceful - continues on error)
/// 3. Updates skills with hash-based modification detection (graceful)
/// 4. Writes new version file (FATAL - fails entire update if unsuccessful)
///
/// # Arguments
///
/// * `target_dir` - Directory where Catalyst is installed
/// * `force` - Whether to overwrite modified files
///
/// # Returns
///
/// Returns an `UpdateReport` with details of what was updated
///
/// # Error Recovery Strategy
///
/// Wrapper script and skill updates use graceful degradation - they continue
/// on error and report issues. However, version file write is FATAL because:
/// - The version file is critical state for the update system
/// - If it fails to update, subsequent `update` commands will be confused
/// - Users would experience confusing repeated update attempts
/// - Better to fail loudly than enter an inconsistent state
pub fn update(target_dir: &Path, force: bool) -> Result<UpdateReport> {
    let mut report = UpdateReport::new();

    // Read installed version
    let installed_version = match read_version_file(target_dir)? {
        Some(v) => v,
        None => {
            return Err(CatalystError::InvalidConfig(
                "No .catalyst-version file found. This directory may not be initialized. Try 'catalyst init' first.".to_string()
            ));
        }
    };

    // Compare versions
    if installed_version == CATALYST_VERSION && !force {
        // Already up to date
        report.success = true;
        return Ok(report);
    }

    // Phase 6.2: Update wrapper scripts (graceful degradation)
    let platform = Platform::detect();
    match generate_wrapper_scripts(target_dir, true, true, platform) {
        Ok(hooks) => {
            report.updated_hooks = hooks;
        }
        Err(e) => {
            let error = format!("Failed to update wrapper scripts: {}", e);
            report.errors.push(error.clone());
            report.success = false;
            eprintln!("⚠️  {}", error);
        }
    }

    // Phase 6.3: Update skills with hash-based detection (graceful degradation)
    match update_skills(target_dir, force) {
        Ok((updated, skipped)) => {
            report.updated_skills = updated;
            report.skipped_skills = skipped;
        }
        Err(e) => {
            let error = format!("Failed to update skills: {}", e);
            report.errors.push(error.clone());
            report.success = false;
            eprintln!("⚠️  {}", error);
        }
    }

    // Write new version file - FATAL error because version file is critical state
    // If this fails, the entire update should be considered failed to avoid
    // inconsistent state where updates were applied but version wasn't recorded
    write_version_file(target_dir)?;

    Ok(report)
}

/// Update skills using hash-based modification detection
///
/// # Arguments
///
/// * `target_dir` - Directory where skills are installed
/// * `force` - Whether to overwrite modified files
///
/// # Returns
///
/// Returns a tuple of (updated_skills, skipped_skills)
///
/// # Implementation Note
///
/// Avoids TOCTOU race by directly reading the hashes file without checking
/// existence first. Missing files are handled as NotFound errors.
fn update_skills(target_dir: &Path, force: bool) -> Result<(Vec<String>, Vec<SkippedSkill>)> {
    let mut updated = Vec::new();
    let mut skipped = Vec::new();

    // Read existing hashes - avoid TOCTOU race by attempting read directly
    let hashes_path = target_dir.join(HASHES_FILE);
    let stored_hashes: CatalystHashes = match fs::read_to_string(&hashes_path) {
        Ok(content) => serde_json::from_str(&content).map_err(CatalystError::Json)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // No hashes file, can't determine modifications
            return Ok((updated, skipped));
        }
        Err(e) => {
            return Err(CatalystError::FileReadFailed {
                path: hashes_path,
                source: e,
            })
        }
    };

    let skills_dir = target_dir.join(SKILLS_DIR);

    // Iterate through installed skills
    for (skill_name, expected_hash) in &stored_hashes.skills {
        let skill_path = skills_dir.join(skill_name).join("SKILL.md");

        // Compute current hash - handle missing files gracefully
        let current_hash = match compute_file_hash(&skill_path) {
            Ok(hash) => hash,
            Err(CatalystError::FileReadFailed { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                // Skill was removed, skip silently
                continue;
            }
            Err(e) => return Err(e),
        };

        // Check if modified
        if current_hash != *expected_hash && !force {
            // Skill was modified by user, skip update
            skipped.push(SkippedSkill {
                name: skill_name.clone(),
                reason: "Modified locally".to_string(),
                current_hash,
                expected_hash: expected_hash.clone(),
            });
            continue;
        }

        // Update skill (copy from embedded resources)
        if let Some(skill_dir) = SKILLS.get_dir(skill_name) {
            // Copy skill files
            copy_skill_files(skill_dir, &skills_dir.join(skill_name))?;
            updated.push(skill_name.clone());
        }
    }

    // Regenerate hashes for updated skills
    if !updated.is_empty() {
        regenerate_hashes(target_dir, &updated)?;
    }

    Ok((updated, skipped))
}

/// Compute SHA256 hash of a file
///
/// # Errors
///
/// Returns `FileReadFailed` with the file path if reading fails
fn compute_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path).map_err(|e| CatalystError::FileReadFailed {
        path: file_path.to_path_buf(),
        source: e,
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Copy skill files from embedded resources to target directory
///
/// # Errors
///
/// Returns detailed errors with file paths for:
/// - Directory creation failures
/// - File write failures
/// - Invalid subdirectory paths
fn copy_skill_files(source_dir: &include_dir::Dir, target_dir: &Path) -> Result<()> {
    // Create target directory
    fs::create_dir_all(target_dir).map_err(|e| CatalystError::DirectoryCreationFailed {
        path: target_dir.to_path_buf(),
        source: e,
    })?;

    // Copy all files
    for file in source_dir.files() {
        let target_path = target_dir.join(file.path());

        // Create parent directories if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| CatalystError::DirectoryCreationFailed {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Write file with error context
        fs::write(&target_path, file.contents()).map_err(|e| CatalystError::FileWriteFailed {
            path: target_path.clone(),
            source: e,
        })?;
    }

    // Recursively copy subdirectories
    for subdir in source_dir.dirs() {
        let file_name = subdir.path().file_name().ok_or_else(|| {
            CatalystError::InvalidPath(format!(
                "Invalid subdirectory path (missing file name): {}",
                subdir.path().display()
            ))
        })?;
        let target_subdir = target_dir.join(file_name);
        copy_skill_files(subdir, &target_subdir)?;
    }

    Ok(())
}

/// Regenerate .catalyst-hashes.json for updated skills
///
/// # Errors
///
/// Returns detailed errors with file paths for:
/// - Reading existing hash file
/// - Computing skill file hashes
/// - Writing updated hash file
///
/// # Implementation Note
///
/// Avoids TOCTOU race by directly attempting to read the hash file
fn regenerate_hashes(target_dir: &Path, updated_skills: &[String]) -> Result<()> {
    let hashes_path = target_dir.join(HASHES_FILE);

    // Read existing hashes - avoid TOCTOU race by attempting read directly
    let mut hashes: CatalystHashes = match fs::read_to_string(&hashes_path) {
        Ok(content) => serde_json::from_str(&content).map_err(CatalystError::Json)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            CatalystHashes::new(CATALYST_VERSION.to_string())
        }
        Err(e) => {
            return Err(CatalystError::FileReadFailed {
                path: hashes_path.clone(),
                source: e,
            })
        }
    };

    let skills_dir = target_dir.join(SKILLS_DIR);

    // Update hashes for updated skills
    for skill_name in updated_skills {
        let skill_path = skills_dir.join(skill_name).join("SKILL.md");
        // compute_file_hash will handle missing files with proper error
        // For regenerate, we only hash skills that were successfully updated
        match compute_file_hash(&skill_path) {
            Ok(hash) => {
                hashes.skills.insert(skill_name.clone(), hash);
            }
            Err(CatalystError::FileReadFailed { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                // Skill file missing - skip it but don't fail the whole operation
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    // Update version and timestamp
    hashes.version = CATALYST_VERSION.to_string();
    hashes.updated_at = chrono::Utc::now().to_rfc3339();

    // Write updated hashes with proper error context
    let json = serde_json::to_string_pretty(&hashes).map_err(CatalystError::Json)?;
    fs::write(&hashes_path, &json).map_err(|e| CatalystError::FileWriteFailed {
        path: hashes_path.clone(),
        source: e,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compute_file_hash_success() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"test content").unwrap();

        let hash = compute_file_hash(&test_file).unwrap();
        // SHA256 of "test content" is a specific value
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_compute_file_hash_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let missing_file = temp_dir.path().join("missing.txt");

        let result = compute_file_hash(&missing_file);
        assert!(result.is_err());
        match result {
            Err(CatalystError::FileReadFailed { path, source }) => {
                assert_eq!(path, missing_file);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected FileReadFailed with NotFound error"),
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_compute_file_hash_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"test content").unwrap();

        // Make file unreadable
        fs::set_permissions(&test_file, fs::Permissions::from_mode(0o000)).unwrap();

        let result = compute_file_hash(&test_file);
        assert!(result.is_err());
        match result {
            Err(CatalystError::FileReadFailed { path, source }) => {
                assert_eq!(path, test_file);
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected FileReadFailed with PermissionDenied error"),
        }

        // Clean up
        fs::set_permissions(&test_file, fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn test_read_version_file_missing_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let result = read_version_file(temp_dir.path()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_regenerate_hashes_handles_missing_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create skills directory
        fs::create_dir_all(target.join(".claude/skills")).unwrap();

        // Call with no existing hash file - should create new one
        let result = regenerate_hashes(target, &[]);
        assert!(result.is_ok());

        // Verify hash file was created
        let hash_file = target.join(".catalyst-hashes.json");
        assert!(hash_file.exists());
    }

    #[test]
    fn test_regenerate_hashes_missing_skill_file() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create skills directory
        fs::create_dir_all(target.join(".claude/skills/missing-skill")).unwrap();

        // Try to regenerate hash for non-existent skill file
        // Should not fail, just skip the missing file
        let result = regenerate_hashes(target, &["missing-skill".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_copy_skill_files_with_error_context() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Make directory read-only
        fs::set_permissions(target, fs::Permissions::from_mode(0o555)).unwrap();

        // Use empty embedded dir for test
        static EMPTY_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../.claude/skills");
        if let Some(skill_dir) = EMPTY_DIR.get_dir("skill-developer") {
            let result = copy_skill_files(skill_dir, &target.join("test-skill"));
            assert!(result.is_err());
            match result {
                Err(CatalystError::DirectoryCreationFailed { path, source }) => {
                    assert!(path.ends_with("test-skill"));
                    assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
                }
                _ => panic!("Expected DirectoryCreationFailed with context"),
            }
        }

        // Clean up
        fs::set_permissions(target, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

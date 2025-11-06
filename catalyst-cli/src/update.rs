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
/// 2. Updates wrapper scripts
/// 3. Updates skills (hash-based detection of modifications)
/// 4. Writes new version file
///
/// # Arguments
///
/// * `target_dir` - Directory where Catalyst is installed
/// * `force` - Whether to overwrite modified files
///
/// # Returns
///
/// Returns an `UpdateReport` with details of what was updated
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

    // Phase 6.2: Update wrapper scripts
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

    // Phase 6.3: Update skills with hash-based detection
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

    // Write new version file
    if let Err(e) = write_version_file(target_dir) {
        let error = format!("Failed to write version file: {}", e);
        report.errors.push(error.clone());
        report.success = false;
        eprintln!("⚠️  {}", error);
    }

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
fn update_skills(target_dir: &Path, force: bool) -> Result<(Vec<String>, Vec<SkippedSkill>)> {
    let mut updated = Vec::new();
    let mut skipped = Vec::new();

    // Read existing hashes
    let hashes_path = target_dir.join(HASHES_FILE);
    let stored_hashes: CatalystHashes = if hashes_path.exists() {
        let content = fs::read_to_string(&hashes_path).map_err(CatalystError::Io)?;
        serde_json::from_str(&content).map_err(CatalystError::Json)?
    } else {
        // No hashes file, can't determine modifications
        return Ok((updated, skipped));
    };

    let skills_dir = target_dir.join(SKILLS_DIR);

    // Iterate through installed skills
    for (skill_name, expected_hash) in &stored_hashes.skills {
        let skill_path = skills_dir.join(skill_name).join("SKILL.md");

        if !skill_path.exists() {
            // Skill was removed, skip
            continue;
        }

        // Compute current hash
        let current_hash = compute_file_hash(&skill_path)?;

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
fn compute_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path).map_err(CatalystError::Io)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Copy skill files from embedded resources to target directory
fn copy_skill_files(source_dir: &include_dir::Dir, target_dir: &Path) -> Result<()> {
    // Create target directory
    fs::create_dir_all(target_dir).map_err(CatalystError::Io)?;

    // Copy all files
    for file in source_dir.files() {
        let target_path = target_dir.join(file.path());

        // Create parent directories
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(CatalystError::Io)?;
        }

        // Write file
        fs::write(&target_path, file.contents()).map_err(CatalystError::Io)?;
    }

    // Recursively copy subdirectories
    for subdir in source_dir.dirs() {
        let target_subdir = target_dir.join(subdir.path().file_name().unwrap());
        copy_skill_files(subdir, &target_subdir)?;
    }

    Ok(())
}

/// Regenerate .catalyst-hashes.json for updated skills
fn regenerate_hashes(target_dir: &Path, updated_skills: &[String]) -> Result<()> {
    let hashes_path = target_dir.join(HASHES_FILE);

    // Read existing hashes
    let mut hashes: CatalystHashes = if hashes_path.exists() {
        let content = fs::read_to_string(&hashes_path).map_err(CatalystError::Io)?;
        serde_json::from_str(&content).map_err(CatalystError::Json)?
    } else {
        CatalystHashes::new(CATALYST_VERSION.to_string())
    };

    let skills_dir = target_dir.join(SKILLS_DIR);

    // Update hashes for updated skills
    for skill_name in updated_skills {
        let skill_path = skills_dir.join(skill_name).join("SKILL.md");
        if skill_path.exists() {
            let hash = compute_file_hash(&skill_path)?;
            hashes.skills.insert(skill_name.clone(), hash);
        }
    }

    // Update version and timestamp
    hashes.version = CATALYST_VERSION.to_string();
    hashes.updated_at = chrono::Utc::now().to_rfc3339();

    // Write updated hashes
    let json = serde_json::to_string_pretty(&hashes).map_err(CatalystError::Json)?;
    fs::write(&hashes_path, json).map_err(CatalystError::Io)?;

    Ok(())
}

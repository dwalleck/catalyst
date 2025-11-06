//! Initialization logic for Catalyst CLI
//!
//! This module handles the `catalyst init` command, which creates the .claude/
//! directory structure, installs hooks, and sets up skills.

use crate::types::{
    CatalystError, InitConfig, InitReport, Platform, Result, AGENTS_DIR, AVAILABLE_SKILLS,
    CATALYST_VERSION, CLAUDE_DIR, COMMANDS_DIR, HOOKS_DIR, SKILLS_DIR, VERSION_FILE,
};
use include_dir::{include_dir, Dir};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process;
use tempfile::NamedTempFile;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Embed wrapper templates at compile time
const WRAPPER_TEMPLATE_SH: &str = include_str!("../resources/wrapper-template.sh");
const WRAPPER_TEMPLATE_PS1: &str = include_str!("../resources/wrapper-template.ps1");

// Embed skills directory at compile time
static SKILLS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../.claude/skills");

/// Lock file name for concurrent init protection
const LOCK_FILE: &str = ".catalyst.lock";

/// EXDEV error code (cross-device link) on Unix systems
#[cfg(unix)]
const EXDEV: i32 = 18;

/// Guard that automatically releases the lock when dropped
///
/// # Lock Cleanup Guarantee
///
/// The lock is **automatically released** when this guard is dropped, even if
/// initialization fails or panics. This RAII pattern ensures that:
/// - Lock files are never leaked on normal program exit
/// - Subsequent initialization attempts can proceed after errors
/// - No manual cleanup is required in error handling paths
///
/// The Drop trait makes lock cleanup exception-safe and foolproof.
pub struct InitLock {
    lock_file: PathBuf,
}

impl Drop for InitLock {
    fn drop(&mut self) {
        let _ = release_init_lock(&self.lock_file);
    }
}

/// Helper function to atomically create a lock file and write PID
///
/// # Arguments
///
/// * `lock_file` - Path to the lock file
/// * `pid` - Process ID to write to the lock file
///
/// # Returns
///
/// Returns an `InitLock` guard or an I/O error
fn try_create_lock_file(lock_file: &Path, pid: u32) -> Result<InitLock> {
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true) // Atomic check-and-create
        .open(lock_file)
        .map_err(CatalystError::Io)?;

    write!(file, "{}", pid).map_err(CatalystError::Io)?;

    Ok(InitLock {
        lock_file: lock_file.to_path_buf(),
    })
}

/// Acquire a lock to prevent concurrent init operations
///
/// Creates a .catalyst.lock file with the current process ID using atomic file creation.
/// Returns an error if a lock already exists and the process is still running.
///
/// # Arguments
///
/// * `target_dir` - The directory being initialized
///
/// # Returns
///
/// Returns an `InitLock` guard that will automatically release the lock when dropped
///
/// # Concurrency Safety
///
/// Uses atomic file creation (O_EXCL on Unix, CREATE_NEW on Windows) to prevent
/// race conditions where two processes might both acquire the lock.
pub fn acquire_init_lock(target_dir: &Path) -> Result<InitLock> {
    let lock_file = target_dir.join(LOCK_FILE);
    let current_pid = process::id();

    // Try to atomically create the lock file
    // This prevents TOCTOU race conditions
    match try_create_lock_file(&lock_file, current_pid) {
        Ok(lock) => Ok(lock),
        Err(CatalystError::Io(e)) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lock file exists - check if it's stale
            let pid_str = fs::read_to_string(&lock_file).map_err(CatalystError::Io)?;

            match pid_str.trim().parse::<u32>() {
                Ok(pid) if is_valid_pid(pid) => {
                    // Valid PID - check if process is still running
                    if is_process_running(pid) {
                        Err(CatalystError::InitInProgress {
                            pid,
                            lock_file: lock_file.display().to_string(),
                        })
                    } else {
                        // Stale lock file - remove and retry once
                        fs::remove_file(&lock_file).map_err(CatalystError::Io)?;

                        // NOTE: There is a small TOCTOU race window between remove_file and
                        // try_create_lock_file where another process could create the lock.
                        // This is acceptable because:
                        // 1. The window is extremely small (microseconds)
                        // 2. If it happens, try_create_lock_file will fail with AlreadyExists,
                        //    causing this init to fail cleanly
                        // 3. The race is rare in practice (requires precise timing)
                        // 4. The failure is safe - no corruption or data loss

                        // Retry lock acquisition (non-recursive)
                        try_create_lock_file(&lock_file, current_pid)
                    }
                }
                _ => {
                    // Invalid PID (0, 1, current, or parse error) - treat as stale
                    fs::remove_file(&lock_file).map_err(CatalystError::Io)?;

                    // NOTE: Known TOCTOU race window here (see comment above)
                    // Retry lock acquisition
                    try_create_lock_file(&lock_file, current_pid)
                }
            }
        }
        Err(e) => Err(e),
    }
}

/// Validate that a PID is reasonable
///
/// Returns false for:
/// - PID 0 (invalid)
/// - PID 1 (system process, likely malicious lock file)
///
/// Note: We intentionally allow checking our own PID. If the lock file contains
/// our PID, we'll check is_process_running() which will return true, causing
/// the lock acquisition to fail with InitInProgress. This prevents the same
/// process from acquiring the lock twice.
fn is_valid_pid(pid: u32) -> bool {
    pid != 0 && pid != 1
}

/// Release the init lock
fn release_init_lock(lock_file: &Path) -> Result<()> {
    if lock_file.exists() {
        fs::remove_file(lock_file).map_err(CatalystError::Io)?;
    }
    Ok(())
}

/// Check if a process is running on the current system
///
/// # Platform-specific behavior
///
/// - **Unix/Linux/macOS**: Uses `kill -0 pid` to check if process exists
/// - **Windows**: Uses OpenProcess to check if process exists
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

    // On Unix, use kill -0 to check if process exists
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // Try to open the process with minimal access rights
    //
    // NOTE: We could use GetExitCodeProcess for more certainty, but OpenProcess
    // with PROCESS_QUERY_LIMITED_INFORMATION is sufficient and requires minimal
    // permissions. The conservative error handling (assume exists on unknown errors)
    // provides adequate safety for lock file cleanup.
    //
    // SAFETY: This is safe because we're just checking if a process exists
    // and we immediately close the handle if successful
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);

        if handle == 0 {
            // Failed to open process - check why
            let error = GetLastError();

            // Explicitly handle known error cases:
            // - ERROR_INVALID_PARAMETER (87): Process definitely doesn't exist
            // - ERROR_ACCESS_DENIED (5): Process exists but is protected (system/elevated)
            // - Other errors: Conservatively assume process exists to avoid stale lock cleanup race
            //
            // This conservative approach prevents accidentally cleaning up locks for
            // running processes in edge cases (network errors, permission issues, etc.)
            match error {
                ERROR_INVALID_PARAMETER => false, // Process doesn't exist
                ERROR_ACCESS_DENIED => true,      // Process exists but protected
                _ => {
                    // Unknown error - be conservative and assume process exists
                    // This prevents false positives that could cause concurrent init
                    true
                }
            }
        } else {
            // Successfully opened - process exists
            CloseHandle(handle);
            true
        }
    }
}

/// Create the .claude subdirectory structure
///
/// First checks that .claude/ exists (created by Claude Code).
/// Then creates:
/// - .claude/hooks/
/// - .claude/skills/
/// - .claude/agents/
/// - .claude/commands/
///
/// Sets permissions to 0755 on Unix systems.
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude exists
/// * `force` - Whether to proceed even if directories exist
///
/// # Returns
///
/// Returns a list of directory paths that were created
///
/// # Errors
///
/// Returns an error if .claude directory doesn't exist, indicating Claude Code
/// hasn't been initialized in this project.
pub fn create_directory_structure(target_dir: &Path, force: bool) -> Result<Vec<String>> {
    let mut created_dirs = Vec::new();

    // First, verify .claude directory exists (created by Claude Code)
    let claude_dir = target_dir.join(CLAUDE_DIR);
    if !claude_dir.exists() {
        return Err(CatalystError::InvalidPath(
            format!(
                ".claude directory not found at {}\n\
                \n\
                Catalyst requires Claude Code to be initialized first.\n\
                Please ensure you're running this command in a Claude Code project.\n\
                \n\
                If this is a new project, open it in Claude Code first - it will create the .claude directory automatically.",
                claude_dir.display()
            )
        ));
    }

    if !claude_dir.is_dir() {
        return Err(CatalystError::InvalidPath(format!(
            "{} exists but is not a directory",
            claude_dir.display()
        )));
    }

    // Now create subdirectories
    let subdirs = vec![HOOKS_DIR, SKILLS_DIR, AGENTS_DIR, COMMANDS_DIR];

    for dir in subdirs {
        let dir_path = target_dir.join(dir);

        // Check if directory exists
        if dir_path.exists() && !force {
            // Check if it's actually a directory
            if !dir_path.is_dir() {
                return Err(CatalystError::InvalidPath(format!(
                    "{} exists but is not a directory",
                    dir_path.display()
                )));
            }
            // Directory exists - skip it (idempotent)
            continue;
        }

        // Create directory
        fs::create_dir_all(&dir_path).map_err(CatalystError::Io)?;

        // Verify directory was created successfully
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(CatalystError::InvalidPath(format!(
                "Failed to create directory: {}",
                dir_path.display()
            )));
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&dir_path, permissions).map_err(CatalystError::Io)?;
        }

        created_dirs.push(dir.to_string());
    }

    Ok(created_dirs)
}

/// Generate wrapper scripts for hooks
///
/// Creates wrapper scripts that call the installed binaries.
/// On Unix: Creates .sh scripts with executable permissions
/// On Windows: Creates .ps1 PowerShell scripts
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude/hooks/ exists
/// * `install_hooks` - Whether to install skill-activation-prompt wrapper
/// * `install_tracker` - Whether to install file-change-tracker wrapper
/// * `platform` - Target platform for wrapper generation
///
/// # Returns
///
/// Returns a list of wrapper file paths that were created
pub fn generate_wrapper_scripts(
    target_dir: &Path,
    install_hooks: bool,
    install_tracker: bool,
    platform: Platform,
) -> Result<Vec<String>> {
    let mut installed = Vec::new();
    let hooks_dir = target_dir.join(HOOKS_DIR);

    // Determine which template to use based on platform
    let (template, extension) = match platform {
        Platform::Windows => (WRAPPER_TEMPLATE_PS1, "ps1"),
        _ => (WRAPPER_TEMPLATE_SH, "sh"),
    };

    // Generate skill-activation-prompt wrapper
    if install_hooks {
        let binary_name = "skill-activation-prompt";
        let wrapper_name = format!("{}.{}", binary_name, extension);
        let wrapper_path = hooks_dir.join(&wrapper_name);

        let content = template.replace("{{BINARY_NAME}}", binary_name);
        fs::write(&wrapper_path, content).map_err(CatalystError::Io)?;

        // Set executable permission on Unix
        #[cfg(unix)]
        if matches!(platform, Platform::Linux | Platform::MacOS | Platform::WSL) {
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&wrapper_path, permissions).map_err(CatalystError::Io)?;
        }

        installed.push(wrapper_name);
    }

    // Generate file-change-tracker wrapper
    if install_tracker {
        let binary_name = "file-change-tracker";
        let wrapper_name = format!("{}.{}", binary_name, extension);
        let wrapper_path = hooks_dir.join(&wrapper_name);

        let content = template.replace("{{BINARY_NAME}}", binary_name);
        fs::write(&wrapper_path, content).map_err(CatalystError::Io)?;

        // Set executable permission on Unix
        #[cfg(unix)]
        if matches!(platform, Platform::Linux | Platform::MacOS | Platform::WSL) {
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&wrapper_path, permissions).map_err(CatalystError::Io)?;
        }

        installed.push(wrapper_name);
    }

    Ok(installed)
}

/// Write content to a file atomically with fallback to regular write
///
/// Attempts to use atomic write (temp file + persist) first for safety.
/// Falls back to regular write if:
/// - Cross-device link error (EXDEV) - common with Docker volumes, network file systems
/// - Temp file creation fails
///
/// # Arguments
///
/// * `path` - Target file path
/// * `content` - Content to write
///
/// # Returns
///
/// Returns `Ok(true)` if atomic write succeeded, `Ok(false)` if fallback was used,
/// or an error if both methods failed.
pub fn write_file_atomic(path: &Path, content: &str) -> Result<bool> {
    // Try atomic write first
    match try_atomic_write(path, content) {
        Ok(()) => Ok(true), // Atomic write succeeded
        Err(e) => {
            // Check if it's a cross-device link error or temp creation failure
            if is_cross_device_error(&e) || is_temp_creation_error(&e) {
                // Fall back to regular write
                eprintln!("⚠️  Atomic write not supported on this filesystem");
                eprintln!("   Reason: {}", e);
                eprintln!("   Falling back to regular write for: {}", path.display());

                fs::write(path, content).map_err(CatalystError::Io)?;

                Ok(false) // Fallback was used
            } else {
                // Other error - propagate it
                Err(CatalystError::Io(e))
            }
        }
    }
}

/// Attempt atomic write using temp file + persist
fn try_atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    // Get parent directory for temp file
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path has no parent directory",
        )
    })?;

    // Create temp file in same directory
    let mut temp_file = NamedTempFile::new_in(parent)?;

    // Write content
    temp_file.write_all(content.as_bytes())?;

    // Flush to disk
    temp_file.flush()?;

    // Atomically persist (rename) to final location
    temp_file.persist(path)?;

    Ok(())
}

/// Check if error is a cross-device link error (EXDEV)
fn is_cross_device_error(e: &std::io::Error) -> bool {
    #[cfg(unix)]
    {
        e.raw_os_error() == Some(EXDEV)
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, check error kind
        matches!(e.kind(), std::io::ErrorKind::Other)
    }
}

/// Check if error is related to temp file creation
fn is_temp_creation_error(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::NotFound
    )
}

/// Create settings.json with hook configurations
///
/// Generates a settings.json file with:
/// - UserPromptSubmit hook for skill-activation-prompt
/// - PostToolUse hook for file-change-tracker (if enabled)
///
/// Uses platform-appropriate wrapper file extensions (.sh or .ps1).
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude/ exists
/// * `install_hooks` - Whether to add skill-activation-prompt hook
/// * `install_tracker` - Whether to add file-change-tracker hook
/// * `platform` - Target platform (determines file extension)
///
/// # Returns
///
/// Returns `Ok(true)` if settings.json was created
pub fn create_settings_json(
    target_dir: &Path,
    install_hooks: bool,
    install_tracker: bool,
    platform: Platform,
) -> Result<bool> {
    let settings_path = target_dir.join(".claude/settings.json");

    // Determine wrapper extension
    let extension = platform.hook_extension();

    // Build hooks array
    let mut hooks = Vec::new();

    // Add skill-activation-prompt hook
    if install_hooks {
        hooks.push(serde_json::json!({
            "event": "UserPromptSubmit",
            "script": format!("$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.{}", extension),
            "async": false
        }));
    }

    // Add file-change-tracker hook
    if install_tracker {
        hooks.push(serde_json::json!({
            "event": "PostToolUse",
            "script": format!("$CLAUDE_PROJECT_DIR/.claude/hooks/file-change-tracker.{}", extension),
            "async": false,
            "matchers": [
                {
                    "toolName": "Write"
                },
                {
                    "toolName": "Edit"
                },
                {
                    "toolName": "MultiEdit"
                }
            ]
        }));
    }

    // Create settings JSON
    let settings = serde_json::json!({
        "hooks": hooks
    });

    // Pretty-print JSON
    let content = serde_json::to_string_pretty(&settings).map_err(CatalystError::Json)?;

    // Write atomically
    write_file_atomic(&settings_path, &content)?;

    Ok(true)
}

/// Install skills from embedded resources
///
/// Extracts skills from the embedded SKILLS directory and installs them
/// to the target `.claude/skills/` directory.
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude exists
/// * `skill_ids` - List of skill IDs to install
/// * `force` - Whether to overwrite existing skill directories
///
/// # Returns
///
/// Returns a list of successfully installed skill IDs
pub fn install_skills(target_dir: &Path, skill_ids: &[String], force: bool) -> Result<Vec<String>> {
    let mut installed = Vec::new();

    // Skip progress bar if no skills to install
    if skill_ids.is_empty() {
        return Ok(installed);
    }

    // Only show progress bar if stdout is a terminal
    let use_progress = io::stdout().is_terminal();

    let pb = if use_progress {
        let pb = ProgressBar::new(skill_ids.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                // Template is static and valid, but use fallback as defensive programming
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("━━╸"),
        );
        Some(pb)
    } else {
        None
    };

    for skill_id in skill_ids {
        if let Some(ref pb) = pb {
            pb.set_message(format!("Installing {}...", skill_id));
        }

        match install_skill(target_dir, skill_id, force) {
            Ok(()) => {
                installed.push(skill_id.clone());
                if pb.is_none() {
                    // If no progress bar, print messages directly
                    println!("  ✓ Installed {}", skill_id);
                }
            }
            Err(e) => {
                let error_msg = format!("⚠️  Failed to install skill '{}': {}", skill_id, e);
                if let Some(ref pb) = pb {
                    pb.println(error_msg);
                } else {
                    eprintln!("{}", error_msg);
                }
            }
        }

        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    if let Some(ref pb) = pb {
        pb.finish_with_message(format!(
            "✅ Installed {} skill{}",
            installed.len(),
            if installed.len() == 1 { "" } else { "s" }
        ));
    }

    Ok(installed)
}

/// Install a single skill from embedded resources
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude exists
/// * `skill_id` - The skill ID to install
/// * `force` - Whether to overwrite existing skill directory
fn install_skill(target_dir: &Path, skill_id: &str, force: bool) -> Result<()> {
    // Validate skill ID against available skills
    if !AVAILABLE_SKILLS.contains(&skill_id) {
        return Err(CatalystError::InvalidConfig(format!(
            "Invalid skill ID: '{}'. Available skills: {}",
            skill_id,
            AVAILABLE_SKILLS.join(", ")
        )));
    }

    let skills_dir = target_dir.join(SKILLS_DIR);
    let skill_target = skills_dir.join(skill_id);

    // Check if skill directory already exists
    if skill_target.exists() && !force {
        return Err(CatalystError::InvalidPath(format!(
            "Skill directory already exists: {}\nUse --force to overwrite.",
            skill_target.display()
        )));
    }

    // Find the skill in embedded resources
    let skill_dir = SKILLS
        .get_dir(skill_id)
        .ok_or_else(|| CatalystError::InvalidPath(format!("Skill not found: {}", skill_id)))?;

    // Create skill directory
    fs::create_dir_all(&skill_target).map_err(CatalystError::Io)?;

    // Copy all files recursively
    copy_dir_recursive(skill_dir, &skill_target)?;

    // Set permissions on Unix
    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&skill_target, permissions).map_err(CatalystError::Io)?;
    }

    Ok(())
}

/// Recursively copy directory contents from embedded resources
fn copy_dir_recursive(source: &include_dir::Dir, target: &Path) -> Result<()> {
    // Copy all files in this directory
    for file in source.files() {
        let file_name = file.path().file_name().ok_or_else(|| {
            CatalystError::InvalidPath(format!("Invalid file path: {:?}", file.path()))
        })?;
        let file_path = target.join(file_name);
        fs::write(&file_path, file.contents()).map_err(CatalystError::Io)?;

        // Set executable permission on Unix if needed
        #[cfg(unix)]
        {
            let permissions = fs::Permissions::from_mode(0o644);
            fs::set_permissions(&file_path, permissions).map_err(CatalystError::Io)?;
        }
    }

    // Recursively copy subdirectories
    for subdir in source.dirs() {
        let subdir_name = subdir.path().file_name().ok_or_else(|| {
            CatalystError::InvalidPath(format!("Invalid directory path: {:?}", subdir.path()))
        })?;
        let subdir_path = target.join(subdir_name);
        fs::create_dir_all(&subdir_path).map_err(CatalystError::Io)?;
        copy_dir_recursive(subdir, &subdir_path)?;
    }

    Ok(())
}

/// Generate skill-rules.json for installed skills
///
/// Creates the skill-rules.json file with activation rules for each installed skill.
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude exists
/// * `installed_skills` - List of skill IDs that were installed
pub fn generate_skill_rules(target_dir: &Path, installed_skills: &[String]) -> Result<()> {
    let skill_rules_path = target_dir.join(SKILLS_DIR).join("skill-rules.json");

    let mut rules = serde_json::json!({
        "version": "1.0",
        "skills": {}
    });

    let skills_obj = rules
        .get_mut("skills")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| {
            CatalystError::InvalidConfig("Failed to access skills object in JSON".to_string())
        })?;

    for skill_id in installed_skills {
        let (keywords, intent_patterns, path_patterns) = get_skill_patterns(skill_id);

        skills_obj.insert(
            skill_id.clone(),
            serde_json::json!({
                "type": "skill",
                "enforcement": "suggest",
                "priority": 1,
                "keywords": keywords,
                "intentPatterns": intent_patterns,
                "pathPatterns": path_patterns,
                "enabled": true
            }),
        );
    }

    // Pretty-print JSON with comment
    let mut content = String::from("// Customize pathPatterns for your project structure\n");
    content.push_str(&serde_json::to_string_pretty(&rules).map_err(CatalystError::Json)?);

    // Write atomically
    write_file_atomic(&skill_rules_path, &content)?;

    Ok(())
}

/// Get skill-specific patterns (keywords, intent, and path patterns)
fn get_skill_patterns(skill_id: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    match skill_id {
        "frontend-dev-guidelines" => (
            vec!["frontend".to_string(), "react".to_string()],
            vec![
                "frontend development".to_string(),
                "react component".to_string(),
            ],
            vec!["**/*.{ts,tsx,js,jsx,vue,svelte}".to_string()],
        ),
        "backend-dev-guidelines" => (
            vec!["backend".to_string(), "api".to_string()],
            vec![
                "backend development".to_string(),
                "api endpoint".to_string(),
            ],
            vec!["**/*.{ts,js}".to_string(), "src/routes/**/*".to_string()],
        ),
        "rust-developer" => (
            vec!["rust".to_string()],
            vec!["rust development".to_string()],
            vec!["**/*.rs".to_string(), "Cargo.toml".to_string()],
        ),
        _ => (
            vec![skill_id.to_string()],
            vec![format!("{} skill", skill_id)],
            vec![
                "src/**/*".to_string(),
                "lib/**/*".to_string(),
                "app/**/*".to_string(),
                "tests/**/*".to_string(),
            ],
        ),
    }
}

/// Compute SHA256 hash of a file
fn hash_file(file_path: &Path) -> Result<String> {
    let contents = fs::read(file_path).map_err(CatalystError::Io)?;
    let hash = Sha256::digest(&contents);
    Ok(format!("{:x}", hash))
}

/// Generate .catalyst-hashes.json for tracking file modifications
///
/// Computes SHA256 hashes for all installed skill files and stores them
/// in .catalyst-hashes.json for modification detection during updates.
///
/// # Arguments
///
/// * `target_dir` - Base directory where .claude exists
/// * `installed_skills` - List of skill IDs that were installed
pub fn generate_skill_hashes(target_dir: &Path, installed_skills: &[String]) -> Result<()> {
    let hashes_path = target_dir.join(SKILLS_DIR).join(".catalyst-hashes.json");
    let skills_dir = target_dir.join(SKILLS_DIR);

    let mut hashes: HashMap<String, String> = HashMap::new();

    for skill_id in installed_skills {
        let skill_path = skills_dir.join(skill_id);
        collect_file_hashes(&skills_dir, &skill_path, &mut hashes)?;
    }

    // Pretty-print JSON
    let content = serde_json::to_string_pretty(&hashes).map_err(CatalystError::Json)?;

    // Write atomically
    write_file_atomic(&hashes_path, &content)?;

    Ok(())
}

/// Recursively collect hashes for all files in a directory
///
/// # Arguments
///
/// * `base_dir` - Base directory for computing relative paths (e.g., .claude/skills)
/// * `current_dir` - Current directory being traversed
/// * `hashes` - HashMap to store file path -> hash mappings
fn collect_file_hashes(
    base_dir: &Path,
    current_dir: &Path,
    hashes: &mut HashMap<String, String>,
) -> Result<()> {
    if !current_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current_dir).map_err(CatalystError::Io)? {
        let entry = entry.map_err(CatalystError::Io)?;
        let path = entry.path();

        if path.is_file() {
            // Compute relative path from base_dir, with proper error handling
            let relative_path = path
                .strip_prefix(base_dir)
                .map_err(|_| {
                    CatalystError::PathTraversalDetected(format!(
                        "Path {} is not within base directory {}",
                        path.display(),
                        base_dir.display()
                    ))
                })?
                .to_string_lossy()
                .to_string();

            let hash = hash_file(&path)?;
            hashes.insert(relative_path, hash);
        } else if path.is_dir() {
            collect_file_hashes(base_dir, &path, hashes)?;
        }
    }

    Ok(())
}

/// Initialize a Claude Code project
///
/// This is the main entry point for the `catalyst init` command.
///
/// # Arguments
///
/// * `config` - Configuration for initialization
///
/// # Returns
///
/// Returns an `InitReport` with details of what was created
///
/// Write .catalyst-version file to track installation version
///
/// # Arguments
///
/// * `target_dir` - Directory where .catalyst-version should be created
///
/// # Returns
///
/// Returns Ok(()) on success
pub fn write_version_file(target_dir: &Path) -> Result<()> {
    let version_path = target_dir.join(VERSION_FILE);
    fs::write(&version_path, format!("{}\n", CATALYST_VERSION)).map_err(|e| {
        CatalystError::FileWriteFailed {
            path: version_path.clone(),
            source: e,
        }
    })?;
    Ok(())
}

/// Read .catalyst-version file
///
/// # Arguments
///
/// * `target_dir` - Directory where .catalyst-version exists
///
/// # Returns
///
/// Returns the version string on success, None if file doesn't exist
///
/// # Implementation Note
///
/// Avoids TOCTOU (Time-of-Check-Time-of-Use) race by directly attempting
/// to read the file instead of checking existence first.
pub fn read_version_file(target_dir: &Path) -> Result<Option<String>> {
    let version_path = target_dir.join(VERSION_FILE);

    match fs::read_to_string(&version_path) {
        Ok(content) => Ok(Some(content.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(CatalystError::FileReadFailed {
            path: version_path,
            source: e,
        }),
    }
}

pub fn initialize(config: &InitConfig) -> Result<InitReport> {
    // Acquire lock to prevent concurrent init
    let _lock = acquire_init_lock(&config.directory)?;

    let mut report = InitReport::new();
    let platform = Platform::detect();

    // Phase 2.1: Create directory structure
    let created_dirs = create_directory_structure(&config.directory, config.force)?;
    report.created_dirs = created_dirs;

    // Phase 2.2: Generate wrapper scripts
    let installed_hooks = generate_wrapper_scripts(
        &config.directory,
        config.install_hooks,
        config.install_tracker,
        platform,
    )?;
    report.installed_hooks = installed_hooks;

    // Phase 2.3: Create settings.json
    let settings_created = create_settings_json(
        &config.directory,
        config.install_hooks,
        config.install_tracker,
        platform,
    )?;
    report.settings_created = settings_created;

    // Phase 3.1-3.2: Install skills
    if !config.skills.is_empty() {
        let installed_skills = install_skills(&config.directory, &config.skills, config.force)?;
        report.installed_skills = installed_skills.clone();

        // Phase 3.3: Generate skill-rules.json (gracefully degrade on failure)
        if !installed_skills.is_empty() {
            if let Err(e) = generate_skill_rules(&config.directory, &installed_skills) {
                let warning = format!("⚠️  Failed to generate skill-rules.json: {}", e);
                eprintln!("{}", warning);
                report.warnings.push(warning);
            }

            // Phase 3.4: Generate .catalyst-hashes.json (gracefully degrade on failure)
            if let Err(e) = generate_skill_hashes(&config.directory, &installed_skills) {
                let warning = format!("⚠️  Failed to generate .catalyst-hashes.json: {}", e);
                eprintln!("{}", warning);
                report.warnings.push(warning);
            }
        }
    }

    // Phase 6.1: Write .catalyst-version file to track installation
    if let Err(e) = write_version_file(&config.directory) {
        let warning = format!("⚠️  Failed to write .catalyst-version: {}", e);
        eprintln!("{}", warning);
        report.warnings.push(warning);
    } else {
        report.version_file_created = true;
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // First create .claude directory (simulating Claude Code)
        fs::create_dir(target.join(".claude")).unwrap();

        let created = create_directory_structure(target, false).unwrap();

        // Should create subdirectories
        assert!(created.len() >= 4); // hooks, skills, agents, commands

        // Verify directories exist
        assert!(target.join(".claude").is_dir());
        assert!(target.join(".claude/hooks").is_dir());
        assert!(target.join(".claude/skills").is_dir());
        assert!(target.join(".claude/agents").is_dir());
        assert!(target.join(".claude/commands").is_dir());

        // Test idempotency - running again should succeed
        let created_again = create_directory_structure(target, false).unwrap();
        // Should return empty list since directories already exist
        assert_eq!(created_again.len(), 0);
    }

    #[test]
    fn test_create_directory_structure_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude directory first (simulating Claude Code)
        fs::create_dir(target.join(".claude")).unwrap();

        // Create directories first time
        create_directory_structure(target, false).unwrap();

        // Create again with force=true should succeed
        let created = create_directory_structure(target, true).unwrap();
        assert!(created.len() >= 4);
    }

    #[test]
    fn test_create_directory_structure_no_claude_dir() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Don't create .claude directory - should fail
        let result = create_directory_structure(target, false);
        assert!(result.is_err());
        match result {
            Err(CatalystError::InvalidPath(msg)) => {
                assert!(msg.contains(".claude directory not found"));
                assert!(msg.contains("Claude Code"));
            }
            _ => panic!("Expected InvalidPath error about missing .claude directory"),
        }
    }

    #[test]
    fn test_acquire_init_lock() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // First lock should succeed
        let lock1 = acquire_init_lock(target).unwrap();

        // Second lock should fail while first is held
        let lock2 = acquire_init_lock(target);
        assert!(lock2.is_err());
        match lock2 {
            Err(CatalystError::InitInProgress { pid, .. }) => {
                assert_eq!(pid, process::id());
            }
            _ => panic!("Expected InitInProgress error"),
        }

        // Drop first lock
        drop(lock1);

        // Now second lock should succeed
        let lock3 = acquire_init_lock(target);
        assert!(lock3.is_ok());
    }

    #[test]
    fn test_stale_lock_removal() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();
        let lock_file = target.join(LOCK_FILE);

        // Create a stale lock file with a non-existent PID
        fs::write(&lock_file, "999999").unwrap();

        // Should remove stale lock and succeed
        let lock = acquire_init_lock(target);
        assert!(lock.is_ok());
    }

    #[test]
    fn test_invalid_pid_lock_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();
        let lock_file = target.join(LOCK_FILE);

        // Test invalid PID 0 (reserved system PID)
        fs::write(&lock_file, "0").unwrap();
        let lock = acquire_init_lock(target);
        assert!(lock.is_ok(), "Should clean up lock file with PID 0");
        drop(lock);

        // Test invalid PID 1 (init process PID)
        fs::write(&lock_file, "1").unwrap();
        let lock = acquire_init_lock(target);
        assert!(lock.is_ok(), "Should clean up lock file with PID 1");
    }

    #[test]
    fn test_malformed_lock_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();
        let lock_file = target.join(LOCK_FILE);

        // Test non-numeric content
        fs::write(&lock_file, "not-a-number").unwrap();
        let lock = acquire_init_lock(target);
        assert!(
            lock.is_ok(),
            "Should clean up lock file with invalid content"
        );
        drop(lock);

        // Test empty lock file
        fs::write(&lock_file, "").unwrap();
        let lock = acquire_init_lock(target);
        assert!(lock.is_ok(), "Should clean up empty lock file");
        drop(lock);

        // Test lock file with whitespace
        fs::write(&lock_file, "   \n\t  ").unwrap();
        let lock = acquire_init_lock(target);
        assert!(
            lock.is_ok(),
            "Should clean up lock file with only whitespace"
        );
    }

    #[test]
    fn test_directory_exists_as_file_error() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create a file where .claude directory should be
        let claude_path = target.join(".claude");
        fs::write(&claude_path, "This is a file, not a directory").unwrap();

        // Should fail with InvalidPath error
        let result = create_directory_structure(target, false);
        assert!(result.is_err());
        match result {
            Err(CatalystError::InvalidPath(msg)) => {
                assert!(msg.contains("not a directory"));
            }
            _ => panic!("Expected InvalidPath error"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_directory_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude directory first (simulating Claude Code)
        fs::create_dir(target.join(".claude")).unwrap();

        create_directory_structure(target, false).unwrap();

        // Check permissions are 0755 on subdirectories
        let metadata = fs::metadata(target.join(".claude/hooks")).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o755);
    }

    #[test]
    fn test_generate_wrapper_scripts_unix() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude and .claude/hooks directories
        fs::create_dir(target.join(".claude")).unwrap();
        fs::create_dir(target.join(".claude/hooks")).unwrap();

        // Generate wrappers for Unix
        let installed = generate_wrapper_scripts(
            target,
            true, // install_hooks
            true, // install_tracker
            Platform::Linux,
        )
        .unwrap();

        // Should create 2 wrappers
        assert_eq!(installed.len(), 2);
        assert!(installed.contains(&"skill-activation-prompt.sh".to_string()));
        assert!(installed.contains(&"file-change-tracker.sh".to_string()));

        // Verify files exist
        let skill_wrapper = target.join(".claude/hooks/skill-activation-prompt.sh");
        let tracker_wrapper = target.join(".claude/hooks/file-change-tracker.sh");
        assert!(skill_wrapper.exists());
        assert!(tracker_wrapper.exists());

        // Verify content has binary name substituted
        let content = fs::read_to_string(&skill_wrapper).unwrap();
        assert!(content.contains("skill-activation-prompt"));
        assert!(!content.contains("{{BINARY_NAME}}"));
        assert!(content.contains("#!/bin/bash"));
    }

    #[test]
    fn test_generate_wrapper_scripts_windows() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude and .claude/hooks directories
        fs::create_dir(target.join(".claude")).unwrap();
        fs::create_dir(target.join(".claude/hooks")).unwrap();

        // Generate wrappers for Windows
        let installed = generate_wrapper_scripts(
            target,
            true,  // install_hooks
            false, // install_tracker
            Platform::Windows,
        )
        .unwrap();

        // Should create 1 wrapper
        assert_eq!(installed.len(), 1);
        assert!(installed.contains(&"skill-activation-prompt.ps1".to_string()));

        // Verify file exists
        let skill_wrapper = target.join(".claude/hooks/skill-activation-prompt.ps1");
        assert!(skill_wrapper.exists());

        // Verify content has binary name substituted
        let content = fs::read_to_string(&skill_wrapper).unwrap();
        assert!(content.contains("skill-activation-prompt.exe"));
        assert!(!content.contains("{{BINARY_NAME}}"));
        assert!(!content.contains("#!")); // No shebang in PowerShell
        assert!(content.contains("@args"));
    }

    #[cfg(unix)]
    #[test]
    fn test_wrapper_permissions_unix() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude and .claude/hooks directories
        fs::create_dir(target.join(".claude")).unwrap();
        fs::create_dir(target.join(".claude/hooks")).unwrap();

        // Generate wrappers for Unix
        generate_wrapper_scripts(
            target,
            true, // install_hooks
            true, // install_tracker
            Platform::Linux,
        )
        .unwrap();

        // Check executable permissions
        let skill_wrapper = target.join(".claude/hooks/skill-activation-prompt.sh");
        let metadata = fs::metadata(&skill_wrapper).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o111, 0o111); // Executable bit set
    }

    #[test]
    fn test_write_file_atomic() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();
        let test_file = target.join("test.txt");

        let content = "Hello, atomic write!";
        let atomic = write_file_atomic(&test_file, content).unwrap();

        // Should succeed with atomic write
        assert!(atomic);

        // File should exist and contain correct content
        assert!(test_file.exists());
        let read_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_create_settings_json() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude directory
        fs::create_dir(target.join(".claude")).unwrap();

        // Create settings.json with both hooks
        let result = create_settings_json(
            target,
            true, // install_hooks
            true, // install_tracker
            Platform::Linux,
        );
        assert!(result.is_ok());

        // Verify settings.json exists
        let settings_path = target.join(".claude/settings.json");
        assert!(settings_path.exists());

        // Parse and verify structure
        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Should have hooks array
        let hooks = settings["hooks"].as_array().unwrap();
        assert_eq!(hooks.len(), 2);

        // First hook should be UserPromptSubmit
        assert_eq!(hooks[0]["event"], "UserPromptSubmit");
        assert!(hooks[0]["script"]
            .as_str()
            .unwrap()
            .contains("skill-activation-prompt.sh"));

        // Second hook should be PostToolUse
        assert_eq!(hooks[1]["event"], "PostToolUse");
        assert!(hooks[1]["script"]
            .as_str()
            .unwrap()
            .contains("file-change-tracker.sh"));

        // PostToolUse should have matchers
        let matchers = hooks[1]["matchers"].as_array().unwrap();
        assert_eq!(matchers.len(), 3);
    }

    #[test]
    fn test_create_settings_json_windows() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude directory
        fs::create_dir(target.join(".claude")).unwrap();

        // Create settings.json for Windows
        let result = create_settings_json(
            target,
            true,  // install_hooks
            false, // no tracker
            Platform::Windows,
        );
        assert!(result.is_ok());

        // Parse and verify
        let settings_path = target.join(".claude/settings.json");
        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        let hooks = settings["hooks"].as_array().unwrap();
        assert_eq!(hooks.len(), 1); // Only skill-activation-prompt

        // Should use .ps1 extension
        assert!(hooks[0]["script"].as_str().unwrap().contains(".ps1"));
    }

    #[test]
    fn test_full_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude directory (simulating Claude Code)
        fs::create_dir(target.join(".claude")).unwrap();

        // Create full config
        let config = InitConfig {
            directory: target.to_path_buf(),
            install_hooks: true,
            install_tracker: true,
            skills: Vec::new(),
            force: false,
        };

        // Run initialize
        let report = initialize(&config).unwrap();

        // Verify report
        assert!(!report.created_dirs.is_empty());
        assert!(!report.installed_hooks.is_empty());
        assert!(report.settings_created);

        // Verify all directories exist
        assert!(target.join(".claude/hooks").is_dir());
        assert!(target.join(".claude/skills").is_dir());
        assert!(target.join(".claude/agents").is_dir());
        assert!(target.join(".claude/commands").is_dir());

        // Verify wrappers exist
        let platform = Platform::detect();
        let extension = platform.hook_extension();
        assert!(target
            .join(format!(
                ".claude/hooks/skill-activation-prompt.{}",
                extension
            ))
            .exists());
        assert!(target
            .join(format!(".claude/hooks/file-change-tracker.{}", extension))
            .exists());

        // Verify settings.json exists
        assert!(target.join(".claude/settings.json").exists());
    }

    #[test]
    fn test_install_skill() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude/skills directory
        fs::create_dir_all(target.join(".claude/skills")).unwrap();

        // Install skill-developer skill
        let result = install_skill(target, "skill-developer", false);
        assert!(result.is_ok());

        // Verify skill directory exists
        let skill_path = target.join(".claude/skills/skill-developer");
        assert!(skill_path.is_dir());

        // Verify SKILL.md exists
        assert!(skill_path.join("SKILL.md").exists());
    }

    #[test]
    fn test_install_skills_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude/skills directory
        fs::create_dir_all(target.join(".claude/skills")).unwrap();

        // Install multiple skills
        let skills = vec!["skill-developer".to_string(), "rust-developer".to_string()];
        let installed = install_skills(target, &skills, false).unwrap();

        assert_eq!(installed.len(), 2);
        assert!(target
            .join(".claude/skills/skill-developer/SKILL.md")
            .exists());
        assert!(target
            .join(".claude/skills/rust-developer/SKILL.md")
            .exists());
    }

    #[test]
    fn test_install_skill_invalid_id() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude/skills directory
        fs::create_dir_all(target.join(".claude/skills")).unwrap();

        // Try to install invalid skill
        let result = install_skill(target, "non-existent-skill", false);
        assert!(result.is_err());

        // Verify error message contains available skills
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid skill ID"));
        assert!(err_msg.contains("Available skills:"));
        assert!(err_msg.contains("skill-developer"));
    }

    #[test]
    fn test_generate_skill_rules() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create .claude/skills directory
        fs::create_dir_all(target.join(".claude/skills")).unwrap();

        // Generate skill rules
        let skills = vec!["skill-developer".to_string(), "rust-developer".to_string()];
        let result = generate_skill_rules(target, &skills);
        assert!(result.is_ok());

        // Verify skill-rules.json exists
        let rules_path = target.join(".claude/skills/skill-rules.json");
        assert!(rules_path.exists());

        // Parse and verify JSON structure
        let content = fs::read_to_string(&rules_path).unwrap();
        assert!(content.contains("// Customize pathPatterns"));
        assert!(content.contains("skill-developer"));
        assert!(content.contains("rust-developer"));

        // Verify it's valid JSON
        let json_start = content.find('{').unwrap();
        let json_content = &content[json_start..];
        let parsed: serde_json::Value = serde_json::from_str(json_content).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["skills"]["skill-developer"].is_object());
        assert!(parsed["skills"]["rust-developer"].is_object());
    }

    #[test]
    fn test_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Write test content
        fs::write(&test_file, "Hello, World!").unwrap();

        // Compute hash
        let hash = hash_file(&test_file).unwrap();

        // Verify hash is non-empty and has expected length (SHA256 = 64 hex chars)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_skill_hashes() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create skill directories with files
        fs::create_dir_all(target.join(".claude/skills/skill-developer")).unwrap();
        fs::write(
            target.join(".claude/skills/skill-developer/SKILL.md"),
            "# Test Skill",
        )
        .unwrap();

        // Generate hashes
        let skills = vec!["skill-developer".to_string()];
        let result = generate_skill_hashes(target, &skills);
        assert!(result.is_ok());

        // Verify .catalyst-hashes.json exists
        let hashes_path = target.join(".claude/skills/.catalyst-hashes.json");
        assert!(hashes_path.exists());

        // Parse and verify JSON
        let content = fs::read_to_string(&hashes_path).unwrap();
        let hashes: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(hashes.is_object());
        assert!(!hashes.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_read_version_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Write a version file
        let version_path = target.join(VERSION_FILE);
        fs::write(&version_path, "0.1.0\n").unwrap();

        // Read it back
        let result = read_version_file(target).unwrap();
        assert_eq!(result, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_read_version_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // No version file exists
        let result = read_version_file(target).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_version_file_with_error_context() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create a version file
        let version_path = target.join(VERSION_FILE);
        fs::write(&version_path, "0.1.0\n").unwrap();

        // Make it unreadable (Unix only)
        #[cfg(unix)]
        {
            fs::set_permissions(&version_path, fs::Permissions::from_mode(0o000)).unwrap();

            // Try to read it - should fail with proper error context
            let result = read_version_file(target);
            assert!(result.is_err());
            match result {
                Err(CatalystError::FileReadFailed { path, source }) => {
                    assert_eq!(path, version_path);
                    assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
                }
                _ => panic!("Expected FileReadFailed with context"),
            }

            // Clean up - restore permissions so tempdir can be deleted
            fs::set_permissions(&version_path, fs::Permissions::from_mode(0o644)).unwrap();
        }
    }

    #[test]
    fn test_write_version_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Write version file
        write_version_file(target).unwrap();

        // Verify it was written correctly
        let version_path = target.join(VERSION_FILE);
        assert!(version_path.exists());
        let content = fs::read_to_string(&version_path).unwrap();
        assert_eq!(content, format!("{}\n", CATALYST_VERSION));
    }

    #[test]
    fn test_write_version_file_with_error_context() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path();

        // Create a read-only directory (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(target, fs::Permissions::from_mode(0o555)).unwrap();

            // Try to write version file - should fail with proper error context
            let result = write_version_file(target);
            assert!(result.is_err());
            match result {
                Err(CatalystError::FileWriteFailed { path, source }) => {
                    assert_eq!(path, target.join(VERSION_FILE));
                    assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
                }
                _ => panic!("Expected FileWriteFailed with context"),
            }

            // Clean up - restore permissions so tempdir can be deleted
            fs::set_permissions(target, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

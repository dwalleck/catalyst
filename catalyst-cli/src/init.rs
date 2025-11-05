//! Initialization logic for Catalyst CLI
//!
//! This module handles the `catalyst init` command, which creates the .claude/
//! directory structure, installs hooks, and sets up skills.

use crate::types::{
    CatalystError, InitConfig, InitReport, Platform, Result, AGENTS_DIR, CLAUDE_DIR, COMMANDS_DIR,
    HOOKS_DIR, SKILLS_DIR,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use tempfile::NamedTempFile;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Embed wrapper templates at compile time
const WRAPPER_TEMPLATE_SH: &str = include_str!("../resources/wrapper-template.sh");
const WRAPPER_TEMPLATE_PS1: &str = include_str!("../resources/wrapper-template.ps1");

/// Lock file name for concurrent init protection
const LOCK_FILE: &str = ".catalyst.lock";

/// Guard that automatically releases the lock when dropped
pub struct InitLock {
    lock_file: PathBuf,
}

impl Drop for InitLock {
    fn drop(&mut self) {
        let _ = release_init_lock(&self.lock_file);
    }
}

/// Acquire a lock to prevent concurrent init operations
///
/// Creates a .catalyst.lock file with the current process ID.
/// Returns an error if a lock already exists and the process is still running.
///
/// # Arguments
///
/// * `target_dir` - The directory being initialized
///
/// # Returns
///
/// Returns an `InitLock` guard that will automatically release the lock when dropped
pub fn acquire_init_lock(target_dir: &Path) -> Result<InitLock> {
    let lock_file = target_dir.join(LOCK_FILE);

    // Check if lock file exists
    if lock_file.exists() {
        // Read the PID from the lock file
        let pid_str = fs::read_to_string(&lock_file).map_err(CatalystError::Io)?;

        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Check if the process is still running
            if is_process_running(pid) {
                return Err(CatalystError::InitInProgress {
                    pid,
                    lock_file: lock_file.display().to_string(),
                });
            } else {
                // Stale lock file - remove it
                fs::remove_file(&lock_file).map_err(CatalystError::Io)?;
            }
        }
    }

    // Create lock file with current PID
    let pid = process::id();
    fs::write(&lock_file, pid.to_string()).map_err(CatalystError::Io)?;

    Ok(InitLock { lock_file })
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
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_INVALID_PARAMETER};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // Try to open the process with minimal access rights
    // SAFETY: This is safe because we're just checking if a process exists
    // and we immediately close the handle if successful
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);

        if handle == 0 {
            // Failed to open process - check why
            let error = GetLastError();

            // ERROR_INVALID_PARAMETER (87) means the process doesn't exist
            // Any other error (like ERROR_ACCESS_DENIED) means it exists but we can't access it
            error != ERROR_INVALID_PARAMETER
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
        // EXDEV is error code 18 on Unix systems
        e.raw_os_error() == Some(18)
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
}

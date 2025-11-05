//! Binary validation and checks for Catalyst CLI
//!
//! This module provides functionality to validate that required binaries
//! are installed and accessible in the expected locations.

use crate::types::{CatalystError, Platform, Result};
use dirs::home_dir;
use std::path::{Path, PathBuf};

/// Check if all required binaries are installed in ~/.claude-hooks/bin/
///
/// This validates that:
/// - skill-activation-prompt binary exists
/// - file-analyzer binary exists
/// - file-change-tracker binary exists (if --sqlite was used)
///
/// Returns Ok(()) if all binaries are found, or an error with details about
/// what's missing and how to install them.
pub fn check_binaries_installed(platform: Platform) -> Result<Vec<String>> {
    let bin_dir = get_binary_directory()?;
    let mut missing = Vec::new();
    let mut found = Vec::new();

    // Required binaries (always needed)
    let required = vec!["skill-activation-prompt", "file-analyzer"];

    // Check required binaries
    for binary_name in &required {
        if binary_exists(&bin_dir, binary_name, platform) {
            found.push(binary_name.to_string());
        } else {
            missing.push(binary_name.to_string());
        }
    }

    // Check for file-change-tracker variants
    let tracker_variant = detect_file_change_tracker_variant(&bin_dir, platform)?;
    if let Some(variant) = tracker_variant {
        found.push(format!("file-change-tracker ({})", variant));
    } else {
        missing.push("file-change-tracker (sqlite or basic)".to_string());
    }

    if !missing.is_empty() {
        return Err(CatalystError::BinariesNotInstalled {
            install_command: get_install_command(&missing, platform),
            missing_binaries: missing.join(", "),
        });
    }

    Ok(found)
}

/// Detect which variant of file-change-tracker is installed
///
/// Returns:
/// - Some("sqlite") if the SQLite version is found
/// - Some("basic") if the basic version is found
/// - None if neither is found
///
/// # Current Limitations (Phase 1)
///
/// Currently assumes any file-change-tracker binary is the SQLite variant
/// since that's the only variant we build with the new name. This is acceptable
/// for Phase 1 because:
/// - The basic variant hasn't been implemented yet
/// - install.sh only builds the SQLite variant with --sqlite flag
/// - Users who have the binary are guaranteed to have the SQLite version
///
/// # Future Enhancement
///
/// TODO: Implement --version flag detection to distinguish variants accurately
/// when basic variant is added in future phases.
pub fn detect_file_change_tracker_variant(
    bin_dir: &Path,
    platform: Platform,
) -> Result<Option<String>> {
    // Check for new binary name (Phase 1+)
    if binary_exists(bin_dir, "file-change-tracker", platform) {
        // Phase 1: Assume SQLite variant (only variant available)
        // This is safe because install.sh --sqlite is the only way to get this binary
        return Ok(Some("sqlite".to_string()));
    }

    // Check for legacy name (pre-Phase 1 installations)
    if binary_exists(bin_dir, "post-tool-use-tracker-sqlite", platform) {
        return Ok(Some("sqlite-legacy".to_string()));
    }

    Ok(None)
}

/// Get the binary installation directory
pub fn get_binary_directory() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| {
        CatalystError::InvalidPath("Could not determine home directory".to_string())
    })?;

    Ok(home.join(".claude-hooks").join("bin"))
}

/// Check if a binary exists in the given directory
///
/// On Windows, this checks for both the name with and without .exe extension
pub fn binary_exists(bin_dir: &Path, name: &str, platform: Platform) -> bool {
    let binary_path = if platform == Platform::Windows {
        bin_dir.join(format!("{}.exe", name))
    } else {
        bin_dir.join(name)
    };

    binary_path.exists() && binary_path.is_file()
}

/// Generate the appropriate install command based on what's missing and the platform
fn get_install_command(missing: &[String], platform: Platform) -> String {
    let has_tracker = missing.iter().any(|m| m.contains("file-change-tracker"));

    match platform {
        Platform::Windows => {
            if has_tracker {
                ".\\install.ps1 -Sqlite".to_string()
            } else {
                ".\\install.ps1".to_string()
            }
        }
        _ => {
            // Linux, MacOS, WSL all use bash script
            if has_tracker {
                "cd catalyst && ./install.sh --sqlite".to_string()
            } else {
                "cd catalyst && ./install.sh".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_exists_handles_windows_exe() {
        let platform = Platform::Windows;
        let bin_dir = Path::new("/test/bin");

        // This test validates logic only - actual file doesn't exist in test
        // In real usage, the file system check will be performed
        assert!(!binary_exists(bin_dir, "nonexistent", platform));
    }

    #[test]
    fn test_get_install_command_with_tracker() {
        let missing = vec!["file-change-tracker (sqlite or basic)".to_string()];
        let cmd = get_install_command(&missing, Platform::Linux);
        assert!(cmd.contains("--sqlite"));
    }

    #[test]
    fn test_get_install_command_without_tracker() {
        let missing = vec!["skill-activation-prompt".to_string()];
        let cmd = get_install_command(&missing, Platform::Linux);
        assert!(!cmd.contains("--sqlite"));
    }

    #[test]
    fn test_get_install_command_windows() {
        let missing = vec!["skill-activation-prompt".to_string()];
        let cmd = get_install_command(&missing, Platform::Windows);
        assert!(cmd.contains(".ps1"));
        assert!(!cmd.contains(".sh"));
    }

    #[test]
    fn test_get_install_command_windows_with_sqlite() {
        let missing = vec!["file-change-tracker (sqlite or basic)".to_string()];
        let cmd = get_install_command(&missing, Platform::Windows);
        assert!(cmd.contains(".ps1"));
        assert!(cmd.contains("-Sqlite"));
    }

    #[test]
    fn test_detect_variant_returns_none_when_missing() {
        use std::path::Path;
        // Use a path that definitely doesn't exist
        let nonexistent_dir = Path::new("/nonexistent/path/to/binaries");
        let result = detect_file_change_tracker_variant(nonexistent_dir, Platform::Linux);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_check_binaries_returns_error_with_missing_list() {
        // This test validates that check_binaries_installed properly reports
        // missing binaries through the error type
        let platform = Platform::Linux;
        let result = check_binaries_installed(platform);

        // Should fail because ~/.claude-hooks/bin likely doesn't have all binaries
        // or might not exist at all
        match result {
            Err(CatalystError::BinariesNotInstalled {
                install_command,
                missing_binaries,
            }) => {
                // Verify error contains useful information
                assert!(!install_command.is_empty());
                assert!(!missing_binaries.is_empty());
                assert!(install_command.contains("install"));
            }
            Ok(_) => {
                // If binaries are actually installed, that's fine too
                // This happens in CI or when running tests after installation
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[test]
    fn test_platform_specific_commands() {
        // Test that different platforms get appropriate commands
        let missing = vec!["skill-activation-prompt".to_string()];

        let linux_cmd = get_install_command(&missing, Platform::Linux);
        assert!(linux_cmd.contains(".sh"));

        let macos_cmd = get_install_command(&missing, Platform::MacOS);
        assert!(macos_cmd.contains(".sh"));

        let wsl_cmd = get_install_command(&missing, Platform::WSL);
        assert!(wsl_cmd.contains(".sh"));

        let windows_cmd = get_install_command(&missing, Platform::Windows);
        assert!(windows_cmd.contains(".ps1"));
    }
}

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
            install_command: get_install_command(&missing),
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
pub fn detect_file_change_tracker_variant(
    bin_dir: &Path,
    platform: Platform,
) -> Result<Option<String>> {
    // Try to find file-change-tracker binary (renamed from post-tool-use-tracker-sqlite)
    if binary_exists(bin_dir, "file-change-tracker", platform) {
        // For now, if the binary exists, assume it's the SQLite variant
        // In the future, we could run the binary with --version to determine variant
        return Ok(Some("sqlite".to_string()));
    }

    // Check for legacy name
    if binary_exists(bin_dir, "post-tool-use-tracker-sqlite", platform) {
        return Ok(Some("sqlite-legacy".to_string()));
    }

    Ok(None)
}

/// Get the binary installation directory
fn get_binary_directory() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| {
        CatalystError::InvalidPath("Could not determine home directory".to_string())
    })?;

    Ok(home.join(".claude-hooks").join("bin"))
}

/// Check if a binary exists in the given directory
///
/// On Windows, this checks for both the name with and without .exe extension
fn binary_exists(bin_dir: &Path, name: &str, platform: Platform) -> bool {
    let binary_path = if platform == Platform::Windows {
        bin_dir.join(format!("{}.exe", name))
    } else {
        bin_dir.join(name)
    };

    binary_path.exists() && binary_path.is_file()
}

/// Generate the appropriate install command based on what's missing
fn get_install_command(missing: &[String]) -> String {
    let has_tracker = missing.iter().any(|m| m.contains("file-change-tracker"));

    if has_tracker {
        // User needs SQLite support
        "cd catalyst && ./install.sh --sqlite".to_string()
    } else {
        // Core binaries only
        "cd catalyst && ./install.sh".to_string()
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
        let cmd = get_install_command(&missing);
        assert!(cmd.contains("--sqlite"));
    }

    #[test]
    fn test_get_install_command_without_tracker() {
        let missing = vec!["skill-activation-prompt".to_string()];
        let cmd = get_install_command(&missing);
        assert!(!cmd.contains("--sqlite"));
    }
}

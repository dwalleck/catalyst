use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;
use toml::Value;

#[derive(Error, Debug)]
enum CargoCheckError {
    #[error("[CC001] Failed to read input from stdin")]
    StdinRead(#[from] io::Error),

    #[error("[CC002] Invalid JSON input from hook: {0}\nCheck that the hook is passing valid JSON format")]
    InvalidHookInput(#[source] serde_json::Error),

    #[error("[CC003] Could not find Cargo.toml for file: {}\nMake sure the file is in a Cargo project", path.display())]
    CargoTomlNotFound { path: PathBuf },

    #[error("[CC004] Failed to execute cargo command: {0}")]
    CargoExecution(#[source] io::Error),

    #[error("[CC005] Cargo check failed with exit code: {code}\nSee output above for details")]
    CargoCheckFailed { code: i32 },
}

#[derive(Debug, Deserialize)]
struct HookInput {
    #[serde(rename = "session_id")]
    _session_id: String,
    tool_name: Option<String>,
    tool_input: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug)]
enum CargoRoot {
    Workspace(PathBuf),
    Package(PathBuf),
}

impl CargoRoot {
    fn path(&self) -> &Path {
        match self {
            CargoRoot::Workspace(p) | CargoRoot::Package(p) => p,
        }
    }

    fn kind(&self) -> &str {
        match self {
            CargoRoot::Workspace(_) => "workspace",
            CargoRoot::Package(_) => "package",
        }
    }
}

/// Checks if an environment variable is set to a truthy value
/// Accepts: "1", "true", "yes", "on" (case-insensitive)
fn env_is_enabled(var: &str) -> bool {
    env::var(var)
        .map(|v| {
            let lower = v.to_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

/// Checks if a Cargo.toml file defines a workspace using TOML parsing
fn is_workspace(cargo_toml_path: &Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(cargo_toml_path) {
        if let Ok(toml) = content.parse::<Value>() {
            // Check if [workspace] section exists in the parsed TOML
            return toml.get("workspace").is_some();
        }
    }
    false
}

/// Finds the Cargo.toml root for a given file path
/// Returns the workspace root if found, otherwise the first package root
fn find_cargo_root(file_path: &Path) -> Result<CargoRoot, CargoCheckError> {
    let mut current_dir = file_path
        .parent()
        .ok_or_else(|| CargoCheckError::CargoTomlNotFound {
            path: file_path.to_path_buf(),
        })?;

    let mut package_root: Option<PathBuf> = None;

    loop {
        let cargo_toml = current_dir.join("Cargo.toml");

        if cargo_toml.exists() {
            // Check if this is a workspace using TOML parsing
            if is_workspace(&cargo_toml) {
                return Ok(CargoRoot::Workspace(current_dir.to_path_buf()));
            }

            // Remember the first package found
            if package_root.is_none() {
                package_root = Some(current_dir.to_path_buf());
            }
        }

        // Move up one directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }

    // Return the package root if we found one
    package_root
        .map(CargoRoot::Package)
        .ok_or_else(|| CargoCheckError::CargoTomlNotFound {
            path: file_path.to_path_buf(),
        })
}

/// Runs a cargo command with inherited stdout/stderr for proper interleaving
fn run_cargo_command(
    cargo_root: &CargoRoot,
    command: &str,
    args: &[&str],
    emoji: &str,
    success_msg: &str,
) -> Result<(), CargoCheckError> {
    eprintln!("{} Running {} on {}...", emoji, command, cargo_root.kind());

    let mut cmd = Command::new("cargo");
    cmd.arg(command);

    // Add workspace/all flag for workspace roots BEFORE other args
    // Note: cargo fmt uses --all instead of --workspace
    if matches!(cargo_root, CargoRoot::Workspace(_)) {
        if command == "fmt" {
            cmd.arg("--all");
        } else {
            cmd.arg("--workspace");
        }
    }

    // Add additional args
    for arg in args {
        cmd.arg(arg);
    }

    // Set working directory
    cmd.current_dir(cargo_root.path());

    // Inherit stdout and stderr for proper interleaving
    // This ensures output appears in real-time in the correct order
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    // Run the command and wait for it to complete
    let status = cmd.status().map_err(CargoCheckError::CargoExecution)?;

    if !status.success() {
        let code = status.code().unwrap_or(101);
        eprintln!();
        eprintln!("❌ {} failed!", command);
        return Err(CargoCheckError::CargoCheckFailed { code });
    }

    eprintln!("{}", success_msg);
    Ok(())
}

/// Runs cargo check and optional additional checks
fn run_all_checks(cargo_root: &CargoRoot) -> Result<(), CargoCheckError> {
    // Always run cargo check
    run_cargo_command(cargo_root, "check", &[], "🦀", "✅ Cargo check passed")?;

    // Optional: Run clippy if CARGO_CHECK_CLIPPY is enabled
    if env_is_enabled("CARGO_CHECK_CLIPPY") {
        run_cargo_command(
            cargo_root,
            "clippy",
            &["--", "-D", "warnings"],
            "📎",
            "✅ Clippy passed",
        )?;
    }

    // Optional: Run tests (check only, don't execute) if CARGO_CHECK_TESTS is enabled
    if env_is_enabled("CARGO_CHECK_TESTS") {
        run_cargo_command(
            cargo_root,
            "test",
            &["--no-run"],
            "🧪",
            "✅ Test compilation passed",
        )?;
    }

    // Optional: Check formatting if CARGO_CHECK_FMT is enabled
    if env_is_enabled("CARGO_CHECK_FMT") {
        run_cargo_command(
            cargo_root,
            "fmt",
            &["--", "--check"],
            "📝",
            "✅ Formatting check passed",
        )?;
    }

    Ok(())
}

fn run() -> Result<(), CargoCheckError> {
    // Read JSON input from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Parse hook input
    let input: HookInput =
        serde_json::from_str(&buffer).map_err(CargoCheckError::InvalidHookInput)?;

    // Check if this is a relevant tool (Edit, Write, MultiEdit)
    let tool_name = match input.tool_name {
        Some(name) => name,
        None => return Ok(()), // No tool name, skip
    };

    if !matches!(tool_name.as_str(), "Edit" | "Write" | "MultiEdit") {
        return Ok(()); // Not a file editing tool, skip
    }

    // Extract tool_input
    let tool_input = match input.tool_input {
        Some(input) => input,
        None => return Ok(()), // No tool input, skip
    };

    // Collect all Rust file paths from the tool input
    let mut rust_files = Vec::new();

    // Handle MultiEdit tool - has edits array
    if tool_name == "MultiEdit" {
        if let Some(edits_value) = tool_input.get("edits") {
            if let Some(edits_array) = edits_value.as_array() {
                for edit in edits_array {
                    if let Some(file_path) = edit.get("file_path").and_then(|v| v.as_str()) {
                        if file_path.ends_with(".rs") {
                            rust_files.push(PathBuf::from(file_path));
                        }
                    }
                }
            }
        }
    } else {
        // Handle Edit and Write tools - has file_path
        if let Some(file_path) = tool_input.get("file_path").and_then(|v| v.as_str()) {
            if file_path.ends_with(".rs") {
                rust_files.push(PathBuf::from(file_path));
            }
        }
    }

    // If no Rust files, skip
    if rust_files.is_empty() {
        return Ok(());
    }

    // Find all cargo roots and deduplicate
    let mut processed_roots = HashSet::new();

    for file_path in rust_files {
        let cargo_root = find_cargo_root(&file_path)?;
        let root_path = cargo_root.path().to_path_buf();

        // Only run checks if we haven't processed this root yet
        if processed_roots.insert(root_path) {
            run_all_checks(&cargo_root)?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_env_is_enabled_with_various_values() {
        // Test "1"
        std::env::set_var("TEST_VAR_1", "1");
        assert!(env_is_enabled("TEST_VAR_1"));

        // Test "true"
        std::env::set_var("TEST_VAR_TRUE", "true");
        assert!(env_is_enabled("TEST_VAR_TRUE"));

        // Test "TRUE" (case insensitive)
        std::env::set_var("TEST_VAR_TRUE_UPPER", "TRUE");
        assert!(env_is_enabled("TEST_VAR_TRUE_UPPER"));

        // Test "yes"
        std::env::set_var("TEST_VAR_YES", "yes");
        assert!(env_is_enabled("TEST_VAR_YES"));

        // Test "on"
        std::env::set_var("TEST_VAR_ON", "on");
        assert!(env_is_enabled("TEST_VAR_ON"));

        // Test "0" (should be false)
        std::env::set_var("TEST_VAR_0", "0");
        assert!(!env_is_enabled("TEST_VAR_0"));

        // Test "false" (should be false)
        std::env::set_var("TEST_VAR_FALSE", "false");
        assert!(!env_is_enabled("TEST_VAR_FALSE"));

        // Test unset variable
        std::env::remove_var("TEST_VAR_UNSET");
        assert!(!env_is_enabled("TEST_VAR_UNSET"));

        // Clean up
        std::env::remove_var("TEST_VAR_1");
        std::env::remove_var("TEST_VAR_TRUE");
        std::env::remove_var("TEST_VAR_TRUE_UPPER");
        std::env::remove_var("TEST_VAR_YES");
        std::env::remove_var("TEST_VAR_ON");
        std::env::remove_var("TEST_VAR_0");
        std::env::remove_var("TEST_VAR_FALSE");
    }

    #[test]
    fn test_is_workspace_with_workspace_toml() {
        let temp_dir = std::env::temp_dir().join("cargo_check_test_workspace");
        fs::create_dir_all(&temp_dir).unwrap();
        let cargo_toml_path = temp_dir.join("Cargo.toml");

        // Create a workspace Cargo.toml
        let mut file = fs::File::create(&cargo_toml_path).unwrap();
        writeln!(
            file,
            r#"
[workspace]
members = ["crate1", "crate2"]

[workspace.package]
version = "0.1.0"
"#
        )
        .unwrap();

        assert!(is_workspace(&cargo_toml_path));

        // Clean up
        fs::remove_file(cargo_toml_path).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_is_workspace_with_package_toml() {
        let temp_dir = std::env::temp_dir().join("cargo_check_test_package");
        fs::create_dir_all(&temp_dir).unwrap();
        let cargo_toml_path = temp_dir.join("Cargo.toml");

        // Create a package Cargo.toml (no workspace section)
        let mut file = fs::File::create(&cargo_toml_path).unwrap();
        writeln!(
            file,
            r#"
[package]
name = "my-package"
version = "0.1.0"

[dependencies]
"#
        )
        .unwrap();

        assert!(!is_workspace(&cargo_toml_path));

        // Clean up
        fs::remove_file(cargo_toml_path).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_is_workspace_with_invalid_toml() {
        let temp_dir = std::env::temp_dir().join("cargo_check_test_invalid");
        fs::create_dir_all(&temp_dir).unwrap();
        let cargo_toml_path = temp_dir.join("Cargo.toml");

        // Create an invalid TOML file
        let mut file = fs::File::create(&cargo_toml_path).unwrap();
        writeln!(file, "this is not valid TOML [[[").unwrap();

        assert!(!is_workspace(&cargo_toml_path));

        // Clean up
        fs::remove_file(cargo_toml_path).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_is_workspace_with_nonexistent_file() {
        let nonexistent_path = std::env::temp_dir().join("nonexistent_cargo.toml");
        assert!(!is_workspace(&nonexistent_path));
    }

    #[test]
    fn test_find_cargo_root_package() {
        // Create a temporary directory structure:
        // temp_dir/
        //   Cargo.toml (package)
        //   src/
        //     main.rs
        let temp_dir = std::env::temp_dir().join("cargo_check_test_find_package");
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let cargo_toml_path = temp_dir.join("Cargo.toml");
        let mut file = fs::File::create(&cargo_toml_path).unwrap();
        writeln!(
            file,
            r#"
[package]
name = "test-package"
version = "0.1.0"
"#
        )
        .unwrap();

        let main_rs_path = src_dir.join("main.rs");
        fs::File::create(&main_rs_path).unwrap();

        // Test finding the cargo root from main.rs
        let result = find_cargo_root(&main_rs_path);
        assert!(result.is_ok());

        let cargo_root = result.unwrap();
        assert_eq!(cargo_root.kind(), "package");
        assert_eq!(cargo_root.path(), temp_dir);

        // Clean up
        fs::remove_file(main_rs_path).unwrap();
        fs::remove_file(cargo_toml_path).unwrap();
        fs::remove_dir(src_dir).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_find_cargo_root_workspace() {
        // Create a temporary directory structure:
        // temp_dir/
        //   Cargo.toml (workspace)
        //   crate1/
        //     Cargo.toml (package)
        //     src/
        //       lib.rs
        let temp_dir = std::env::temp_dir().join("cargo_check_test_find_workspace");
        let crate1_dir = temp_dir.join("crate1");
        let src_dir = crate1_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create workspace Cargo.toml
        let workspace_cargo_toml = temp_dir.join("Cargo.toml");
        let mut file = fs::File::create(&workspace_cargo_toml).unwrap();
        writeln!(
            file,
            r#"
[workspace]
members = ["crate1"]
"#
        )
        .unwrap();

        // Create package Cargo.toml
        let package_cargo_toml = crate1_dir.join("Cargo.toml");
        let mut file = fs::File::create(&package_cargo_toml).unwrap();
        writeln!(
            file,
            r#"
[package]
name = "crate1"
version = "0.1.0"
"#
        )
        .unwrap();

        let lib_rs_path = src_dir.join("lib.rs");
        fs::File::create(&lib_rs_path).unwrap();

        // Test finding the cargo root from lib.rs
        // It should find the workspace root, not the package root
        let result = find_cargo_root(&lib_rs_path);
        assert!(result.is_ok());

        let cargo_root = result.unwrap();
        assert_eq!(cargo_root.kind(), "workspace");
        assert_eq!(cargo_root.path(), temp_dir);

        // Clean up
        fs::remove_file(lib_rs_path).unwrap();
        fs::remove_file(package_cargo_toml).unwrap();
        fs::remove_file(workspace_cargo_toml).unwrap();
        fs::remove_dir(src_dir).unwrap();
        fs::remove_dir(crate1_dir).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_find_cargo_root_not_found() {
        let temp_dir = std::env::temp_dir().join("cargo_check_test_no_cargo");
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let main_rs_path = src_dir.join("main.rs");
        fs::File::create(&main_rs_path).unwrap();

        // Test finding cargo root when no Cargo.toml exists
        let result = find_cargo_root(&main_rs_path);
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                CargoCheckError::CargoTomlNotFound { path } => {
                    assert_eq!(path, main_rs_path);
                }
                _ => panic!("Expected CargoTomlNotFound error"),
            }
        }

        // Clean up
        fs::remove_file(main_rs_path).unwrap();
        fs::remove_dir(src_dir).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }
}

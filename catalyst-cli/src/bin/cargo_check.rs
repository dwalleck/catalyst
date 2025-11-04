// Cargo check hook - automatically runs cargo check when editing Rust files
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, BufRead, BufReader, Read};
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
}

#[derive(Debug, Deserialize)]
struct HookInput {
    #[serde(rename = "session_id")]
    _session_id: String,
    tool_name: Option<String>,
    tool_input: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct HookResponse {
    decision: String,
    reasoning: String,
    #[serde(rename = "additionalContext")]
    additional_context: String,
}

#[derive(Debug)]
struct CommandResult {
    success: bool,
    output: String,
    exit_code: i32,
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
                let root_path = if current_dir.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    current_dir.to_path_buf()
                };
                return Ok(CargoRoot::Workspace(root_path));
            }

            // Remember the first package found
            if package_root.is_none() {
                let root_path = if current_dir.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    current_dir.to_path_buf()
                };
                package_root = Some(root_path);
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

/// Runs a cargo command and captures output
fn run_cargo_command(
    cargo_root: &CargoRoot,
    command: &str,
    args: &[&str],
    emoji: &str,
    success_msg: &str,
) -> Result<CommandResult, CargoCheckError> {
    let quiet = env_is_enabled("CARGO_CHECK_QUIET");
    let mut output_buffer = String::new();

    if !quiet {
        let msg = format!(
            "{} Running {} on {}...\n",
            emoji,
            command,
            cargo_root.kind()
        );
        output_buffer.push_str(&msg);
    }

    // Just use "cargo" - the wrapper script should ensure PATH is set correctly
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

    // In quiet mode, use -q to suppress "Checking..." messages
    // Cargo will still output errors even with -q
    if quiet && command != "fmt" {
        cmd.arg("-q");
    }

    // Add additional args
    for arg in args {
        cmd.arg(arg);
    }

    // Set working directory
    cmd.current_dir(cargo_root.path());

    // Capture stdout and stderr
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    // Spawn the command
    let mut child = cmd.spawn().map_err(CargoCheckError::CargoExecution)?;

    // Capture output streams
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "Failed to capture stdout"))
        .map_err(CargoCheckError::CargoExecution)?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "Failed to capture stderr"))
        .map_err(CargoCheckError::CargoExecution)?;

    // Capture output to buffer
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Process stdout
    for line in stdout_reader.lines().map_while(Result::ok) {
        if !quiet || line.contains("error") || line.contains("warning") {
            output_buffer.push_str(&line);
            output_buffer.push('\n');
        }
    }

    // Process stderr (always capture, even in quiet mode)
    for line in stderr_reader.lines().map_while(Result::ok) {
        output_buffer.push_str(&line);
        output_buffer.push('\n');
    }

    // Wait for the command to complete
    let status = child.wait().map_err(CargoCheckError::CargoExecution)?;
    let exit_code = status.code().unwrap_or(101);

    if !status.success() {
        // Add failure summary to output
        output_buffer.push('\n');
        output_buffer.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        output_buffer.push_str(&format!(
            "❌ Cargo {} failed with exit code {}\n",
            command, exit_code
        ));
        output_buffer.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        return Ok(CommandResult {
            success: false,
            output: output_buffer,
            exit_code,
        });
    }

    if !quiet {
        output_buffer.push_str(success_msg);
        output_buffer.push('\n');
    }

    Ok(CommandResult {
        success: true,
        output: output_buffer,
        exit_code: 0,
    })
}

/// Runs cargo check and optional additional checks
/// Returns accumulated output and whether all checks passed
fn run_all_checks(cargo_root: &CargoRoot) -> Result<CommandResult, CargoCheckError> {
    let mut accumulated_output = String::new();
    let mut all_success = true;
    let mut final_exit_code = 0;

    // Always run cargo check
    let result = run_cargo_command(cargo_root, "check", &[], "🦀", "✅ Cargo check passed")?;
    accumulated_output.push_str(&result.output);
    if !result.success {
        all_success = false;
        final_exit_code = result.exit_code;
    }

    // Optional: Run clippy if CARGO_CHECK_CLIPPY is enabled
    if env_is_enabled("CARGO_CHECK_CLIPPY") {
        let result = run_cargo_command(
            cargo_root,
            "clippy",
            &["--", "-D", "warnings"],
            "📎",
            "✅ Clippy passed",
        )?;
        accumulated_output.push_str(&result.output);
        if !result.success {
            all_success = false;
            final_exit_code = result.exit_code;
        }
    }

    // Optional: Run tests (check only, don't execute) if CARGO_CHECK_TESTS is enabled
    if env_is_enabled("CARGO_CHECK_TESTS") {
        let result = run_cargo_command(
            cargo_root,
            "test",
            &["--no-run"],
            "🧪",
            "✅ Test compilation passed",
        )?;
        accumulated_output.push_str(&result.output);
        if !result.success {
            all_success = false;
            final_exit_code = result.exit_code;
        }
    }

    // Optional: Check formatting if CARGO_CHECK_FMT is enabled
    if env_is_enabled("CARGO_CHECK_FMT") {
        let result = run_cargo_command(
            cargo_root,
            "fmt",
            &["--", "--check"],
            "📝",
            "✅ Formatting check passed",
        )?;
        accumulated_output.push_str(&result.output);
        if !result.success {
            all_success = false;
            final_exit_code = result.exit_code;
        }
    }

    Ok(CommandResult {
        success: all_success,
        output: accumulated_output,
        exit_code: final_exit_code,
    })
}

fn run() -> Result<Option<HookResponse>, CargoCheckError> {
    // Read JSON input from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Parse hook input
    let input: HookInput =
        serde_json::from_str(&buffer).map_err(CargoCheckError::InvalidHookInput)?;

    // Check if this is a relevant tool (Edit, Write, MultiEdit)
    let tool_name = match input.tool_name {
        Some(name) => name,
        None => return Ok(None), // No tool name, skip
    };

    if !matches!(tool_name.as_str(), "Edit" | "Write" | "MultiEdit") {
        return Ok(None); // Not a file editing tool, skip
    }

    // Extract tool_input
    let tool_input = match input.tool_input {
        Some(input) => input,
        None => return Ok(None), // No tool input, skip
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
        return Ok(None);
    }

    // Find all cargo roots and deduplicate
    let mut processed_roots = HashSet::new();
    let mut accumulated_output = String::new();
    let mut any_failed = false;

    for file_path in rust_files {
        let cargo_root = find_cargo_root(&file_path)?;
        let root_path = cargo_root.path().to_path_buf();

        // Only run checks if we haven't processed this root yet
        if processed_roots.insert(root_path) {
            let result = run_all_checks(&cargo_root)?;
            accumulated_output.push_str(&result.output);

            if !result.success {
                any_failed = true;
            }
        }
    }

    // If any checks failed, return a block response
    if any_failed {
        Ok(Some(HookResponse {
            decision: "block".to_string(),
            reasoning: "Rust compilation checks failed - code contains errors that must be fixed before proceeding".to_string(),
            additional_context: accumulated_output,
        }))
    } else {
        // All checks passed - no need to output anything
        Ok(None)
    }
}

fn main() {
    match run() {
        Ok(Some(response)) => {
            // Output JSON response to stdout
            if let Ok(json) = serde_json::to_string_pretty(&response) {
                println!("{}", json);
            } else {
                eprintln!("Failed to serialize hook response");
            }
            // Exit with 0 - the JSON decision field indicates the block
            std::process::exit(0);
        }
        Ok(None) => {
            // Success, no output needed
            std::process::exit(0);
        }
        Err(e) => {
            // Hook execution error (not cargo failure) - output as block with error
            let response = HookResponse {
                decision: "block".to_string(),
                reasoning: format!("Cargo check hook error: {}", e),
                additional_context: "The cargo check hook encountered an internal error. Please check your Rust project configuration.".to_string(),
            };

            if let Ok(json) = serde_json::to_string_pretty(&response) {
                println!("{}", json);
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(0);
        }
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
    fn test_find_cargo_root_relative_path_at_workspace_root() {
        // Regression test for empty path bug when using relative paths
        // This tests the case where a relative path walks up to the workspace root
        let temp_dir = std::env::temp_dir().join("cargo_check_test_relative_workspace");
        let crate1_dir = temp_dir.join("crate1");
        let src_dir = crate1_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create workspace Cargo.toml in temp_dir
        let workspace_cargo = temp_dir.join("Cargo.toml");
        let mut file = fs::File::create(&workspace_cargo).unwrap();
        writeln!(file, "[workspace]\nmembers = [\"crate1\"]").unwrap();

        // Create package Cargo.toml in crate1
        let package_cargo = crate1_dir.join("Cargo.toml");
        let mut file = fs::File::create(&package_cargo).unwrap();
        writeln!(file, "[package]\nname = \"crate1\"\nversion = \"0.1.0\"").unwrap();

        let lib_rs = src_dir.join("lib.rs");
        fs::File::create(&lib_rs).unwrap();

        // Change to temp_dir and use a relative path
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Use relative path from workspace root
        let relative_path = PathBuf::from("crate1/src/lib.rs");
        let result = find_cargo_root(&relative_path);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let cargo_root = result.unwrap();

        // Should find workspace root
        assert_eq!(cargo_root.kind(), "workspace");

        // The path should be "." not empty string
        let path = cargo_root.path();
        assert!(!path.as_os_str().is_empty(), "Path should not be empty");
        assert!(path == PathBuf::from(".") || path.is_absolute());

        // Clean up
        fs::remove_file(lib_rs).unwrap();
        fs::remove_file(package_cargo).unwrap();
        fs::remove_file(workspace_cargo).unwrap();
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

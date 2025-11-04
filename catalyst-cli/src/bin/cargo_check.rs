use serde::Deserialize;
use std::collections::HashMap;
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
        let code = status.code().unwrap_or(-1);
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

    // Extract file_path from tool_input
    let tool_input = match input.tool_input {
        Some(input) => input,
        None => return Ok(()), // No tool input, skip
    };

    let file_path_value = match tool_input.get("file_path") {
        Some(value) => value,
        None => return Ok(()), // No file_path, skip
    };

    let file_path_str = match file_path_value.as_str() {
        Some(s) => s,
        None => return Ok(()), // file_path is not a string, skip
    };

    // Check if this is a Rust file
    if !file_path_str.ends_with(".rs") {
        return Ok(()); // Not a Rust file, skip
    }

    let file_path = PathBuf::from(file_path_str);

    // Find the Cargo.toml root
    let cargo_root = find_cargo_root(&file_path)?;

    // Run cargo check and optional additional checks
    run_all_checks(&cargo_root)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

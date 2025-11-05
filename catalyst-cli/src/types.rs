// Core data structures for the Catalyst CLI
// Phase 0.1: Complete type definitions for all commands

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum CatalystError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Binary not found: {0}")]
    BinaryNotFound(String),

    #[error("Required binaries not installed. Please run: {install_command}\n\nMissing: {missing_binaries}")]
    BinariesNotInstalled {
        install_command: String,
        missing_binaries: String,
    },

    #[error("Hook installation failed: {0}")]
    HookInstallationFailed(String),

    #[error("Skill installation failed: {0}")]
    SkillInstallationFailed(String),

    #[error("Initialization already in progress (PID {pid}). If this is stale, remove the lock file at: {lock_file}")]
    InitInProgress { pid: u32, lock_file: String },

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Hash mismatch: {0}")]
    HashMismatch(String),

    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("Path traversal detected: {0}")]
    PathTraversalDetected(String),
}

pub type Result<T> = std::result::Result<T, CatalystError>;

// ============================================================================
// Platform Detection
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    WSL, // Windows Subsystem for Linux
}

impl Platform {
    /// Detects the current platform
    pub fn detect() -> Self {
        // Check for WSL first (via WSL_DISTRO_NAME environment variable)
        if std::env::var("WSL_DISTRO_NAME").is_ok() {
            return Platform::WSL;
        }

        // Then check for native platforms
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        }
    }

    /// Returns the appropriate hook file extension for the platform
    pub fn hook_extension(&self) -> &'static str {
        match self {
            Platform::Linux | Platform::MacOS | Platform::WSL => "sh",
            Platform::Windows => "ps1",
        }
    }

    /// Returns the shebang line for hook scripts
    pub fn hook_shebang(&self) -> Option<&'static str> {
        match self {
            Platform::Linux | Platform::MacOS | Platform::WSL => Some("#!/bin/bash"),
            Platform::Windows => None, // PowerShell doesn't use shebangs
        }
    }
}

// ============================================================================
// Init Command Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    /// Whether to install skill auto-activation hooks
    pub install_hooks: bool,

    /// Whether to install the file-change-tracker hook
    pub install_tracker: bool,

    /// List of skills to install (e.g., ["skill-developer", "backend-dev-guidelines"])
    pub skills: Vec<String>,

    /// Force installation even if .claude directory already exists
    pub force: bool,

    /// Directory to initialize (defaults to current directory)
    pub directory: PathBuf,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            install_hooks: true,
            install_tracker: true,
            skills: Vec::new(),
            force: false,
            directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitReport {
    /// Directories that were created
    pub created_dirs: Vec<String>,

    /// Hooks that were installed (file paths)
    pub installed_hooks: Vec<String>,

    /// Skills that were installed (skill names)
    pub installed_skills: Vec<String>,

    /// Whether settings.json was created
    pub settings_created: bool,

    /// Whether .catalyst-version was created
    pub version_file_created: bool,

    /// Whether .catalyst-hashes.json was created
    pub hashes_file_created: bool,

    /// Any warnings or notes for the user
    pub warnings: Vec<String>,
}

impl InitReport {
    pub fn new() -> Self {
        Self {
            created_dirs: Vec::new(),
            installed_hooks: Vec::new(),
            installed_skills: Vec::new(),
            settings_created: false,
            version_file_created: false,
            hashes_file_created: false,
            warnings: Vec::new(),
        }
    }
}

// ============================================================================
// Update Command Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReport {
    /// Skills that were updated
    pub updated_skills: Vec<String>,

    /// Skills that were skipped because user modified them
    pub skipped_skills: Vec<SkippedSkill>,

    /// Hooks that were updated
    pub updated_hooks: Vec<String>,

    /// Whether binary updates are available
    pub binary_updates_available: Vec<String>,

    /// Overall success status
    pub success: bool,

    /// Any errors that occurred
    pub errors: Vec<String>,
}

impl UpdateReport {
    pub fn new() -> Self {
        Self {
            updated_skills: Vec::new(),
            skipped_skills: Vec::new(),
            updated_hooks: Vec::new(),
            binary_updates_available: Vec::new(),
            success: true,
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedSkill {
    /// Name of the skill
    pub name: String,

    /// Reason it was skipped
    pub reason: String,

    /// Current hash (user's modified version)
    pub current_hash: String,

    /// Expected hash (from .catalyst-hashes.json)
    pub expected_hash: String,
}

// ============================================================================
// Fix Command Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixReport {
    /// Issues that were fixed
    pub fixed_issues: Vec<String>,

    /// Issues that could not be auto-fixed
    pub remaining_issues: Vec<Issue>,

    /// Overall success status
    pub success: bool,
}

impl FixReport {
    pub fn new() -> Self {
        Self {
            fixed_issues: Vec::new(),
            remaining_issues: Vec::new(),
            success: true,
        }
    }
}

// ============================================================================
// Status Command Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    /// Overall status level
    pub level: StatusLevel,

    /// Binary statuses (skill-activation-prompt, file-change-tracker)
    pub binaries: Vec<BinaryStatus>,

    /// Hook statuses
    pub hooks: Vec<HookStatus>,

    /// Skill statuses
    pub skills: Vec<SkillStatus>,

    /// List of issues found
    pub issues: Vec<Issue>,

    /// Whether .catalyst-version exists and matches
    pub version_status: VersionStatus,
}

impl StatusReport {
    pub fn new() -> Self {
        Self {
            level: StatusLevel::Ok,
            binaries: Vec::new(),
            hooks: Vec::new(),
            skills: Vec::new(),
            issues: Vec::new(),
            version_status: VersionStatus::Missing,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusLevel {
    /// Everything is working perfectly
    Ok,

    /// Minor issues, everything still works
    Warning,

    /// Critical issues, some features broken
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStatus {
    /// Binary name (e.g., "skill-activation-prompt")
    pub name: String,

    /// Whether the binary exists in ~/.claude-hooks/bin/
    pub exists: bool,

    /// Whether the binary is executable
    pub executable: bool,

    /// Binary version (if detectable)
    pub version: Option<String>,

    /// Expected version (from embedded resources or latest release)
    pub expected_version: Option<String>,

    /// Whether version matches expected
    pub version_matches: bool,

    /// Full path to binary
    pub path: Option<PathBuf>,

    /// Variant of the binary (for file-change-tracker: "sqlite" or "basic")
    /// None for binaries that don't have variants
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookStatus {
    /// Hook name (e.g., "skill-activation-prompt.sh")
    pub name: String,

    /// Whether the hook file exists in .claude/hooks/
    pub exists: bool,

    /// Whether the hook is executable (Unix only)
    pub executable: bool,

    /// Whether the hook is configured in settings.json
    pub configured: bool,

    /// The event it's configured for (e.g., "UserPromptSubmit")
    pub event: Option<String>,

    /// Full path to hook file
    pub path: Option<PathBuf>,

    /// Whether the hook script calls the correct binary
    pub calls_correct_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStatus {
    /// Skill name (e.g., "skill-developer")
    pub name: String,

    /// Whether the skill directory exists in .claude/skills/
    pub exists: bool,

    /// Whether the skill has a SKILL.md file
    pub has_main_file: bool,

    /// Whether the skill is registered in skill-rules.json
    pub registered: bool,

    /// Hash of SKILL.md (for modification detection)
    pub current_hash: Option<String>,

    /// Expected hash from .catalyst-hashes.json
    pub expected_hash: Option<String>,

    /// Whether the skill has been modified by user
    pub modified: bool,

    /// Full path to skill directory
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Component affected (e.g., "skill-activation-prompt binary")
    pub component: String,

    /// Human-readable description
    pub description: String,

    /// Whether this can be auto-fixed
    pub auto_fixable: bool,

    /// Suggested fix command (e.g., "catalyst fix")
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issue, feature is broken
    Error,

    /// Non-critical issue, feature works but not optimal
    Warning,

    /// Informational message
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionStatus {
    /// .catalyst-version file doesn't exist
    Missing,

    /// Version matches expected
    Ok { version: String },

    /// Version doesn't match expected
    Mismatch { expected: String, found: String },
}

// ============================================================================
// Settings.json Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub hooks: Vec<Hook>,

    #[serde(flatten)]
    pub other: serde_json::Value, // Preserve other fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub event: String,
    pub script: String,
    pub async_mode: Option<String>,
}

// ============================================================================
// Skill Rules Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRules {
    pub skills: Vec<SkillRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRule {
    pub name: String,

    #[serde(default)]
    pub keywords: Vec<String>,

    #[serde(default, rename = "pathPatterns")]
    pub path_patterns: Vec<String>,

    #[serde(default, rename = "intentPatterns")]
    pub intent_patterns: Vec<String>,

    #[serde(flatten)]
    pub other: serde_json::Value, // Preserve other fields
}

// ============================================================================
// Hash Tracking Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalystHashes {
    /// Version of the catalyst CLI that created these hashes
    pub version: String,

    /// Timestamp when hashes were created/updated
    pub updated_at: String,

    /// Skill file hashes (skill_name -> hash)
    pub skills: std::collections::HashMap<String, String>,

    /// Hook file hashes (hook_name -> hash)
    pub hooks: std::collections::HashMap<String, String>,
}

impl CatalystHashes {
    pub fn new(version: String) -> Self {
        use chrono::Utc;
        Self {
            version,
            updated_at: Utc::now().to_rfc3339(),
            skills: std::collections::HashMap::new(),
            hooks: std::collections::HashMap::new(),
        }
    }
}

// ============================================================================
// Constants
// ============================================================================

/// Catalyst CLI version (from Cargo.toml)
pub const CATALYST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default skills available for installation
pub const AVAILABLE_SKILLS: &[&str] = &[
    "skill-developer",
    "backend-dev-guidelines",
    "frontend-dev-guidelines",
    "route-tester",
    "error-tracking",
];

/// Default directory structure
pub const CLAUDE_DIR: &str = ".claude";
pub const HOOKS_DIR: &str = ".claude/hooks";
pub const SKILLS_DIR: &str = ".claude/skills";
pub const AGENTS_DIR: &str = ".claude/agents";
pub const COMMANDS_DIR: &str = ".claude/commands";

/// Configuration files
pub const SETTINGS_FILE: &str = ".claude/settings.json";
pub const SKILL_RULES_FILE: &str = ".claude/skills/skill-rules.json";
pub const VERSION_FILE: &str = ".catalyst-version";
pub const HASHES_FILE: &str = ".catalyst-hashes.json";

/// Binary installation directory
pub const BINARY_DIR: &str = ".claude-hooks/bin";

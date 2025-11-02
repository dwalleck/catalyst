use anyhow::{Context, Result};
use chrono::Utc;
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tracing::debug;

// Pre-compiled regex patterns for file analysis (10-100x faster than compiling on each call)
static TRY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"try\s*\{").unwrap());
static ASYNC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"async\s+").unwrap());
static PRISMA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"prisma\.|PrismaClient").unwrap());
static CONTROLLER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Controller|router\.|app\.(get|post)").unwrap());
static API_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"fetch\(|axios\.|apiClient\.").unwrap());

// SQL update statements as named constants for maintainability
const SQL_UPDATE_BACKEND: &str = "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1, backend_files = backend_files + 1 WHERE session_id = ?2";
const SQL_UPDATE_FRONTEND: &str = "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1, frontend_files = frontend_files + 1 WHERE session_id = ?2";
const SQL_UPDATE_DATABASE: &str = "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1, database_files = database_files + 1 WHERE session_id = ?2";
const SQL_UPDATE_OTHER: &str =
    "UPDATE sessions SET last_activity = ?1, total_files = total_files + 1 WHERE session_id = ?2";

/// Returns the home directory path in a cross-platform way
/// On Windows: Uses USERPROFILE, falls back to HOME, then TEMP, then LOCALAPPDATA, then C:\Users\Default
/// On Unix/Linux/macOS: Uses HOME
#[cfg(windows)]
fn get_home_dir() -> PathBuf {
    env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .or_else(|_| env::var("TEMP"))
        .or_else(|_| env::var("LOCALAPPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"))
}

#[cfg(not(windows))]
fn get_home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// File category classification for tracking purposes
#[derive(Debug, Clone, Copy)]
enum Category {
    Backend,
    Frontend,
    Database,
    Other,
}

impl Category {
    /// Returns the string representation for database storage
    fn as_str(&self) -> &'static str {
        match self {
            Category::Backend => "backend",
            Category::Frontend => "frontend",
            Category::Database => "database",
            Category::Other => "other",
        }
    }

    /// Returns the SQL update statement for this category
    fn sql_update(&self) -> &'static str {
        match self {
            Category::Backend => SQL_UPDATE_BACKEND,
            Category::Frontend => SQL_UPDATE_FRONTEND,
            Category::Database => SQL_UPDATE_DATABASE,
            Category::Other => SQL_UPDATE_OTHER,
        }
    }
}

#[derive(Debug, Deserialize)]
struct HookInput {
    session_id: String,
    tool_name: Option<String>,
    tool_args: Option<HashMap<String, serde_json::Value>>,
}

struct Database {
    conn: Connection,
}

/// Validates session_id to prevent path traversal attacks
/// Only allows alphanumeric characters, hyphens, and underscores
fn validate_session_id(session_id: &str) -> Result<()> {
    if session_id.is_empty() {
        anyhow::bail!("session_id cannot be empty");
    }

    if session_id.len() > 255 {
        anyhow::bail!("session_id exceeds maximum length of 255 characters");
    }

    // Only allow alphanumeric, hyphens, and underscores (prevent path traversal)
    if !session_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "session_id contains invalid characters (only alphanumeric, hyphens, and underscores allowed)"
        );
    }

    Ok(())
}

impl Database {
    fn new(session_id: &str) -> Result<Self> {
        // Validate session_id to prevent path traversal attacks
        validate_session_id(session_id)?;

        // Cross-platform home directory and path construction
        let home = get_home_dir();
        let hooks_dir = home.join(".claude").join("hooks-state-rust");
        let db_path = hooks_dir.join(format!("{session_id}.db"));

        // Ensure directory exists
        fs::create_dir_all(&hooks_dir)
            .with_context(|| format!("Failed to create hooks directory: {:?}", hooks_dir))?;

        let conn = Connection::open(&db_path)?;

        // Create schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_modifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                tool TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                category TEXT NOT NULL,
                has_async BOOLEAN DEFAULT 0,
                has_try_catch BOOLEAN DEFAULT 0,
                has_prisma BOOLEAN DEFAULT 0,
                has_controller BOOLEAN DEFAULT 0,
                has_api_call BOOLEAN DEFAULT 0,
                line_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Create indexes for fast queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session
             ON file_modifications(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_category
             ON file_modifications(session_id, category)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp
             ON file_modifications(timestamp DESC)",
            [],
        )?;

        // Create session summary table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                start_time TEXT NOT NULL,
                last_activity TEXT NOT NULL,
                total_files INTEGER DEFAULT 0,
                backend_files INTEGER DEFAULT 0,
                frontend_files INTEGER DEFAULT 0,
                database_files INTEGER DEFAULT 0
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    fn track_modification(&self, session_id: &str, file_path: &str, tool: &str) -> Result<()> {
        let category = get_file_category(file_path);
        let analysis = if should_analyze(file_path) {
            analyze_file(file_path)
        } else {
            FileAnalysis::default()
        };

        let timestamp = Utc::now().to_rfc3339();

        // Insert file modification
        self.conn.execute(
            "INSERT INTO file_modifications
             (session_id, file_path, tool, timestamp, category,
              has_async, has_try_catch, has_prisma, has_controller, has_api_call, line_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session_id,
                file_path,
                tool,
                timestamp,
                category.as_str(),
                analysis.has_async,
                analysis.has_try_catch,
                analysis.has_prisma,
                analysis.has_controller,
                analysis.has_api_call,
                analysis.line_count,
            ],
        )?;

        // Update session summary
        self.update_session_summary(session_id, category)?;

        Ok(())
    }

    fn update_session_summary(&self, session_id: &str, category: Category) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        // Check if session exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM sessions WHERE session_id = ?1",
                params![session_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !exists {
            // Create new session
            self.conn.execute(
                "INSERT INTO sessions (session_id, start_time, last_activity, total_files)
                 VALUES (?1, ?2, ?3, 1)",
                params![session_id, &now, &now],
            )?;
        }

        // Update session using type-safe category enum with const SQL strings
        self.conn
            .execute(category.sql_update(), params![&now, session_id])?;

        Ok(())
    }
}

#[derive(Default)]
struct FileAnalysis {
    has_async: bool,
    has_try_catch: bool,
    has_prisma: bool,
    has_controller: bool,
    has_api_call: bool,
    line_count: i32,
}

// Cross-platform path categorization using path components instead of string contains
fn get_file_category(path: &str) -> Category {
    let path_obj = Path::new(path);

    // Check each path component (works on both Unix and Windows)
    for component in path_obj.components() {
        if let Some(comp_str) = component.as_os_str().to_str() {
            match comp_str {
                "frontend" | "client" | "components" | "features" => return Category::Frontend,
                "controllers" | "services" | "routes" | "api" | "backend" | "server" => {
                    return Category::Backend
                }
                "database" | "prisma" | "migrations" => return Category::Database,
                _ => continue,
            }
        }
    }

    Category::Other
}

fn should_analyze(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    !path_lower.contains(".test.")
        && !path_lower.contains(".spec.")
        && (path_lower.ends_with(".ts")
            || path_lower.ends_with(".tsx")
            || path_lower.ends_with(".js")
            || path_lower.ends_with(".jsx"))
}

fn analyze_file(path: &str) -> FileAnalysis {
    let Ok(content) = fs::read_to_string(path) else {
        return FileAnalysis::default();
    };

    let line_count = content.lines().count() as i32;

    // Use pre-compiled static regexes (10-100x faster than compiling on each call)
    FileAnalysis {
        has_try_catch: TRY_REGEX.is_match(&content),
        has_async: ASYNC_REGEX.is_match(&content),
        has_prisma: PRISMA_REGEX.is_match(&content),
        has_controller: CONTROLLER_REGEX.is_match(&content),
        has_api_call: API_REGEX.is_match(&content),
        line_count,
    }
}

fn extract_file_path(_tool: &str, args: &HashMap<String, serde_json::Value>) -> Option<String> {
    args.get("file_path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Read stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let data: HookInput = serde_json::from_str(&input)?;

    // Only track file modification tools
    let file_tools = ["Edit", "Write", "MultiEdit", "NotebookEdit"];
    if let Some(ref tool) = data.tool_name {
        if !file_tools.contains(&tool.as_str()) {
            return Ok(());
        }

        // Extract file path
        if let Some(ref args) = data.tool_args {
            if let Some(file_path) = extract_file_path(tool, args) {
                let db = Database::new(&data.session_id)?;
                db.track_modification(&data.session_id, &file_path, tool)?;

                // Structured logging (controlled by RUST_LOG=debug)
                debug!(
                    file_path = %file_path,
                    category = %get_file_category(&file_path).as_str(),
                    tool = %tool,
                    "Tracked file modification"
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_session_id_valid() {
        assert!(validate_session_id("test-123").is_ok());
        assert!(validate_session_id("session_456").is_ok());
        assert!(validate_session_id("abc123-xyz789").is_ok());
        assert!(validate_session_id("UPPERCASE").is_ok());
        assert!(validate_session_id("lower-Case_123").is_ok());
    }

    #[test]
    fn test_validate_session_id_invalid_empty() {
        assert!(validate_session_id("").is_err());
    }

    #[test]
    fn test_validate_session_id_invalid_length() {
        let long_id = "a".repeat(256);
        assert!(validate_session_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_session_id_invalid_characters() {
        // Path traversal attempts
        assert!(validate_session_id("../etc/passwd").is_err());
        assert!(validate_session_id("..\\windows\\system32").is_err());

        // Special characters
        assert!(validate_session_id("session@123").is_err());
        assert!(validate_session_id("session#123").is_err());
        assert!(validate_session_id("session 123").is_err());
        assert!(validate_session_id("session/123").is_err());
        assert!(validate_session_id("session\\123").is_err());
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(Category::Backend.as_str(), "backend");
        assert_eq!(Category::Frontend.as_str(), "frontend");
        assert_eq!(Category::Database.as_str(), "database");
        assert_eq!(Category::Other.as_str(), "other");
    }

    #[test]
    fn test_category_sql_update() {
        assert!(Category::Backend.sql_update().contains("backend_files"));
        assert!(Category::Frontend.sql_update().contains("frontend_files"));
        assert!(Category::Database.sql_update().contains("database_files"));
        assert!(!Category::Other.sql_update().contains("backend_files"));
    }

    #[test]
    fn test_get_file_category_frontend() {
        assert!(matches!(
            get_file_category("/project/frontend/App.tsx"),
            Category::Frontend
        ));
        assert!(matches!(
            get_file_category("/project/client/Button.tsx"),
            Category::Frontend
        ));
        assert!(matches!(
            get_file_category("/project/src/components/Header.tsx"),
            Category::Frontend
        ));
        assert!(matches!(
            get_file_category("/project/features/auth/Login.tsx"),
            Category::Frontend
        ));
    }

    #[test]
    fn test_get_file_category_backend() {
        assert!(matches!(
            get_file_category("/project/controllers/UserController.ts"),
            Category::Backend
        ));
        assert!(matches!(
            get_file_category("/project/services/AuthService.ts"),
            Category::Backend
        ));
        assert!(matches!(
            get_file_category("/project/routes/api.ts"),
            Category::Backend
        ));
        assert!(matches!(
            get_file_category("/project/api/handlers.ts"),
            Category::Backend
        ));
        assert!(matches!(
            get_file_category("/project/backend/server.ts"),
            Category::Backend
        ));
        assert!(matches!(
            get_file_category("/project/server/index.ts"),
            Category::Backend
        ));
    }

    #[test]
    fn test_get_file_category_database() {
        assert!(matches!(
            get_file_category("/project/database/schema.sql"),
            Category::Database
        ));
        assert!(matches!(
            get_file_category("/project/prisma/schema.prisma"),
            Category::Database
        ));
        assert!(matches!(
            get_file_category("/project/migrations/001_init.sql"),
            Category::Database
        ));
    }

    #[test]
    fn test_get_file_category_other() {
        assert!(matches!(
            get_file_category("/project/utils/helpers.ts"),
            Category::Other
        ));
        assert!(matches!(
            get_file_category("/project/lib/logger.ts"),
            Category::Other
        ));
        assert!(matches!(
            get_file_category("/project/README.md"),
            Category::Other
        ));
    }

    #[test]
    fn test_should_analyze_valid_files() {
        assert!(should_analyze("/project/app.ts"));
        assert!(should_analyze("/project/Component.tsx"));
        assert!(should_analyze("/project/script.js"));
        assert!(should_analyze("/project/App.jsx"));
    }

    #[test]
    fn test_should_analyze_skip_test_files() {
        assert!(!should_analyze("/project/app.test.ts"));
        assert!(!should_analyze("/project/Component.spec.tsx"));
        assert!(!should_analyze("/project/test.spec.js"));
    }

    #[test]
    fn test_should_analyze_skip_non_code_files() {
        assert!(!should_analyze("/project/README.md"));
        assert!(!should_analyze("/project/config.json"));
        assert!(!should_analyze("/project/styles.css"));
    }

    #[test]
    fn test_extract_file_path_with_valid_path() {
        let mut args = HashMap::new();
        args.insert(
            "file_path".to_string(),
            serde_json::Value::String("/project/test.ts".to_string()),
        );

        let result = extract_file_path("Edit", &args);
        assert_eq!(result, Some("/project/test.ts".to_string()));
    }

    #[test]
    fn test_extract_file_path_missing_key() {
        let args = HashMap::new();
        let result = extract_file_path("Edit", &args);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_file_path_non_string_value() {
        let mut args = HashMap::new();
        args.insert(
            "file_path".to_string(),
            serde_json::Value::Number(123.into()),
        );

        let result = extract_file_path("Edit", &args);
        assert_eq!(result, None);
    }

    #[test]
    fn test_hook_input_deserialization() {
        let json = r#"{
            "session_id": "test-123",
            "tool_name": "Edit",
            "tool_args": {
                "file_path": "/project/app.ts"
            }
        }"#;

        let result: Result<HookInput, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.session_id, "test-123");
        assert_eq!(input.tool_name, Some("Edit".to_string()));
    }

    #[test]
    fn test_hook_input_optional_fields() {
        let json = r#"{
            "session_id": "test-123"
        }"#;

        let result: Result<HookInput, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.session_id, "test-123");
        assert_eq!(input.tool_name, None);
        assert_eq!(input.tool_args, None);
    }

    #[test]
    fn test_file_analysis_default() {
        let analysis = FileAnalysis::default();
        assert!(!analysis.has_try_catch);
        assert!(!analysis.has_async);
        assert!(!analysis.has_prisma);
        assert!(!analysis.has_controller);
        assert!(!analysis.has_api_call);
        assert_eq!(analysis.line_count, 0);
    }
}

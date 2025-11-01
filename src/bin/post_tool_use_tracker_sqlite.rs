use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
struct HookInput {
    session_id: String,
    tool_name: Option<String>,
    tool_args: Option<HashMap<String, serde_json::Value>>,
}

struct Database {
    conn: Connection,
}

impl Database {
    fn new(session_id: &str) -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let db_path = format!("{home}/.claude/hooks-state-rust/{session_id}.db");

        // Ensure directory exists
        fs::create_dir_all(format!("{home}/.claude/hooks-state-rust"))?;

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
                category,
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

    fn update_session_summary(&self, session_id: &str, category: &str) -> Result<()> {
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

        // Update session
        let category_col = match category {
            "backend" => "backend_files",
            "frontend" => "frontend_files",
            "database" => "database_files",
            _ => "",
        };

        if !category_col.is_empty() {
            self.conn.execute(
                &format!(
                    "UPDATE sessions
                     SET last_activity = ?1,
                         total_files = total_files + 1,
                         {category_col} = {category_col} + 1
                     WHERE session_id = ?2"
                ),
                params![&now, session_id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE sessions
                 SET last_activity = ?1,
                     total_files = total_files + 1
                 WHERE session_id = ?2",
                params![&now, session_id],
            )?;
        }

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

fn get_file_category(path: &str) -> &str {
    if path.contains("/frontend/")
        || path.contains("/client/")
        || path.contains("/src/components/")
        || path.contains("/src/features/")
    {
        "frontend"
    } else if path.contains("/src/controllers/")
        || path.contains("/src/services/")
        || path.contains("/src/routes/")
        || path.contains("/backend/")
    {
        "backend"
    } else if path.contains("/database/") || path.contains("/prisma/") {
        "database"
    } else {
        "other"
    }
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

    // Compile regexes
    let try_regex = Regex::new(r"try\s*\{").unwrap();
    let async_regex = Regex::new(r"async\s+").unwrap();
    let prisma_regex = Regex::new(r"prisma\.|PrismaClient").unwrap();
    let controller_regex = Regex::new(r"Controller|router\.|app\.(get|post)").unwrap();
    let api_regex = Regex::new(r"fetch\(|axios\.|apiClient\.").unwrap();

    FileAnalysis {
        has_try_catch: try_regex.is_match(&content),
        has_async: async_regex.is_match(&content),
        has_prisma: prisma_regex.is_match(&content),
        has_controller: controller_regex.is_match(&content),
        has_api_call: api_regex.is_match(&content),
        line_count,
    }
}

fn extract_file_path(_tool: &str, args: &HashMap<String, serde_json::Value>) -> Option<String> {
    args.get("file_path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn main() -> Result<()> {
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

                // Debug output
                if std::env::var("DEBUG_HOOKS").is_ok() {
                    eprintln!(
                        "[Rust/SQLite] Tracked: {file_path} ({})",
                        get_file_category(&file_path)
                    );
                }
            }
        }
    }

    Ok(())
}

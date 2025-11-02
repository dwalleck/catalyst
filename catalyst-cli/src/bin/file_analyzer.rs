use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info, warn};

// Pre-compile regex patterns at module initialization (CRITICAL PERFORMANCE IMPROVEMENT)
static TRY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"try\s*\{|try:|except:").unwrap());
static ASYNC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"async\s+|async def|async fn|Task<").unwrap());
static PRISMA_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"prisma\.|PrismaClient|findMany|findUnique|create\(").unwrap());
static CONTROLLER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Controller|router\.|app\.(get|post|put|delete)|HttpGet|HttpPost").unwrap()
});
static API_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"fetch\(|axios\.|HttpClient|apiClient\.").unwrap());

// Pre-compiled globsets for efficient pattern matching (Phase 2.5 optimization)
static CODE_EXTENSIONS: Lazy<GlobSet> = Lazy::new(|| {
    let mut builder = GlobSetBuilder::new();
    for pattern in &[
        "*.ts", "*.tsx", "*.js", "*.jsx", "*.rs", "*.cs", "*.py", "*.go", "*.java", "*.c", "*.cpp",
        "*.h",
    ] {
        builder.add(Glob::new(pattern).unwrap());
    }
    builder.build().unwrap()
});

static SKIP_PATTERNS: Lazy<GlobSet> = Lazy::new(|| {
    let mut builder = GlobSetBuilder::new();
    // Skip test files, config files, and type definitions
    for pattern in &[
        "*.test.*",
        "*.spec.*",
        "*.config.*",
        "*/types/*",
        "*.json",
        "*.md",
    ] {
        builder.add(Glob::new(pattern).unwrap());
    }
    builder.build().unwrap()
});

/// Analyzes files in a directory for error-prone patterns
#[derive(Parser)]
#[command(name = "file-analyzer")]
#[command(about = "Analyzes files for error-prone patterns", long_about = None)]
#[command(version)]
struct Args {
    /// Directory to analyze
    directory: PathBuf,

    /// Show detailed output including all files analyzed
    #[arg(short, long)]
    verbose: bool,

    /// Output format
    #[arg(short, long, default_value = "text", value_parser = ["text", "json"])]
    format: String,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

#[derive(Debug, Default)]
struct FileAnalysis {
    has_try_catch: bool,
    has_async: bool,
    has_prisma: bool,
    has_controller: bool,
    has_api_call: bool,
}

#[derive(Default)]
struct Stats {
    total_files: usize,
    backend_files: usize,
    frontend_files: usize,
    database_files: usize,
    other_files: usize,
    async_files: usize,
    try_catch_files: usize,
    prisma_files: usize,
    controller_files: usize,
    api_call_files: usize,
    failed_files: usize,
}

// Cross-platform path categorization using path components instead of string contains
fn get_file_category(path: &Path) -> &str {
    // Check each path component (works on both Unix and Windows)
    for component in path.components() {
        if let Some(comp_str) = component.as_os_str().to_str() {
            match comp_str {
                "frontend" | "client" | "components" | "features" => return "frontend",
                "controllers" | "services" | "routes" | "api" | "backend" | "server" => {
                    return "backend"
                }
                "database" | "prisma" | "migrations" => return "database",
                _ => continue,
            }
        }
    }

    // Check file extension for SQL files
    if path.extension().and_then(|ext| ext.to_str()) == Some("sql") {
        return "database";
    }

    "other"
}

// Phase 2.5: Optimized with globset (O(1) instead of O(n) chain of checks)
fn should_analyze(path: &Path) -> bool {
    // Skip files matching skip patterns
    if SKIP_PATTERNS.is_match(path) {
        return false;
    }

    // Check if it's a code file
    CODE_EXTENSIONS.is_match(path)
}

fn analyze_file(path: &Path) -> Result<FileAnalysis> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Use pre-compiled static regexes (10-100x faster than compiling on each call)
    Ok(FileAnalysis {
        has_try_catch: TRY_REGEX.is_match(&content),
        has_async: ASYNC_REGEX.is_match(&content),
        has_prisma: PRISMA_REGEX.is_match(&content),
        has_controller: CONTROLLER_REGEX.is_match(&content),
        has_api_call: API_REGEX.is_match(&content),
    })
}

fn print_json_results(stats: &Stats, elapsed: std::time::Duration) {
    let json = serde_json::json!({
        "total_files": stats.total_files,
        "failed_files": stats.failed_files,
        "categories": {
            "backend": stats.backend_files,
            "frontend": stats.frontend_files,
            "database": stats.database_files,
            "other": stats.other_files
        },
        "patterns": {
            "async": stats.async_files,
            "try_catch": stats.try_catch_files,
            "prisma": stats.prisma_files,
            "controllers": stats.controller_files,
            "api_calls": stats.api_call_files
        },
        "duration_ms": elapsed.as_millis()
    });

    // Handle serialization error gracefully (though unlikely with simple JSON)
    println!(
        "{}",
        serde_json::to_string_pretty(&json)
            .unwrap_or_else(|e| { format!(r#"{{"error": "Failed to serialize JSON: {}"}}"#, e) })
    );
}

fn print_text_results(stats: &Stats, elapsed: std::time::Duration, use_color: bool) {
    if use_color {
        println!(
            "\n{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue()
        );
        println!("{}\n", "📊 ANALYSIS RESULTS".bright_yellow().bold());
    } else {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 ANALYSIS RESULTS\n");
    }

    println!("Total Files:    {}", stats.total_files);
    if stats.failed_files > 0 {
        println!("Failed Files:   {}", stats.failed_files);
    }
    println!("  Backend:      {}", stats.backend_files);
    println!("  Frontend:     {}", stats.frontend_files);
    println!("  Database:     {}", stats.database_files);
    println!("  Other:        {}", stats.other_files);
    println!("\nPatterns Detected:");
    println!("  Async:        {}", stats.async_files);
    println!("  Try/Catch:    {}", stats.try_catch_files);
    println!("  Prisma:       {}", stats.prisma_files);
    println!("  Controllers:  {}", stats.controller_files);
    println!("  API Calls:    {}", stats.api_call_files);

    if use_color {
        println!(
            "{}",
            format!("\n⚡ Analysis completed in {:.2?}", elapsed).bright_green()
        );
        println!(
            "{}\n",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue()
        );
    } else {
        println!("\n⚡ Analysis completed in {:.2?}", elapsed);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Disable colors if requested or if NO_COLOR is set
    let use_color = !args.no_color && std::env::var("NO_COLOR").is_err();
    if !use_color {
        colored::control::set_override(false);
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Analyzing directory: {:?}", args.directory);

    if !args.directory.exists() {
        anyhow::bail!("Directory does not exist: {}", args.directory.display());
    }

    let start = Instant::now();

    if args.format == "text" {
        if use_color {
            println!(
                "\n{}\n",
                format!("🔍 ANALYZING FILES IN: {}", args.directory.display())
                    .bright_cyan()
                    .bold()
            );
        } else {
            println!("\n🔍 ANALYZING FILES IN: {}\n", args.directory.display());
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    let mut stats = Stats::default();

    // Phase 2.5: Use ignore crate instead of WalkDir (respects .gitignore, 10-100x faster)
    for result in WalkBuilder::new(&args.directory).build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                warn!("Failed to read entry: {}", err);
                continue;
            }
        };

        // Only process files
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();

        // Phase 2.5: Optimized pattern matching with globset
        if !should_analyze(path) {
            continue;
        }

        stats.total_files += 1;
        let category = get_file_category(path);

        match category {
            "backend" => stats.backend_files += 1,
            "frontend" => stats.frontend_files += 1,
            "database" => stats.database_files += 1,
            _ => stats.other_files += 1,
        }

        if args.verbose {
            debug!("Analyzing: {} ({})", path.display(), category);
        }

        match analyze_file(path) {
            Ok(analysis) => {
                if analysis.has_async {
                    stats.async_files += 1;
                }
                if analysis.has_try_catch {
                    stats.try_catch_files += 1;
                }
                if analysis.has_prisma {
                    stats.prisma_files += 1;
                }
                if analysis.has_controller {
                    stats.controller_files += 1;
                }
                if analysis.has_api_call {
                    stats.api_call_files += 1;
                }

                // Flag risky patterns
                if analysis.has_async && !analysis.has_try_catch {
                    if args.format == "text" {
                        // Safe: We know this is a file from walkdir, so file_name() won't be None
                        let file_name = path
                            .file_name()
                            .map(|name| name.to_string_lossy())
                            .unwrap_or_else(|| path.display().to_string().into());

                        if use_color {
                            println!(
                                "{}",
                                format!("⚠️  {} - Async without try/catch", file_name).yellow()
                            );
                        } else {
                            println!("⚠️  {} - Async without try/catch", file_name);
                        }
                    }

                    warn!(
                        file = %path.display(),
                        "Async code without try/catch"
                    );
                }
            }
            Err(e) => {
                warn!("Failed to analyze {}: {}", path.display(), e);
                stats.failed_files += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    match args.format.as_str() {
        "json" => print_json_results(&stats, elapsed),
        _ => print_text_results(&stats, elapsed, use_color),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_file_category_frontend() {
        assert_eq!(
            get_file_category(&PathBuf::from("/project/frontend/App.tsx")),
            "frontend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/client/Button.tsx")),
            "frontend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/src/components/Header.tsx")),
            "frontend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/features/auth/Login.tsx")),
            "frontend"
        );
    }

    #[test]
    fn test_get_file_category_backend() {
        assert_eq!(
            get_file_category(&PathBuf::from("/project/controllers/UserController.ts")),
            "backend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/services/AuthService.ts")),
            "backend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/routes/api.ts")),
            "backend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/backend/server.ts")),
            "backend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/server/index.ts")),
            "backend"
        );
    }

    #[test]
    fn test_get_file_category_database() {
        assert_eq!(
            get_file_category(&PathBuf::from("/project/database/schema.sql")),
            "database"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/prisma/schema.prisma")),
            "database"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/migrations/001_init.sql")),
            "database"
        );
        // SQL files are categorized as database regardless of path
        assert_eq!(
            get_file_category(&PathBuf::from("/project/queries/user.sql")),
            "database"
        );
    }

    #[test]
    fn test_get_file_category_other() {
        assert_eq!(
            get_file_category(&PathBuf::from("/project/utils/helpers.ts")),
            "other"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/lib/logger.ts")),
            "other"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("/project/README.md")),
            "other"
        );
    }

    #[test]
    fn test_should_analyze_valid_extensions() {
        assert!(should_analyze(&PathBuf::from("/project/app.ts")));
        assert!(should_analyze(&PathBuf::from("/project/Component.tsx")));
        assert!(should_analyze(&PathBuf::from("/project/script.js")));
        assert!(should_analyze(&PathBuf::from("/project/App.jsx")));
        assert!(should_analyze(&PathBuf::from("/project/main.rs")));
        assert!(should_analyze(&PathBuf::from("/project/program.cs")));
        assert!(should_analyze(&PathBuf::from("/project/script.py")));
    }

    #[test]
    fn test_should_analyze_skip_test_files() {
        assert!(!should_analyze(&PathBuf::from("/project/app.test.ts")));
        assert!(!should_analyze(&PathBuf::from(
            "/project/Component.spec.tsx"
        )));
        assert!(!should_analyze(&PathBuf::from("/project/test.spec.js")));
    }

    #[test]
    fn test_should_analyze_skip_config_files() {
        assert!(!should_analyze(&PathBuf::from(
            "/project/webpack.config.js"
        )));
        assert!(!should_analyze(&PathBuf::from("/project/tsconfig.json")));
        assert!(!should_analyze(&PathBuf::from("/project/README.md")));
    }

    #[test]
    fn test_async_regex() {
        let code_with_async = "async function fetchData() { return data; }";
        assert!(ASYNC_REGEX.is_match(code_with_async));

        let code_with_async_arrow = "const fetch = async () => { return data; }";
        assert!(ASYNC_REGEX.is_match(code_with_async_arrow));

        let code_without_async = "function getData() { return data; }";
        assert!(!ASYNC_REGEX.is_match(code_without_async));
    }

    #[test]
    fn test_async_regex_edge_cases() {
        // Test with varied whitespace
        assert!(ASYNC_REGEX.is_match("async  function test() {}"));
        assert!(ASYNC_REGEX.is_match("async\tfunction test() {}"));
        assert!(ASYNC_REGEX.is_match("async\nfunction test() {}"));

        // Test in comments (should still match - this is expected behavior)
        assert!(ASYNC_REGEX.is_match("// async function in comment"));

        // Test partial matches - asyncFunction without space doesn't match
        assert!(!ASYNC_REGEX.is_match("asyncFunction() {}"));
        // Note: "myasync function" WILL match because "async " appears in it
        // This is acceptable behavior - we're looking for the async keyword pattern

        // Test TypeScript async patterns
        assert!(ASYNC_REGEX.is_match("async def process():"));
        assert!(ASYNC_REGEX.is_match("async fn handler() ->"));
        assert!(ASYNC_REGEX.is_match("Task<string> result"));
    }

    #[test]
    fn test_try_regex() {
        // Test all branches: try\s*\{|try:|except:

        // Branch 1: try\s*\{ (JavaScript/Java try blocks)
        let code_with_try = "try { doSomething(); } catch (e) { handleError(e); }";
        assert!(TRY_REGEX.is_match(code_with_try));
        assert!(TRY_REGEX.is_match("try{ noSpace(); }")); // No space before brace

        // Branch 2: try: (Python try)
        let code_with_python_try = "try:\n    do_something()\nexcept Exception as e:\n    pass";
        assert!(TRY_REGEX.is_match(code_with_python_try));

        // Branch 3: except: (Python bare except block)
        assert!(TRY_REGEX.is_match("except:\n    pass"));
        assert!(TRY_REGEX.is_match("    except:")); // Indented except

        let code_without_try = "function process() { return result; }";
        assert!(!TRY_REGEX.is_match(code_without_try));
    }

    #[test]
    fn test_prisma_regex() {
        // Test all branches: prisma\.|PrismaClient|findMany|findUnique|create\(

        // Branch 1: prisma. (Prisma client method calls)
        let code_with_prisma = "const user = await prisma.user.findUnique({ where: { id } });";
        assert!(PRISMA_REGEX.is_match(code_with_prisma));

        // Branch 2: PrismaClient (client instantiation)
        assert!(PRISMA_REGEX.is_match("const prisma = new PrismaClient();"));
        assert!(PRISMA_REGEX.is_match("import { PrismaClient } from '@prisma/client';"));

        // Branch 3: findMany (query method)
        assert!(PRISMA_REGEX.is_match("const users = await findMany({ where: { active: true } });"));

        // Branch 4: findUnique (query method)
        assert!(PRISMA_REGEX.is_match("const user = await findUnique({ where: { id } });"));

        // Branch 5: create( (create method)
        let code_with_prisma_create = "const post = await prisma.post.create({ data: { title } });";
        assert!(PRISMA_REGEX.is_match(code_with_prisma_create));

        let code_without_prisma = "const user = await database.query('SELECT * FROM users');";
        assert!(!PRISMA_REGEX.is_match(code_without_prisma));
    }

    #[test]
    fn test_controller_regex() {
        // Test all branches: Controller|router\.|app\.(get|post|put|delete)|HttpGet|HttpPost

        // Branch 1: Controller (controller classes)
        let code_with_controller = "export class UserController { }";
        assert!(CONTROLLER_REGEX.is_match(code_with_controller));

        // Branch 2: router. (router method calls)
        let code_with_router = "router.get('/users', (req, res) => { });";
        assert!(CONTROLLER_REGEX.is_match(code_with_router));

        // Branch 3: app.(get|post|put|delete) (Express app methods)
        assert!(CONTROLLER_REGEX.is_match("app.get('/api/users', handler);"));
        assert!(CONTROLLER_REGEX.is_match("app.post('/api/users', handler);"));
        assert!(CONTROLLER_REGEX.is_match("app.put('/api/users/:id', handler);"));
        assert!(CONTROLLER_REGEX.is_match("app.delete('/api/users/:id', handler);"));

        // Branch 4: HttpGet (HTTP decorators)
        assert!(CONTROLLER_REGEX.is_match("@HttpGet('/users')"));

        // Branch 5: HttpPost (HTTP decorators)
        assert!(CONTROLLER_REGEX.is_match("@HttpPost('/users')"));

        let code_without_controller = "const helpers = { format: () => {} };";
        assert!(!CONTROLLER_REGEX.is_match(code_without_controller));
    }

    #[test]
    fn test_api_regex() {
        // Test all branches: fetch\(|axios\.|HttpClient|apiClient\.

        // Branch 1: fetch( (Fetch API)
        let code_with_fetch = "const response = await fetch('/api/users');";
        assert!(API_REGEX.is_match(code_with_fetch));

        // Branch 2: axios. (Axios library)
        let code_with_axios = "const data = await axios.get('/api/data');";
        assert!(API_REGEX.is_match(code_with_axios));

        // Branch 3: HttpClient (HTTP client class)
        assert!(API_REGEX.is_match("const client = new HttpClient();"));
        assert!(API_REGEX.is_match("private readonly HttpClient httpClient;"));

        // Branch 4: apiClient. (custom API client)
        assert!(API_REGEX.is_match("const data = await apiClient.get('/users');"));
        assert!(API_REGEX.is_match("apiClient.post('/api/create', body);"));

        let code_without_api = "const result = processData(input);";
        assert!(!API_REGEX.is_match(code_without_api));
    }

    #[test]
    fn test_file_analysis_default() {
        let analysis = FileAnalysis::default();
        assert!(!analysis.has_try_catch);
        assert!(!analysis.has_async);
        assert!(!analysis.has_prisma);
        assert!(!analysis.has_controller);
        assert!(!analysis.has_api_call);
    }

    #[test]
    fn test_stats_default() {
        let stats = Stats::default();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.backend_files, 0);
        assert_eq!(stats.frontend_files, 0);
        assert_eq!(stats.database_files, 0);
        assert_eq!(stats.other_files, 0);
        assert_eq!(stats.async_files, 0);
        assert_eq!(stats.try_catch_files, 0);
        assert_eq!(stats.failed_files, 0);
    }

    #[test]
    fn test_get_file_category_windows_paths() {
        // Note: On Windows, Rust PathBuf automatically handles both / and \ as separators
        // On Unix, only / is treated as a separator, so we use forward slashes for cross-platform tests

        // Windows paths with forward slashes (works on all platforms)
        assert_eq!(
            get_file_category(&PathBuf::from("C:/project/frontend/App.tsx")),
            "frontend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("C:/project/controllers/UserController.ts")),
            "backend"
        );
        assert_eq!(
            get_file_category(&PathBuf::from("C:/project/database/schema.sql")),
            "database"
        );

        // UNC paths (Windows network paths)
        assert_eq!(
            get_file_category(&PathBuf::from("//storage/share/frontend/App.tsx")),
            "frontend"
        );
    }

    #[test]
    fn test_should_analyze_windows_paths() {
        // Note: Using forward slashes for cross-platform compatibility
        // On Windows, Rust automatically normalizes these

        // Windows paths with drive letters
        assert!(should_analyze(&PathBuf::from("C:/project/app.ts")));
        assert!(should_analyze(&PathBuf::from("C:/Users/dev/Component.tsx")));

        // Skip test files on Windows paths
        assert!(!should_analyze(&PathBuf::from("C:/project/app.test.ts")));
        assert!(!should_analyze(&PathBuf::from(
            "D:/code/Component.spec.tsx"
        )));

        // UNC paths (Windows network paths)
        assert!(should_analyze(&PathBuf::from("//storage/share/app.ts")));
        assert!(!should_analyze(&PathBuf::from(
            "//storage/share/app.test.ts"
        )));
    }
}

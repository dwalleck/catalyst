use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

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

fn should_analyze(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Skip test files, config files
    if path_lower.contains(".test.")
        || path_lower.contains(".spec.")
        || path_lower.contains(".config.")
        || path_lower.contains("/types/")
        || path_lower.ends_with(".json")
        || path_lower.ends_with(".md")
    {
        return false;
    }

    // Check for code files
    path_lower.ends_with(".ts")
        || path_lower.ends_with(".tsx")
        || path_lower.ends_with(".js")
        || path_lower.ends_with(".jsx")
        || path_lower.ends_with(".rs")
        || path_lower.ends_with(".cs")
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

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
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

    for entry in WalkDir::new(&args.directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        if !should_analyze(&path_str) {
            debug!("Skipping: {}", path_str);
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
            debug!("Analyzing: {} ({})", path_str, category);
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

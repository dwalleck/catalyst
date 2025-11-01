use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;
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

#[derive(Debug)]
struct FileAnalysis {
    has_try_catch: bool,
    has_async: bool,
    has_prisma: bool,
    has_controller: bool,
    has_api_call: bool,
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

fn analyze_file(path: &Path) -> io::Result<FileAnalysis> {
    let content = fs::read_to_string(path)?;

    // Use pre-compiled static regexes (10-100x faster than compiling on each call)
    Ok(FileAnalysis {
        has_try_catch: TRY_REGEX.is_match(&content),
        has_async: ASYNC_REGEX.is_match(&content),
        has_prisma: PRISMA_REGEX.is_match(&content),
        has_controller: CONTROLLER_REGEX.is_match(&content),
        has_api_call: API_REGEX.is_match(&content),
    })
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        eprintln!("\nAnalyzes files in directory for error-prone patterns");
        return Ok(());
    }

    let dir = &args[1];
    let start = Instant::now();

    println!("\n🔍 ANALYZING FILES IN: {}\n", dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut stats = Stats::default();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        if !should_analyze(&path_str) {
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

        if let Ok(analysis) = analyze_file(path) {
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
                println!(
                    "⚠️  {} - Async without try/catch",
                    path.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }

    let elapsed = start.elapsed();

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 ANALYSIS RESULTS\n");
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
    println!("\n⚡ Analysis completed in {:.2?}", elapsed);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
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

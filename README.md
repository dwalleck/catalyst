# Catalyst

**High-performance Rust hooks for Claude Code skill auto-activation**

Born from 6 months of production use, Catalyst provides battle-tested Rust hooks that make Claude Code skills activate automatically based on context. ~2ms startup time, zero runtime dependencies.

---

## Quick Start

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and install
./install.sh

# For SQLite-backed state management
./install.sh --sqlite
```

**That's it!** Binaries are installed to `~/.claude-hooks/bin/`

---

## What This Provides

###  **skill-activation-prompt** (~2ms)
Analyzes user prompts and automatically suggests relevant Claude Code skills based on:
- Keyword triggers
- File path patterns
- Intent matching
- File content analysis

### **file-analyzer** (~45ms for 1000 files)
Scans codebases for patterns:
- Async functions without try/catch
- Database usage
- API endpoints
- Framework-specific patterns

### **post-tool-use-tracker-sqlite** (optional)
SQLite-backed state management for tracking file modifications across sessions with rich query capabilities.

---

## Performance

| Metric | Performance |
|--------|-------------|
| **Startup Time** | ~2ms (60x faster than interpreted languages) |
| **Memory Usage** | 3-5MB (10x less than Node.js/Python) |
| **Binary Size** | 1.8-2.4MB (self-contained) |
| **Runtime** | None required (zero dependencies) |

**Why this matters:** The `skill-activation-prompt` hook runs on EVERY user prompt. At 2ms, it's imperceptible. At 100-200ms (TypeScript/Python), users notice lag.

---

## Architecture

```
catalyst/
├── src/bin/                    # Rust hook implementations
│   ├── skill_activation_prompt.rs
│   ├── file_analyzer.rs
│   └── post_tool_use_tracker_sqlite.rs
├── Cargo.toml                  # With idiomatic feature flags
├── install.sh                  # Standalone installer
├── docs/                       # Comprehensive documentation
│   ├── rust-hooks.md
│   ├── databases.md
│   ├── performance-comparison.md
│   └── standalone-installation.md
└── .claude/                    # Example Claude Code infrastructure
    ├── hooks/                  # Bash wrappers + optional hooks
    ├── skills/                 # 5 production skills
    └── agents/                 # 10 specialized agents
```

---

## Features

### Cargo Features

```bash
# Core hooks only (default)
cargo build --release

# With SQLite support
cargo build --release --features sqlite
```

**Features:**
- `sqlite` - Enables post-tool-use-tracker-sqlite with rich query capabilities

### Optimization Profile

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Remove debug symbols
```

Result: 1.8MB stripped binary with ~2ms startup.

---

## Installation

### Option 1: Standalone (Recommended)

**Build once, use everywhere:**

```bash
# From catalyst directory
./install.sh

# Or with SQLite support
./install.sh --sqlite
```

Installs to `~/.claude-hooks/bin/` for use across all projects.

### Option 2: Per-Project

**For customization:**

```bash
# Build in catalyst directory
cargo build --release

# Binaries in target/release/
```

---

## Usage

### In Your Claude Code Projects

Create thin wrappers in your project's `.claude/hooks/`:

```bash
cd your-project/.claude/hooks/

# Skill activation hook (essential)
cat > skill-activation-prompt.sh << 'EOF'
#!/bin/bash
cat | ~/.claude-hooks/bin/skill-activation-prompt
EOF
chmod +x skill-activation-prompt.sh
```

### Configuration

Add to `.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
          }
        ]
      }
    ]
  }
}
```

See **[docs/standalone-installation.md](docs/standalone-installation.md)** for complete setup.

---

## How It Works

### Skill Auto-Activation

```
User types prompt
    ↓
skill-activation-prompt (~2ms)
    ↓
Reads skill-rules.json
    ↓
Matches against:
  - Keywords ("backend", "API", "Prisma")
  - File paths (**/*.controller.ts)
  - Intent patterns (regex: "create.*controller")
  - File content (contains "prisma.")
    ↓
Injects skill suggestions
    ↓
Claude loads relevant skills
```

**The breakthrough:** Skills activate automatically based on context, not manual invocation.

---

## Development

### Building

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized)
cargo build --release

# With SQLite
cargo build --release --features sqlite

# Check code
cargo check
cargo clippy
cargo fmt
```

### Testing

```bash
# Test with sample input
echo '{
  "session_id": "test-123",
  "prompt": "create a backend controller",
  "cwd": "/project",
  "permission_mode": "normal"
}' | ./target/release/skill-activation-prompt
```

### Benchmarking

```bash
# Measure startup time
time ./target/release/skill-activation-prompt < test-input.json

# Statistical analysis
hyperfine './target/release/skill-activation-prompt < test-input.json'
```

---

## SQLite State Management

Enable rich analytics with the `sqlite` feature:

```bash
./install.sh --sqlite
```

**Capabilities:**
- Track file modifications across sessions
- Query patterns (async without try/catch)
- Generate statistics (files per category)
- 100x faster queries than file-based approaches

**Performance:**
- Insert 1000 records: 180ms
- Complex query: 0.8ms
- Aggregation: 2ms

See **[docs/databases.md](docs/databases.md)** for details.

---

## Cross-Platform

Build for multiple targets:

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS ARM (M1/M2)
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

---

## Documentation

- **[docs/rust-hooks.md](docs/rust-hooks.md)** - Detailed hook documentation
- **[docs/standalone-installation.md](docs/standalone-installation.md)** - Complete setup guide
- **[docs/performance-comparison.md](docs/performance-comparison.md)** - Performance analysis
- **[docs/databases.md](docs/databases.md)** - SQLite state management
- **[CLAUDE.md](CLAUDE.md)** - Integration guide for Claude Code
- **[PROJECT_DESIGN.md](PROJECT_DESIGN.md)** - Architecture documentation

---

## Example Claude Code Infrastructure

The `.claude/` directory contains production-tested examples:

### Skills (5)
- **backend-dev-guidelines** - Express/Prisma patterns
- **frontend-dev-guidelines** - React/MUI v7 patterns
- **skill-developer** - Meta-skill for creating skills
- **route-tester** - JWT auth testing patterns
- **error-tracking** - Sentry integration

### Agents (10)
- code-architecture-reviewer
- refactor-planner
- frontend-error-fixer
- documentation-architect
- And 6 more...

### Hooks (Bash)
- post-tool-use-tracker.sh - File tracking (no deps)
- tsc-check.sh - TypeScript compilation checks
- trigger-build-resolver.sh - Auto-fix build errors

**These are examples** - copy and adapt for your projects.

---

## Why Rust?

### Performance
- **~2ms startup** vs 100-200ms (Node.js/Python)
- **3-5MB memory** vs 30-50MB (Node.js/Python)
- **60x faster** for operations running hundreds of times

### Deployment
- **Single binary** - no runtime needed
- **1.8-2.4MB** - self-contained executable
- **Zero dependencies** - works anywhere

### Safety
- **No null pointers** - compiler prevents null errors
- **No data races** - ownership system prevents concurrency bugs
- **Memory safe** - no buffer overflows or use-after-free

### When Rust Matters

**Critical for:**
- UserPromptSubmit hooks (run on every prompt)
- High-frequency operations (hundreds per session)
- Large codebases (analyzing thousands of files)

**Less critical for:**
- PostToolUse hooks (run in background)
- Stop hooks (user already paused)
- One-time operations

**For this project:** The skill-activation-prompt hook runs on EVERY prompt. Rust's 2ms startup is imperceptible; 100-200ms would be noticeable lag.

---

## Real-World Usage

Extracted from managing:
- 6 TypeScript microservices
- 50,000+ lines of code
- React frontend with complex data grids
- Sophisticated workflow engine
- 6 months of daily Claude Code use

**These patterns solved real problems:**
- ✅ Skills now activate automatically
- ✅ No noticeable latency
- ✅ Zero dependency hell
- ✅ Works across all projects (standalone install)
- ✅ Rich analytics with SQLite

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Test with sample inputs
6. Submit a pull request

**Code standards:**
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Clear comments and meaningful names

---

## License

MIT License - Use freely in your projects, commercial or personal.

---

## See Also

- [Claude Code Documentation](https://docs.claude.com/claude-code)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

**Start here:** Run `./install.sh` and see the auto-activation magic happen.

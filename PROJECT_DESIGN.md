# Claude Code Infrastructure - Project Design Document

**Version:** 2.0
**Last Updated:** 2025-10-30
**Status:** Production-Ready Reference Implementation

---

## Executive Summary

**Claude Code Infrastructure** is a comprehensive reference library of production-tested patterns for extending Claude Code through skills, hooks, agents, and slash commands. Born from 6 months of real-world use on a TypeScript microservices project, it provides high-performance Rust hooks for maximum performance and zero runtime dependencies.

**Key Innovation:** Auto-activating skills via Rust hooks that monitor user prompts and file context, eliminating the need to manually invoke skills.

---

## Project Vision

### Mission Statement

Provide developers with battle-tested infrastructure that makes Claude Code's skills system practical for production use through:

1. Automatic skill activation based on context
2. High-performance Rust implementation
3. Zero runtime dependencies
4. Production-ready patterns and best practices

### Goals

**Primary Goals:**

- ✅ Solve the skill auto-activation problem
- ✅ Provide maximum performance Rust hooks (~2ms startup)
- ✅ Enable zero-dependency deployment
- ✅ Maintain production-quality code as reference
- ✅ Support optional SQLite state management

**Secondary Goals:**

- ✅ Document dev docs pattern for context preservation
- ✅ Showcase specialized agents for complex tasks
- ✅ Provide integration guides for different project structures
- ✅ Build community around Claude Code infrastructure

### Non-Goals

- ❌ **Not** a working application or starter template
- ❌ **Not** framework-specific (examples are generic)
- ❌ **Not** production deployment (reference only)

---

## Architecture Overview

### System Components

```
Claude Code Infrastructure
├── Skills (5)              # Domain knowledge loaded on-demand
├── Hooks (Rust)           # High-performance event-driven automation
├── Agents (10)            # Specialized task handlers
├── Commands (3)           # Slash commands for workflows
└── Dev Docs Pattern       # Context preservation methodology
```

### Core Architecture Principles

**1. Performance First**

- Rust implementation for imperceptible latency (~2ms)
- Zero runtime dependencies (single binary)
- Minimal memory footprint (3-5MB)
- Optional SQLite for rich queries

**2. Progressive Disclosure**

- Skills use 500-line rule (main file + resource files)
- Load overview first, details only when needed
- Prevents context limit issues

**3. Modular & Composable**

- Each component works independently
- Users can mix and match
- No tight coupling between components

**4. Production-Tested**

- All patterns extracted from real-world use
- Generic examples (blog domain) for broad applicability
- Documented edge cases and gotchas

---

## Component Design

### 1. Skills System

**Purpose:** Provide domain-specific knowledge that Claude loads when relevant.

**Architecture:**

```
skill-name/
├── SKILL.md                 # Main file (<500 lines)
│   ├── Purpose
│   ├── When to Use
│   ├── Quick Reference
│   └── Resource Navigation
└── resources/               # Progressive disclosure
    ├── topic-1.md
    ├── topic-2.md
    └── topic-3.md
```

**Auto-Activation Mechanism:**

```
skill-rules.json defines:
├── Keyword triggers      ("backend", "API", "Prisma")
├── Intent patterns       (regex: "create.*controller")
├── File path patterns    (**/*.controller.ts)
└── Content patterns      (file contains "prisma.")

UserPromptSubmit hook (Rust):
1. Reads skill-rules.json
2. Matches prompt against triggers
3. Checks file context
4. Injects skill suggestions
```

**Implemented Skills:**

1. **backend-dev-guidelines** - Express/Prisma/TypeScript patterns
2. **frontend-dev-guidelines** - React/MUI v7/TanStack patterns
3. **skill-developer** - Meta-skill for creating skills
4. **route-tester** - JWT cookie auth testing
5. **error-tracking** - Sentry integration patterns

**Design Decisions:**

- ✅ 500-line limit prevents context overflow
- ✅ Generic blog domain (Post/Comment/User) for broad applicability
- ✅ Framework-specific to provide concrete examples
- ⚠️ Users must adapt for different tech stacks

---

### 2. Hooks System

**Purpose:** Enable event-driven automation at specific lifecycle points.

**Hook Events:**

- **UserPromptSubmit** - Before processing user input
- **PreToolUse** - Before tool execution
- **PostToolUse** - After tool completion
- **Stop** - When user requests stop

**Essential Hooks:**

**skill-activation-prompt (UserPromptSubmit)**

- Analyzes user prompt
- Checks file context
- Matches against skill-rules.json
- Suggests relevant skills
- **Implementation:** Rust (maximum performance)

**post-tool-use-tracker (PostToolUse)**

- Tracks file modifications
- Records tool usage
- Builds session context
- Auto-detects project structure
- **Implementation:** Bash (simple and effective)

**Architecture: Rust Implementation**

**Performance Characteristics:**

| Metric | Rust Performance |
|--------|-----------------|
| **Startup Time** | ~2ms |
| **Memory Usage** | 3-5MB |
| **Binary Size** | 1.8-2.4MB |
| **Runtime Required** | None (zero dependencies) |

**Deployment Model: Standalone Installation**

```
~/.claude-hooks/
├── bin/
│   ├── skill-activation-prompt  # 1.8MB binary
│   └── file-analyzer
└── src/  # Source for updates

project/.claude/hooks/
└── skill-activation-prompt.sh  # 50 byte wrapper
```

**Advantages:**

- ✅ Compile once (45s), use everywhere (0s per project)
- ✅ Update in one place, all projects benefit
- ✅ Tiny per-project footprint (50 bytes vs 2MB)
- ✅ Consistent version across all projects

---

### 3. Agents System

**Purpose:** Handle complex, multi-step tasks autonomously.

**Architecture:**

- Standalone markdown files
- No dependencies on skills or hooks
- Launched via Task tool
- Run in subprocess with own context

**Categories:**

**Code Quality:**

- code-architecture-reviewer
- code-refactor-master
- plan-reviewer
- refactor-planner

**Debugging:**

- frontend-error-fixer
- auth-route-debugger
- auto-error-resolver

**Development:**

- documentation-architect
- auth-route-tester
- web-research-specialist

**Design Principles:**

- ✅ Stateless - each invocation is fresh
- ✅ Focused - one task per agent
- ✅ Autonomous - runs without user input
- ✅ Reporting - returns comprehensive results

---

### 4. Dev Docs Pattern

**Purpose:** Preserve context across Claude Code session resets.

**Three-File Structure:**

```
dev/active/[task-name]/
├── [task-name]-plan.md      # Strategic plan with phases
├── [task-name]-context.md   # Current state + decisions
└── [task-name]-tasks.md     # Checklist format
```

**Workflow:**

1. `/dev-docs` creates initial structure
2. Work proceeds, updating context
3. `/dev-docs-update` before context reset
4. New session reads all three files
5. Resume exactly where left off

**Design Decisions:**

- ✅ Three files balance detail vs readability
- ✅ Markdown for version control
- ✅ Slash commands automate creation
- ⚠️ Requires discipline to update

---

## Technical Specifications

### Rust Implementation

**File Structure:**

```
catalyst/
├── src/bin/
│   ├── skill_activation_prompt.rs
│   ├── file_analyzer.rs
│   └── post_tool_use_tracker_sqlite.rs
├── Cargo.toml           # With idiomatic features
├── install.sh
└── docs/                # Documentation
```

**Dependencies:**

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
walkdir = "2.4"

# Optional: For SQLite version
rusqlite = { version = "0.31", features = ["bundled"] }
```

**Execution Model:**

- Compile to native binary
- Install to `~/.claude-hooks/bin/`
- Per-project wrapper calls binary
- ~2ms startup time

**Performance:**

**Startup Time:** 2-3ms
**Memory Usage:** 3-5MB
**Binary Size:** 1.8-2.4MB stripped

**Pros:**

- Imperceptible latency
- No runtime needed
- Minimal memory
- Maximum safety (no null pointers, no data races)

**Cons:**

- Requires Rust for building from source
- 45s compile time
- Steeper learning curve for modifications

---

## State Management Design

### Problem Statement

Hooks need to track file modifications across a session to:

- Analyze patterns (async without try/catch)
- Generate statistics (files modified per category)
- Provide context-aware reminders
- Enable analytics and reporting

### Solutions Comparison

#### Option 1: File-Based (Current for post-tool-use-tracker)

**Implementation:**

```bash
# edited-files.log
2024-10-30T14:30:00Z    Edit    src/controllers/UserController.ts
2024-10-30T14:35:12Z    Write   src/services/userService.ts
```

**Pros:**

- ✅ Simplest implementation
- ✅ Fastest writes (append only)
- ✅ Human readable
- ✅ No dependencies

**Cons:**

- ❌ O(n) query time (must scan entire file)
- ❌ No structured queries
- ❌ Manual parsing required

**Verdict:** Good for simple tracking, adequate for current needs.

---

#### Option 2: SQLite (Rust)

**Implementation:**

```rust
let conn = Connection::open(&db_path)?;

// Schema with indexes
conn.execute("CREATE TABLE files (...)", [])?;
conn.execute("CREATE INDEX idx_session_category
              ON files(session_id, category)", [])?;

// Fast indexed queries
let mut stmt = conn.prepare(
    "SELECT * FROM files
     WHERE session_id = ?1
     AND has_async = 1
     AND has_try_catch = 0"
)?;
```

**Pros:**

- ✅ 100x faster queries than file-based
- ✅ Smallest database files
- ✅ Best query performance
- ✅ sqlite3 CLI for inspection
- ✅ Battle-tested (used everywhere)

**Cons:**

- ❌ More complex setup
- ❌ Requires Cargo-sqlite.toml configuration

**Verdict:** Best for analytics and complex queries.

---

### Performance Benchmarks

Testing 1000 file modifications:

| Operation | File-Based | SQLite (Rust) |
|-----------|-----------|---------------|
| **Insert 1000** | 50ms | 180ms |
| **Find by category** | 80ms | 0.8ms |
| **Complex query** | 300ms+ | 1.2ms |
| **Aggregation** | 500ms+ | 2ms |

**Conclusion:** SQLite is 100-250x faster for queries, worth the complexity for analytics use cases.

---

## Integration Patterns

### Monorepo Pattern

**skill-rules.json:**

```json
{
  "backend-dev-guidelines": {
    "fileTriggers": {
      "pathPatterns": [
        "services/*/src/**/*.ts",
        "packages/api/src/**/*.ts"
      ]
    }
  }
}
```

**Hook Detection:**

```rust
fn detect_repo(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts[0] == "services" && parts.len() > 1 {
        return parts[1].to_string();
    }
    if parts[0] == "packages" && parts.len() > 1 {
        return parts[1].to_string();
    }
    "root".to_string()
}
```

---

### Single-App Pattern

**skill-rules.json:**

```json
{
  "backend-dev-guidelines": {
    "fileTriggers": {
      "pathPatterns": [
        "src/**/*.ts",
        "backend/**/*.ts"
      ]
    }
  }
}
```

---

## Development Workflow

### For Skill Developers

**Adding a New Skill:**

1. Create skill directory
2. Write SKILL.md (<500 lines)
3. Create resource files for details
4. Add to skill-rules.json
5. Test activation triggers
6. Document in README

**Best Practices:**

- Use generic domain examples
- Include both what to do and what to avoid
- Provide code snippets
- Reference resource files for depth
- Keep main file under 500 lines

---

### For Hook Developers

**Modifying Rust Hooks:**

1. Edit source in `src/bin/`
2. Test with `cargo run --bin <hook-name>`
3. Build release: `cargo build --release`
4. Reinstall: `./install.sh`
5. Test in real project

**Adding New Functionality:**

```rust
// Example: Add new pattern matching
fn check_pattern(content: &str) -> bool {
    let pattern = Regex::new(r"your_pattern").unwrap();
    pattern.is_match(content)
}
```

---

## Testing Strategy

### Hook Testing

**Unit Tests:**

```bash
# Test with sample input
echo '{
  "session_id": "test-123",
  "prompt": "create a backend controller"
}' | ./target/release/skill-activation-prompt

# Expected output:
# 📚 RECOMMENDED SKILLS:
#   → backend-dev-guidelines
```

**Performance Tests:**

```bash
# Benchmark startup time
time ./target/release/skill-activation-prompt < input.json

# Should be <5ms
```

---

### Skill Testing

**Activation Tests:**

```json
{
  "test": "keyword trigger",
  "prompt": "help with backend API",
  "expected": ["backend-dev-guidelines"]
}
```

**Content Tests:**

- Verify main file < 500 lines
- Check resource files exist
- Validate markdown syntax
- Test progressive disclosure

---

## Performance Requirements

### Hook Performance

**UserPromptSubmit Hooks:**

- Target: <50ms (imperceptible to user)
- Achieved: ~2ms with Rust ✅

**PostToolUse/Stop Hooks:**

- More lenient (run in background or when paused)
- File-based implementation acceptable

---

### Database Performance (SQLite)

**Query Requirements:**

- Simple queries: <5ms
- Complex queries: <20ms
- Aggregations: <50ms

**Achieved with Rust + SQLite:**

- SQLite (Rust): 0.8ms - 2ms ✅

---

## Security Considerations

### Input Validation

**All hooks must:**

- Validate JSON input structure
- Sanitize file paths (prevent traversal)
- Limit input size (prevent DoS)
- Handle malformed input gracefully

**Example:**

```rust
// Validate session ID format
if !session_id.chars().all(|c| c.is_alphanumeric() || c == '-') {
    return Err("Invalid session ID");
}

// Prevent path traversal
let canonical = fs::canonicalize(&file_path)?;
if !canonical.starts_with(&project_dir) {
    return Err("Path outside project");
}
```

---

### File System Access

**Hooks should:**

- Only read from `$CLAUDE_PROJECT_DIR`
- Only write to `~/.claude/hooks-state/`
- Validate all file paths
- Use canonical paths

**Never:**

- Write to arbitrary locations
- Execute user-provided commands
- Read sensitive files (/.ssh/, /etc/)

---

### Database Security (SQLite)

**Best Practices:**

- Use parameterized queries (prevent injection)
- Validate input before queries
- Limit database size (prevent disk fill)
- Auto-cleanup old sessions

**Example:**

```rust
// Good: Parameterized query
conn.execute(
    "INSERT INTO files (session_id, path) VALUES (?1, ?2)",
    params![session_id, path]
)?;

// Bad: String interpolation (SQL injection risk)
// conn.execute(&format!("INSERT INTO files VALUES ('{}')", session_id))?;
```

---

## Deployment Strategy

### Distribution Channels

**1. GitHub Repository** (Primary)

- Complete source code
- Pre-built binaries (GitHub Releases)
- Documentation and examples
- Issue tracking

**2. Installation Script**

```bash
# From catalyst directory
./install.sh
# Installs to ~/.claude-hooks/bin/

# With SQLite support
./install.sh --sqlite
```

---

## Future Roadmap

### Phase 1: Foundation (Complete ✅)

- [x] Rust hook implementation
- [x] SQLite state management option
- [x] Comprehensive documentation
- [x] Production-tested patterns

### Phase 2: Ecosystem (In Progress)

- [ ] Pre-built binaries for all platforms
- [ ] Homebrew formula for macOS
- [ ] Community contributions

### Phase 3: Advanced Features

- [ ] Web-based dashboard for session analytics
- [ ] Cross-session pattern detection
- [ ] Team collaboration features

---

## Success Metrics

### Adoption Metrics

- GitHub stars and forks
- Download counts (releases)
- Community contributions

### Quality Metrics

- Issue resolution time
- Test coverage
- Performance benchmarks
- Documentation completeness

### Impact Metrics

- User testimonials
- Production deployments
- Derived projects

---

## Contributing Guidelines

### Code Contributions

**Before Contributing:**

1. Check existing issues
2. Discuss major changes first
3. Follow Rust coding standards
4. Add tests
5. Update documentation

**Code Standards:**

- Rust: rustfmt + clippy
- Clear comments, meaningful names

---

## License

MIT License - Permissive, allows commercial use.

**Rationale:**

- Maximize adoption
- Allow commercial use
- Enable derivatives
- Minimal restrictions

---

## Appendix

### Glossary

**Skill** - Domain knowledge module loaded on-demand
**Hook** - Event-driven script executed at lifecycle points
**Agent** - Autonomous task handler for complex operations
**Progressive Disclosure** - Loading details only when needed
**Dev Docs** - Three-file context preservation pattern

### References

- [Claude Code Documentation](https://docs.claude.com/claude-code)
- [Rust Hooks Documentation](./docs/rust-hooks.md)
- [Performance Comparison](./docs/performance-comparison.md)
- [SQLite Databases](./docs/databases.md)
- [Standalone Installation](./docs/standalone-installation.md)
- [SQLite Performance Tips](https://www.sqlite.org/fasterthanfs.html)
- [Rust Embedded Databases](https://lib.rs/database)

---

**Document History:**

- 2025-10-30: Version 2.0 - Rust-only implementation
- 2025-10-30: Version 1.0 - Initial version with multiple language implementations

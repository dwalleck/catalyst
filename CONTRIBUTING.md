# Contributing to Claude Code Infrastructure

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Contribution Workflow](#contribution-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Community](#community)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive experience for everyone. We expect all contributors to:

- ✅ Be respectful and considerate
- ✅ Welcome newcomers and help them learn
- ✅ Focus on what's best for the community
- ✅ Show empathy towards others
- ❌ No harassment, discrimination, or trolling

### Enforcement

Violations can be reported to project maintainers. All reports will be reviewed and investigated promptly.

---

## How Can I Contribute?

### Reporting Bugs

**Before submitting a bug report:**
1. Check existing issues to avoid duplicates
2. Try to reproduce with latest version
3. Gather relevant information

**Bug Report Template:**
```markdown
**Description:**
Brief description of the issue

**Steps to Reproduce:**
1. Step one
2. Step two
3. ...

**Expected Behavior:**
What should happen

**Actual Behavior:**
What actually happens

**Environment:**
- OS: [e.g., Linux, macOS, Windows]
- Language/Runtime: [e.g., Rust 1.70, Node.js 18, .NET 8]
- Implementation: [e.g., TypeScript hooks, Rust + SQLite]

**Additional Context:**
Any other relevant information
```

---

### Suggesting Features

**Before suggesting a feature:**
1. Check if it's already proposed
2. Consider if it fits the project vision
3. Think about implementation complexity

**Feature Request Template:**
```markdown
**Problem Statement:**
What problem does this solve?

**Proposed Solution:**
How would this work?

**Alternatives Considered:**
Other approaches you've thought about

**Additional Context:**
Use cases, examples, mockups
```

---

### Contributing Code

**Great first contributions:**
- 🐛 Bug fixes
- 📝 Documentation improvements
- ✨ Small feature additions
- 🧪 Additional tests
- 🌍 Translations

**Larger contributions:**
- New language implementations
- Database backends
- New skills or agents
- Performance optimizations

---

## Development Setup

### Prerequisites

Choose based on what you're working on:

**TypeScript Development:**
```bash
# Node.js 18+ required
node --version

# Install dependencies
cd .claude/hooks
npm install
```

**C# Development:**
```bash
# .NET 8+ required
dotnet --version

# Restore packages
cd .claude/hooks/LiteDbHooks
dotnet restore
```

**Rust Development:**
```bash
# Rust 1.70+ required
rustc --version

# Build
cd .claude/hooks/RustHooks
cargo build
```

---

### Repository Structure

```
claude-code-infrastructure/
├── .claude/
│   ├── skills/          # Skill implementations
│   ├── hooks/           # Hook implementations
│   │   ├── *.ts         # TypeScript hooks
│   │   ├── *.cs         # C# hooks
│   │   ├── LiteDbHooks/ # C# + LiteDB
│   │   └── RustHooks/   # Rust hooks
│   ├── agents/          # Agent definitions
│   └── commands/        # Slash commands
├── dev/                 # Dev docs examples
├── docs/                # Documentation
└── tests/               # Test suite
```

---

## Contribution Workflow

### 1. Fork and Clone

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR-USERNAME/claude-code-infrastructure.git
cd claude-code-infrastructure
```

### 2. Create a Branch

```bash
# Use descriptive branch names
git checkout -b feature/add-python-hooks
git checkout -b fix/litedb-query-bug
git checkout -b docs/improve-rust-readme
```

### 3. Make Changes

**Follow these principles:**
- One feature/fix per branch
- Keep commits focused and atomic
- Write clear commit messages
- Add tests for new functionality
- Update documentation

### 4. Test Your Changes

```bash
# TypeScript
cd .claude/hooks
npm test

# C#
cd .claude/hooks/LiteDbHooks
dotnet test

# Rust
cd .claude/hooks/RustHooks
cargo test

# Integration test
./test-integration.sh
```

### 5. Commit Changes

**Commit Message Format:**
```
type(scope): Short description

Longer explanation if needed.

Fixes #123
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `perf`: Performance improvement

**Examples:**
```bash
git commit -m "feat(rust): Add PostgreSQL backend support"
git commit -m "fix(litedb): Resolve query timeout on large datasets"
git commit -m "docs(readme): Update installation instructions"
```

### 6. Push and Create PR

```bash
git push origin feature/add-python-hooks
```

Then create a Pull Request on GitHub with:
- Clear title describing the change
- Description of what and why
- Reference to related issues
- Screenshots if UI-related
- Checklist of what was done

**PR Template:**
```markdown
## Description
Brief description of changes

## Related Issues
Fixes #123

## Changes Made
- Added X
- Updated Y
- Fixed Z

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Documentation
- [ ] README updated
- [ ] Code comments added
- [ ] Examples updated

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

---

## Coding Standards

### TypeScript

**Style:**
```typescript
// Use ESLint + Prettier
// 2-space indentation
// Semicolons required
// Use TypeScript types, not any

// Good
interface HookInput {
    sessionId: string;
    prompt: string;
}

function processHook(input: HookInput): string[] {
    return matchSkills(input);
}

// Bad
function processHook(input: any) {
    return matchSkills(input);
}
```

**File Organization:**
```typescript
// 1. Imports
import { readFileSync } from 'fs';

// 2. Types/Interfaces
interface Config { ... }

// 3. Constants
const MAX_RESULTS = 10;

// 4. Helper functions
function helper() { ... }

// 5. Main logic
async function main() { ... }

// 6. Entry point
main().catch(handleError);
```

---

### C#

**Style:**
```csharp
// Use .editorconfig
// PascalCase for public members
// camelCase for private fields
// Use records for DTOs

// Good
public record FileModification(
    string SessionId,
    string FilePath,
    DateTime Timestamp
);

public class DatabaseService
{
    private readonly LiteDatabase _db;

    public void TrackFile(FileModification file)
    {
        _db.GetCollection<FileModification>("files").Insert(file);
    }
}

// Bad
public class databaseService
{
    public LiteDatabase db;

    public void trackfile(FileModification file) { ... }
}
```

**Async Patterns:**
```csharp
// Prefer async/await
public async Task<List<FileModification>> GetFilesAsync(string sessionId)
{
    return await Task.Run(() =>
        _db.GetCollection<FileModification>("files")
           .Find(x => x.SessionId == sessionId)
           .ToList()
    );
}
```

---

### Rust

**Style:**
```rust
// Use rustfmt + clippy
// snake_case for functions/variables
// PascalCase for types
// Use Result<T, E> for errors

// Good
pub struct FileModification {
    pub session_id: String,
    pub file_path: PathBuf,
    pub timestamp: DateTime<Utc>,
}

pub fn track_modification(file: &FileModification) -> Result<(), Error> {
    validate_file(&file.file_path)?;
    save_to_database(file)?;
    Ok(())
}

// Bad
pub fn TrackModification(file: &FileModification) {
    // Using unwrap() for error handling
    validate_file(&file.file_path).unwrap();
}
```

**Error Handling:**
```rust
// Use ? operator for propagation
fn load_config() -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string("config.json")?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}

// Avoid unwrap() in production code
// Use unwrap_or(), unwrap_or_else(), or ? instead
```

---

## Testing Guidelines

### Unit Tests

**TypeScript:**
```typescript
// use Jest or similar
describe('skill matching', () => {
    it('should match backend keywords', () => {
        const prompt = 'create a backend controller';
        const matches = matchSkills(prompt);
        expect(matches).toContain('backend-dev-guidelines');
    });

    it('should handle empty prompt', () => {
        const matches = matchSkills('');
        expect(matches).toEqual([]);
    });
});
```

**C#:**
```csharp
[Fact]
public void TrackModification_ValidFile_InsertsRecord()
{
    // Arrange
    var db = new DatabaseService("test.db");
    var file = new FileModification("session1", "test.ts", DateTime.UtcNow);

    // Act
    db.TrackModification(file);

    // Assert
    var files = db.GetFiles("session1");
    Assert.Single(files);
    Assert.Equal("test.ts", files[0].FilePath);
}
```

**Rust:**
```rust
#[test]
fn test_file_categorization() {
    assert_eq!(
        get_file_category("src/controllers/UserController.ts"),
        "backend"
    );

    assert_eq!(
        get_file_category("src/components/Button.tsx"),
        "frontend"
    );
}
```

---

### Integration Tests

**Test End-to-End Flow:**
```bash
#!/bin/bash
# test-integration.sh

# Test skill activation
echo '{
  "session_id": "test-123",
  "prompt": "create a backend API"
}' | ./hooks/skill-activation-prompt.sh > output.txt

# Verify output
if grep -q "backend-dev-guidelines" output.txt; then
    echo "✅ Skill activation works"
else
    echo "❌ Skill activation failed"
    exit 1
fi
```

---

### Performance Tests

**Benchmark Critical Paths:**
```rust
#[bench]
fn bench_skill_matching(b: &mut Bencher) {
    let prompt = "create a backend controller";
    b.iter(|| {
        match_skills(prompt)
    });
}
```

**Performance Requirements:**
- UserPromptSubmit hooks: <50ms (target), <200ms (acceptable)
- Database queries: <5ms
- File analysis: <100ms per file

---

## Documentation

### Code Comments

**When to Comment:**
- ✅ Complex algorithms
- ✅ Non-obvious design decisions
- ✅ Public APIs
- ✅ Workarounds or hacks
- ❌ Obvious code (let code be self-documenting)

**Examples:**
```typescript
// Good: Explains why
// We use a Set to deduplicate because skill-rules.json may have
// overlapping triggers, and we want to suggest each skill only once
const uniqueSkills = new Set(matchedSkills);

// Bad: States the obvious
// Add skill to the set
uniqueSkills.add(skill);
```

---

### README Files

**Every component should have:**
- Purpose and use cases
- Installation instructions
- Usage examples
- Configuration options
- Troubleshooting section

**Template:**
```markdown
# Component Name

Brief description of what it does.

## Purpose

Why this component exists.

## Installation

How to install and configure.

## Usage

Basic examples.

## Advanced

Complex use cases.

## Troubleshooting

Common issues and solutions.
```

---

### API Documentation

**For libraries/shared code:**
- Document all public functions
- Include examples
- List parameters and return types
- Note any side effects

**TypeScript (JSDoc):**
```typescript
/**
 * Matches user prompt against skill triggers.
 *
 * @param prompt - The user's input text
 * @param rules - Skill rules configuration
 * @returns Array of matched skill names
 *
 * @example
 * ```ts
 * const matches = matchSkills("create API", rules);
 * // ['backend-dev-guidelines']
 * ```
 */
export function matchSkills(prompt: string, rules: SkillRules): string[] {
    // Implementation
}
```

---

## Community

### Getting Help

**For Questions:**
- GitHub Discussions for general questions
- Issues for bugs and features
- Discord/Slack (if available)

**Before Asking:**
- Search existing issues/discussions
- Check documentation
- Try to reproduce with minimal example

---

### Reviewing Pull Requests

**Guidelines for Reviewers:**
- Be respectful and constructive
- Focus on code, not people
- Ask questions rather than demand changes
- Suggest improvements with examples
- Approve quickly when ready

**What to Review:**
- Code quality and style
- Test coverage
- Documentation updates
- Performance impact
- Breaking changes

---

### Recognition

**Contributors are recognized through:**
- GitHub contributors list
- CONTRIBUTORS.md file
- Release notes mentions
- Project credits

**Significant contributions may earn:**
- Collaborator status
- Maintainer role
- Project decision input

---

## Release Process

**For Maintainers:**

### 1. Version Bump

```bash
# Update version in all files
# - package.json (TypeScript)
# - *.csproj (C#)
# - Cargo.toml (Rust)
# - CHANGELOG.md

# Tag release
git tag -a v1.2.0 -m "Release version 1.2.0"
git push origin v1.2.0
```

### 2. Build Artifacts

```bash
# Build all language implementations
./scripts/build-release.sh

# Creates:
# - TypeScript tarball
# - C# NuGet packages
# - Rust binaries for all platforms
```

### 3. Create GitHub Release

```bash
gh release create v1.2.0 \
    --title "Version 1.2.0" \
    --notes-file CHANGELOG.md \
    dist/*
```

### 4. Update Documentation

- Update README.md with new features
- Add migration guide if breaking changes
- Update examples
- Announce in community channels

---

## Questions?

- 📧 Open an issue for public questions
- 💬 Use GitHub Discussions for brainstorming
- 🐛 Report bugs via Issues
- ✨ Suggest features via Issues

---

Thank you for contributing to Claude Code Infrastructure! 🎉

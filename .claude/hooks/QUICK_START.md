# Hook Implementation Quick Start Guide

**Choose your implementation in 60 seconds.**

---

## Decision Tree

```
START: What's your priority?

├─ Fast Development
│  └─ Know JavaScript/TypeScript?
│     ├─ YES → Use TypeScript ✅
│     └─ NO  → Use C# (single-file)
│
├─ Maximum Performance
│  └─ Know Rust?
│     ├─ YES → Use Rust + SQLite ✅
│     └─ NO  → Use C# + Native AOT
│
├─ Need Database/Analytics
│  └─ Know Rust?
│     ├─ YES → Use Rust + SQLite ✅ (4x faster)
│     └─ NO  → Use C# + LiteDB ✅ (easier)
│
└─ Team uses .NET
   └─ Need database?
      ├─ YES → Use C# + LiteDB ✅
      └─ NO  → Use C# (single-file) ✅
```

---

## Quick Comparison

| If you need... | Use this | Why |
|----------------|----------|-----|
| **Fastest to write** | TypeScript | 45 min development time |
| **Fastest execution** | Rust | 2ms startup vs 135ms+ |
| **Best queries** | Rust + SQLite | 100x faster than grep |
| **Familiar tech (.NET)** | C# + LiteDB | Your team knows it |
| **Single binary** | Rust | No runtime needed |
| **npm ecosystem** | TypeScript | Access to all npm packages |

---

## Implementation Paths

### Path 1: TypeScript (Recommended for Beginners)

**Setup time:** 5 minutes

```bash
# Already have Node.js installed? You're done!
cd .claude/hooks
npm install

# Use existing hooks
chmod +x skill-activation-prompt.sh
```

**Pros:** Easiest, fastest development
**Cons:** Slowest execution (~135ms)

---

### Path 2: C# Single-File

**Setup time:** 10 minutes

```bash
# Install .NET SDK (if needed)
# See: https://dotnet.microsoft.com/download

# Use single-file hooks (no project needed!)
dotnet run skill-activation-prompt.cs
```

**Pros:** Strong typing, familiar to .NET devs
**Cons:** Requires .NET runtime

---

### Path 3: C# + LiteDB

**Setup time:** 15 minutes

```bash
cd .claude/hooks/LiteDbHooks

# Restore dependencies
dotnet restore

# Build
dotnet build --configuration Release
```

**Pros:** Rich queries, analytics, LINQ
**Cons:** Slower than Rust options

---

### Path 4: Rust

**Setup time:** 20 minutes (first time only)

```bash
# Install Rust (one-time)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build hooks
cd .claude/hooks/RustHooks
cargo build --release

# Binaries in: target/release/
```

**Pros:** 60x faster than TypeScript, smallest binary
**Cons:** Longer compile times, steeper learning curve

---

### Path 5: Rust + SQLite

**Setup time:** 20 minutes (first time only)

```bash
# Same as Rust, but use different Cargo.toml
cd .claude/hooks/RustHooks
cp Cargo-sqlite.toml Cargo.toml

cargo build --release
```

**Pros:** Best performance + rich queries
**Cons:** Most complex to implement

---

## Example: Setting Up Your First Hook

### TypeScript Example

```bash
# 1. Copy hook files (already done in showcase)
# 2. Make executable
chmod +x .claude/hooks/skill-activation-prompt.sh

# 3. Update settings.json
{
  "hooks": {
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/skill-activation-prompt.sh"
      }]
    }]
  }
}

# 4. Test
echo '{"prompt":"help with backend","session_id":"test"}' | \
  ./.claude/hooks/skill-activation-prompt.sh
```

---

### Rust Example

```bash
# 1. Build once
cd .claude/hooks/RustHooks
cargo build --release

# 2. Create wrapper script
cat > ../skill-activation-prompt-rust.sh << 'EOF'
#!/bin/bash
set -e
HOOK_DIR="$CLAUDE_PROJECT_DIR/.claude/hooks/RustHooks"
cat | "$HOOK_DIR/target/release/skill-activation-prompt"
EOF

chmod +x ../skill-activation-prompt-rust.sh

# 3. Update settings.json (same as TypeScript but different path)
# 4. Test
echo '{"prompt":"help with backend","session_id":"test"}' | \
  ./skill-activation-prompt-rust.sh
```

---

## Performance at a Glance

### Startup Times (Lower is Better)

```
Rust             ████ 2ms
Rust + SQLite    █████ 3ms
C# Native AOT    ███████████████████ 18ms
TypeScript       ████████████████████████████████████████████████ 135ms
C# dotnet run    ███████████████████████████████████████████████████████████████████ 245ms
```

### Query Performance (Lower is Better)

```
Rust + SQLite    █ 0.8ms
C# + LiteDB      ██ 3ms
File-based grep  ████████████████████████████████████████ 80ms
```

---

## Migration Path

**Start Simple → Add Power When Needed**

```
Week 1: TypeScript
  ↓
  "This is easy! Let me try C# for better types"
  ↓
Week 2: C# single-file
  ↓
  "I need analytics, let me add LiteDB"
  ↓
Week 3: C# + LiteDB
  ↓
  "Hooks feel slow, profile shows it's a bottleneck"
  ↓
Week 4: Rust + SQLite (only if needed!)
```

**Most users stop at Week 1-2.** Only optimize if profiling shows it's needed.

---

## Common Mistakes to Avoid

### ❌ DON'T: Jump to Rust immediately

**Why:** Development is 2-3x slower. Start simple.

**Instead:** Use TypeScript or C#, measure performance, optimize if needed.

---

### ❌ DON'T: Use `dotnet run` for UserPromptSubmit hooks

**Why:** 245ms startup is noticeable to users.

**Instead:** Pre-compile or use Rust for frequently-run hooks.

---

### ❌ DON'T: Use file-based tracking for complex queries

**Why:** 10-400x slower than database queries.

**Instead:** Use LiteDB or SQLite if you need to query patterns.

---

### ❌ DON'T: Optimize before measuring

**Why:** Premature optimization wastes time.

**Instead:** Profile first. TypeScript is often "fast enough."

---

## When to Use Each

### TypeScript
✅ Prototyping
✅ Node.js projects
✅ Team doesn't know Rust/C#
✅ Hook runs infrequently
❌ Performance-critical hooks
❌ Need compiled binary

### C# Single-File
✅ .NET teams
✅ Want strong typing
✅ Simple hooks
❌ Need fastest startup
❌ Need smallest binary

### C# + LiteDB
✅ Need analytics
✅ .NET teams
✅ Familiar with LINQ
❌ Need maximum performance
❌ Want smallest database

### Rust
✅ Performance critical
✅ No database needed
✅ Want single binary
✅ Team knows Rust
❌ Rapid prototyping
❌ Learning Rust

### Rust + SQLite
✅ Maximum performance
✅ Complex queries needed
✅ Production system
✅ Team knows Rust
❌ Quick prototype
❌ Learning Rust

---

## Next Steps

1. **Pick an implementation** from the decision tree above
2. **Follow the setup steps** for that implementation
3. **Test with example input** to verify it works
4. **Update settings.json** to use the hook
5. **Monitor performance** - optimize only if needed

---

## Getting Help

**Questions about:**
- TypeScript: See original `.ts` files
- C#: See `.claude/hooks/CSHARP_HOOKS.md`
- LiteDB: See `.claude/hooks/LiteDbHooks/README.md`
- Rust: See `.claude/hooks/RustHooks/README.md`
- Performance: See `.claude/hooks/PERFORMANCE_COMPARISON.md`
- Databases: See `.claude/hooks/RustHooks/DATABASES.md`

---

**Remember:** There's no "best" implementation - only the best one **for your team and requirements**!

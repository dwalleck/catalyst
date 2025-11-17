# AI-Powered Skill Activation System - Context

**Last Updated:** 2025-11-11
**Status:** Planning Phase

---

## SESSION PROGRESS (2025-11-11)

### ✅ COMPLETED
- Initial analysis of claude-skills-supercharged repository
- Reviewed catalyst's current architecture
- Created comprehensive implementation plan
- **Added pluggable provider system design** (Anthropic, Local LLM, OpenAI)
- Clarified dual-process Rust architecture (no TypeScript!)
- Updated documentation with provider configuration
- **Documented hybrid database access approach** (direct for critical path, API for analytics)
- Added optional Phase 6 (Analytics API) to plan
- **Identified 10 ambiguities/unknowns in plan**
- **Created Phase 0: Pre-Implementation Research** (7 tasks, 1-2 days)

### 🟡 IN PROGRESS
- Planning phase complete
- Ready to start Phase 0 (Research)

### ⚠️ BLOCKERS
- None currently

### 📋 NEXT STEPS
1. ✅ Review plan with team/stakeholders
2. 🎯 **START PHASE 0: Pre-Implementation Research**
   - Test llama.cpp integration with gpt-oss and qwen3-coder (2h)
   - Clarify affinity algorithm (1h)
   - Define config precedence (30m)
   - Error handling spec (1h)
   - Windows process mgmt (1h)
   - SQLite concurrency (1h)
   - Test prompt across providers (2h)
3. ⏳ Validate 95%+ confidence achieved
4. ⏳ Then start Phase 1.1: Implement Cache Manager

---

## Key Files and Locations

### Catalyst Repository (`/home/dwalleck/repos/catalyst/`)

**Core Architecture:**
- `catalyst-cli/src/bin/skill_activation_prompt.rs` - Current keyword-based skill matching hook
- `catalyst-core/src/` - Shared library (settings, utilities)
- `.claude/skills/skill-rules.json` - Skill configuration with triggers

**Existing Hooks:**
- `skill-activation-prompt` - UserPromptSubmit hook (Rust, ~2ms startup)
- `file-analyzer` - File pattern analysis
- `file-change-tracker` - PostToolUse tracking (requires SQLite feature)

**Current Matching Logic:**
- Keyword substring matching (case-insensitive)
- Regex intentPatterns matching
- File path pattern matching
- Content pattern matching (if file edited)

### claude-skills-supercharged Repository (`/home/dwalleck/repos/claude-skills-supercharged/`)

**Reference Implementation:**
- `.claude/hooks/lib/` - Modular TypeScript architecture
  - `intent-analyzer.ts` - AI orchestration
  - `anthropic-client.ts` - API client
  - `skill-filtration.ts` - Promotion + affinity logic
  - `cache-manager.ts` - MD5-based caching
  - `session-state-manager.ts` - Skill tracking
- `.claude/hooks/config/intent-analysis-prompt.txt` - AI prompt template
- `.claude/skills/skill-rules.json` - Skill configuration (TypeScript format)

**Key Innovations:**
1. **AI Intent Analysis** - Claude Haiku 4.5 with confidence scoring
2. **Affinity Injection** - Bidirectional skill relationships (free of slot cost)
3. **Smart Caching** - 1-hour TTL, MD5-based invalidation
4. **7-Stage Pipeline** - Analysis → Scoring → Filtration → Promotion → Affinity → Dependencies → State
5. **Comprehensive Testing** - 120 tests across 11 test files

---

## Architecture Decisions

### Decision 1: Dual-Process Rust Architecture

**Context:** catalyst uses Rust for performance (~2ms), claude-skills-supercharged uses TypeScript/Node.js (~120ms)

**Decision:** Dual-process Rust architecture (both components are Rust!)
- **Process 1:** Rust hook (skill-activation-prompt) - existing, fast, synchronous
- **Process 2:** Rust microservice (intent-analyzer using Axum) - new, handles async AI calls
- Fall back to keyword matching if AI service unavailable

**Why "Dual-Process"?**
- Hook runs synchronously in Claude Code (must be <50ms)
- AI API calls take 100-250ms (too slow for hook)
- Solution: Separate Rust microservice handles AI calls
- Hook makes HTTP call to microservice (200ms timeout)

**Rationale:**
- **100% Rust stack** - No TypeScript/Node.js
- Maintains catalyst's performance advantages
- Adds AI intelligence without forcing it on users
- Optional deployment (CATALYST_USE_AI=1 environment variable)
- Graceful degradation (200ms timeout → keyword fallback)

**Architecture:**
```
┌─────────────────────────────────────┐
│ skill-activation-prompt (Rust)      │  ← Existing hook
│ • Fast keyword matching (~2ms)      │
│ • Optional HTTP call to AI service  │
└──────────────┬──────────────────────┘
               │ HTTP (optional)
               ↓
┌─────────────────────────────────────┐
│ intent-analyzer (Axum/Rust)         │  ← NEW microservice
│ • OpenAI-compatible API             │
│ • Pluggable providers               │
└─────────────────────────────────────┘
```

**Alternatives Considered:**
1. ❌ Embed AI in hook - Too slow (>50ms), blocks Claude Code
2. ❌ Port to TypeScript - Loses performance benefits
3. ✅ Dual-process Rust - Best of both worlds

---

### Decision 2: Pluggable Provider System

**Context:** AI intent analysis can use different backends (Anthropic API, local LLMs, OpenAI, etc.)

**Decision:** Abstract provider interface with multiple implementations
- `IntentAnalysisProvider` trait in Rust
- Implementations: Anthropic, LocalLLM (llama.cpp), OpenAI
- Configuration via TOML or environment variables
- Runtime provider selection

**Rationale:**
- **User Choice** - Pick provider based on needs (cost, privacy, performance)
- **Local LLM Support** - Run 20-60GB models locally (zero cost, privacy, offline)
- **Zero Cost Option** - No API fees with local models
- **Future-Proof** - Easy to add new providers (Gemini, Mistral, etc.)
- **Privacy** - Local models keep prompts on-device

**Provider Options:**

| Provider | Cost | Speed | Privacy | Offline | Quality |
|----------|------|-------|---------|---------|---------|
| **Anthropic** | $1-2/mo | ~200ms | ❌ Cloud | ❌ No | ⭐⭐⭐⭐⭐ |
| **Local (llama.cpp)** | $0 | ~100ms | ✅ Local | ✅ Yes | ⭐⭐⭐⭐ |
| **OpenAI** | $2-3/mo | ~150ms | ❌ Cloud | ❌ No | ⭐⭐⭐⭐⭐ |

**Configuration:**
```toml
# ~/.claude-hooks/config.toml
[ai]
provider = "local"  # or "anthropic", "openai"

[ai.local]
endpoint = "http://localhost:8080/v1/chat/completions"
model = "gpt-oss"  # or "qwen3-coder"
timeout_ms = 5000

[ai.anthropic]
api_key = "sk-ant-..."
model = "claude-haiku-4-5"
timeout_ms = 10000
```

**Implementation:**
```rust
#[async_trait]
trait IntentAnalysisProvider: Send + Sync {
    async fn analyze(
        &self,
        prompt: &str,
        skills: &SkillRules
    ) -> Result<IntentAnalysis>;
}

// Implementations
struct AnthropicProvider { /* ... */ }
struct LocalLLMProvider { /* llama.cpp via OpenAI-compatible API */ }
struct OpenAIProvider { /* GPT-3.5/4 */ }
```

**Alternatives Considered:**
1. ❌ Anthropic-only - Locks users into paid API
2. ❌ Hard-coded providers - Not extensible
3. ✅ Pluggable trait system - Maximum flexibility

---

### Decision 3: Cache Location and Format

**Context:** Need to cache AI analysis results to reduce API costs

**Decision:**
- Location: `~/.claude-hooks/cache/intent-analysis/`
- Format: JSON files named by MD5 hash
- Structure: `{timestamp, result: {required: [], suggested: []}}`
- TTL: 1 hour

**Rationale:**
- Separate from project directories (shared across projects)
- MD5 hash ensures unique keys (prompt + skill-rules.json hash)
- JSON for human readability and debugging
- 1-hour TTL balances freshness vs cost

**Alternatives Considered:**
1. ❌ SQLite cache - More complex, overkill for key-value storage
2. ❌ In-memory cache - Doesn't persist across hook invocations
3. ✅ File-based JSON - Simple, debuggable, adequate performance

---

### Decision 4: Affinity as Separate Field

**Context:** claude-skills-supercharged uses `affinity` array in skill-rules.json

**Decision:** Add `affinity` field to catalyst's SkillRule struct
- Maximum 2 affinity skills per skill
- Bidirectional semantics (both directions checked)
- Free of slot cost (doesn't count toward 2-skill limit)

**Rationale:**
- Separates concern from requiredSkills (which are one-way dependencies)
- Limit of 2 prevents explosion of injected skills
- Free slot cost enables richer context without overwhelming

**Schema:**
```json
{
  "frontend-dev-guidelines": {
    "affinity": ["backend-dev-guidelines"]
  }
}
```

---

### Decision 5: Session State in SQLite

**Context:** Need to track which skills have been loaded in a session

**Decision:** Extend existing SQLite state management
- Table: `acknowledged_skills`
- Columns: session_id, skill_name, injected_at, injection_type, confidence
- Track injection type (direct, affinity, promoted)

**Rationale:**
- Leverage existing SQLite infrastructure (already have feature flag)
- Efficient queries (indexed by session_id)
- Can track metadata (confidence scores, timestamps)
- Enables future analytics

**Alternatives Considered:**
1. ❌ File-based state - O(n) queries, no structured queries
2. ✅ SQLite - 100x faster queries, already in codebase

---

### Decision 6: Axum for Microservice

**Context:** Need HTTP server for AI intent analysis

**Decision:** Use Axum framework
- Tokio-based async runtime
- Type-safe request/response handling
- Excellent performance (~500 req/s)
- Well-documented, actively maintained

**Rationale:**
- Axum is the modern standard for Rust HTTP servers
- Tokio async enables concurrent API calls
- Type safety catches errors at compile time
- Small binary footprint (~2-3MB)

**Alternatives Considered:**
1. ❌ Actix-web - More complex, actor-based
2. ❌ Rocket - Not async (Tokio-based)
3. ✅ Axum - Simple, performant, type-safe

---

### Decision 7: Hybrid Database Access (Direct vs API)

**Context:** Should session state database be accessed directly or through the API?

**Decision:** Hybrid approach - critical path direct, analytics through API
- **Phase 1-5:** Hook accesses SQLite directly (performance-critical)
- **Phase 6 (optional):** API adds read-only analytics endpoints

**Critical Path (Direct):**
```
skill-activation-prompt → Direct SQLite (~1-2ms)
```

**Optional Analytics (API):**
```
Web Dashboard/CLI → API → Read-only SQLite
```

**Rationale:**
- **Performance:** Hook stays <50ms with direct access
- **Independence:** Hook works offline (no API dependency)
- **Simplicity:** One process, fewer failure points
- **Future Growth:** Can add analytics later without breaking changes
- **Optional Features:** Web dashboard, CLI stats don't block core functionality

**Why Direct is Critical:**
- Session state queries are on the hot path (every prompt)
- 1-2ms (direct) vs 5-10ms (API) = 3-8ms penalty
- Hook has 50ms budget total
- Needs to work when AI service is down

**Why API for Analytics is Optional:**
- Analytics aren't time-sensitive
- Enables web dashboard (browser can't access SQLite directly)
- CLI stats can still use direct access
- Read-only = safe, no concurrency issues

**Phase 6 (Optional) Features:**
- GET /api/stats/skills - Most-used skills
- GET /api/stats/sessions - Usage over time
- GET /api/stats/confidence - AI accuracy tracking
- GET /dashboard - Web visualization
- catalyst stats skills - CLI analytics
- catalyst stats export - Data export

**Architecture:**
```
┌─────────────────────┐          ┌─────────────────────┐
│ skill-activation    │          │ Web Dashboard       │
│ (Direct DB)         │          │ (Optional)          │
│ • Fast path         │          │ • Via API           │
│ • Works offline     │          │ • Read-only         │
└──────────┬──────────┘          └──────────┬──────────┘
           │                                 │
           │ HTTP (AI only)                  │ HTTP (analytics)
           ↓                                 ↓
┌──────────────────────────────────────────────────────┐
│ intent-analyzer (Axum API)                           │
│ • POST /analyze (Phase 2-5, required)                │
│ • GET /api/stats/* (Phase 6, optional)               │
└──────────────┬───────────────────────────────────────┘
               │
               ↓ (read-only, Phase 6 only)
        [Session State DB]
               ↑
               │ (read/write, always)
        [skill-activation-prompt]
```

**Alternatives Considered:**
1. ❌ All access through API - Too slow, adds dependency
2. ❌ No analytics - Loses valuable insights
3. ✅ Hybrid - Best of both worlds

---

## Technical Constraints

### Performance Constraints

**Hook Performance:**
- UserPromptSubmit hooks must be <50ms (imperceptible)
- Current catalyst: ~2ms
- Target with cache: <10ms
- Target with AI: <250ms (with 200ms timeout → fallback)

**Cache Performance:**
- Read: <1ms
- Write: <5ms (atomic)
- Invalidation: Instant (new MD5 key)

**Database Performance:**
- Session state query: <5ms
- Insert acknowledged skill: <5ms

### Cost Constraints

**AI API Costs:**
- Target: $1-2/month @ 100 prompts/day
- Haiku pricing: $0.25 per 1M input tokens, $1.25 per 1M output tokens
- Average prompt: ~1000 input, ~100 output tokens
- Caching reduces by ~80%

**Disk Space:**
- Cache: <10MB (1-hour TTL, auto-cleanup)
- State: <5MB (7-day retention)
- Binaries: ~5MB total

### Compatibility Constraints

**Backward Compatibility:**
- Existing keyword matching must still work
- CATALYST_USE_AI=0 disables AI features
- Hooks work without intent-analyzer service
- skill-rules.json remains valid without affinity field

**Platform Support:**
- Linux, macOS, Windows
- Rust 1.70+ (MSRV)
- SQLite 3.40+

---

## Important Code Patterns

### Catalyst Patterns

**Error Handling:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum SkillActivationError {
    #[error("[SA001] Failed to read input from stdin")]
    StdinRead(#[from] io::Error),

    #[error("[SA002] Invalid JSON input: {0}")]
    InvalidInput(#[source] serde_json::Error),
}
```

**Atomic File Writes:**
```rust
use tempfile::NamedTempFile;

fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let mut temp = NamedTempFile::new()?;
    temp.write_all(content.as_bytes())?;
    temp.persist(path)?;
    Ok(())
}
```

**SQLite with rusqlite:**
```rust
let conn = Connection::open(&db_path)?;
conn.execute(
    "INSERT INTO table (col1, col2) VALUES (?1, ?2)",
    params![value1, value2]
)?;
```

### claude-skills-supercharged Patterns

**Confidence Scoring:**
```typescript
const CONFIDENCE_THRESHOLD = 0.65;  // Auto-inject
const SUGGESTED_THRESHOLD = 0.50;   // Suggest

function categorizeSkills(analysis: IntentAnalysis) {
  const required = analysis.skills
    .filter(s => s.confidence > CONFIDENCE_THRESHOLD)
    .map(s => s.name);
  const suggested = analysis.skills
    .filter(s => s.confidence >= SUGGESTED_THRESHOLD && s.confidence <= CONFIDENCE_THRESHOLD)
    .map(s => s.name);
  return { required, suggested };
}
```

**Affinity Injection (Bidirectional):**
```typescript
function findAffinityInjections(toInject: string[], acknowledged: string[], rules: SkillRules) {
  const affinitySet = new Set<string>();

  for (const skill of toInject) {
    // Direction 1: skill → affinity
    for (const affinity of rules[skill].affinity || []) {
      if (!acknowledged.includes(affinity) && !toInject.includes(affinity)) {
        affinitySet.add(affinity);
      }
    }

    // Direction 2: other skills → skill (reverse)
    for (const [otherSkill, otherConfig] of Object.entries(rules)) {
      if ((otherConfig.affinity || []).includes(skill)) {
        if (!acknowledged.includes(otherSkill) && !toInject.includes(otherSkill)) {
          affinitySet.add(otherSkill);
        }
      }
    }
  }

  return Array.from(affinitySet);
}
```

---

## Environment Variables

**Existing:**
- `CLAUDE_PROJECT_DIR` - Project root directory
- `CLAUDE_SKILL_DEBUG` - Enable debug logging

**New (to be added):**
- `CATALYST_USE_AI` - Enable AI intent analysis (1 = enabled, 0 = keyword only)
- `CATALYST_AI_PROVIDER` - Provider to use (default: local)
  - Values: `local`, `anthropic`, `openai`
- `CATALYST_AI_TIMEOUT` - AI request timeout in ms (default: 200)
- `CATALYST_CACHE_TTL` - Cache TTL in seconds (default: 3600)
- `CATALYST_DEBUG` - Show confidence scores in banner

**Provider-Specific:**
- `ANTHROPIC_API_KEY` - Anthropic API key (if using anthropic provider)
- `ANTHROPIC_MODEL` - Model to use (default: claude-haiku-4-5)
- `OPENAI_API_KEY` - OpenAI API key (if using openai provider)
- `OPENAI_MODEL` - Model to use (default: gpt-3.5-turbo)
- `LOCAL_LLM_ENDPOINT` - Local LLM endpoint (default: http://localhost:8080/v1/chat/completions)
- `LOCAL_LLM_MODEL` - Local model name (default: gpt-oss or qwen3-coder)

---

## Dependencies to Add

**Cargo.toml (catalyst-cli):**
```toml
[dependencies]
# Existing...

# HTTP client for AI service
reqwest = { version = "0.11", features = ["json", "blocking"] }

# MD5 hashing for cache keys
md5 = "0.7"
```

**Cargo.toml (intent-analyzer, new crate):**
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
async-trait = "0.1"  # For provider trait
toml = "0.8"         # For config parsing
```

**Cargo.toml (catalyst-core - add config support):**
```toml
[dependencies]
# Existing...
toml = "0.8"  # For AI config parsing
```

---

## Testing Strategy

**Unit Tests:**
- Each module (cache, affinity, state) has its own test module
- Use `#[cfg(test)]` modules in same file
- Mock external dependencies (Anthropic API)

**Integration Tests:**
- End-to-end tests in `tests/` directory
- Test both AI path and keyword fallback
- Test cache hit/miss scenarios
- Test session state across multiple prompts

**Benchmarks:**
- Use Criterion for performance benchmarks
- Track cache read/write latency
- Track full pipeline latency (cache hit vs miss)
- Ensure <10ms for cache hits

**Test Coverage:**
- Target: >80% coverage
- Use `cargo tarpaulin` for coverage reporting
- CI/CD should run tests on every commit

---

## Quick Resume Instructions

**To continue this work:**

1. **Read this file first** - Understand current state and decisions
2. **Read the plan** - See the full implementation roadmap
3. **Check tasks.md** - See what's next in the checklist
4. **Start with Phase 1.1** - Implement Cache Manager
   - File: `catalyst-core/src/cache.rs`
   - Tests: `catalyst-core/src/cache.rs` (#[cfg(test)])
   - Reference: `.claude/hooks/lib/cache-manager.ts` in claude-skills-supercharged

**Development Workflow:**
1. Create feature branch: `git checkout -b feature/ai-skill-activation`
2. Implement task from plan
3. Write tests for new functionality
4. Run `cargo test` to ensure all tests pass
5. Run `cargo clippy` for linting
6. Run `cargo fmt` for formatting
7. Commit with descriptive message
8. Update context.md with progress

**Key References:**
- claude-skills-supercharged: `/home/dwalleck/repos/claude-skills-supercharged/`
- Catalyst design doc: `PROJECT_DESIGN.md`
- Dev docs pattern: `dev/README.md`

---

**End of Context**
**Last Updated:** 2025-11-11

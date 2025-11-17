# AI-Powered Skill Activation System - Implementation Plan

**Last Updated:** 2025-11-11
**Status:** Planning
**Estimated Duration:** 4-6 weeks
**Complexity:** High

---

## Executive Summary

This plan outlines the integration of AI-powered skill activation features from claude-skills-supercharged into the catalyst project. The primary innovation is replacing keyword/regex-based skill matching with Claude Haiku 4.5 intent analysis, achieving 95%+ accuracy while maintaining catalyst's high-performance Rust foundation.

**Core Innovation:** Hybrid architecture combining Rust hook performance (2ms) with optional AI intent analysis via Axum-based microservice, delivering intelligent skill detection without sacrificing speed.

**Key Deliverables:**

1. AI intent analysis microservice (Axum + Anthropic API)
2. Affinity injection system (bidirectional skill relationships)
3. Smart caching layer (1-hour TTL, MD5-based invalidation)
4. Enhanced session state management with SQLite
5. Comprehensive test suite (120+ tests)
6. /wrap command for skill maintenance

---

## Current State Analysis

### Catalyst's Existing Architecture

**Strengths:**
- ✅ High-performance Rust hooks (~2ms startup)
- ✅ Zero runtime dependencies
- ✅ Standalone installation model (~/.claude-hooks/bin/)
- ✅ Keyword + regex + path pattern matching
- ✅ Optional SQLite state management
- ✅ Well-documented, production-tested

**Limitations:**
- ❌ Simple keyword matching (60-70% accuracy)
- ❌ No affinity/relationship system between skills
- ❌ No caching of skill suggestions
- ❌ Limited session state tracking
- ❌ No AI-powered intent understanding
- ❌ No skill maintenance automation

**Current skill-activation-prompt.rs:**
```rust
// Matching logic (simplified)
fn match_skills(prompt: &str, rules: &SkillRules) -> Vec<String> {
    // Keyword substring matching (lowercase)
    // Regex pattern matching (intentPatterns)
    // File path patterns (if files edited)
}
```

---

### claude-skills-supercharged Architecture

**Innovations:**
- ✅ AI intent analysis (Claude Haiku 4.5, 0.0-1.0 confidence scores)
- ✅ Smart caching (MD5 hash, 1-hour TTL)
- ✅ Affinity injection (bidirectional, free of slot cost)
- ✅ 7-stage injection pipeline
- ✅ Promotion logic (fill 2-skill target)
- ✅ Comprehensive testing (120 tests)
- ✅ /wrap command for skill updates

**Limitations:**
- ❌ TypeScript/Node.js (~120ms startup)
- ❌ Requires npm install in every project
- ❌ Runtime dependencies (node_modules/)
- ❌ No standalone installation model

---

## Proposed Future State

### Dual-Process Rust Architecture

**Design Philosophy:** 100% Rust stack with pluggable AI providers

**Key Clarification:** Both components are Rust!
- **Process 1:** skill-activation-prompt (Rust hook)
- **Process 2:** intent-analyzer (Axum/Rust microservice)
- **No TypeScript/Node.js** - Entire stack is Rust

```
┌─────────────────────────────────────────────────────────────┐
│ User Prompt                                                 │
│ "Help me write Python code"                                 │
└────────────────────┬────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────────────┐
│ skill-activation-prompt (Rust)                              │
│ • Read skill-rules.json                                     │
│ • Check cache first (MD5 hash)                              │
│ • Decide: AI analysis or keyword fallback?                  │
└────────────────────┬────────────────────────────────────────┘
                     ↓
              [Cache Hit?]
              ┌────┴────┐
            YES          NO
              ↓           ↓
        Use Cached   ┌─────────────────────────────────────┐
        Result       │ Intent Analysis Service (Axum/Rust) │
                     │ • Pluggable provider system         │
                     │ • Confidence scoring (0.0-1.0)      │
                     │ • Cache result (1-hour TTL)         │
                     └────────────┬────────────────────────┘
                                  ↓
                         [Select Provider]
                    ┌───────┬─────────┬───────┐
                    ↓       ↓         ↓       ↓
              ┌──────────┬────────┬─────────┬───────┐
              │Anthropic │ Local    │ OpenAI  │ ...   │
              │ API      │ LLM      │ API     │       │
              │(Haiku)   │(llama.cpp)│(GPT-3.5)│       │
              └──────────┴────────┴─────────┴───────┘
                                  ↓
┌─────────────────────────────────────────────────────────────┐
│ Skill Filtration & Promotion (Rust)                         │
│ • Filter acknowledged skills                                │
│ • Apply 2-skill injection limit                             │
│ • Promote suggested skills to fill slots                    │
└────────────────────┬────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────────────┐
│ Affinity Injection (Rust)                                   │
│ • Find bidirectional relationships                          │
│ • Free bonus skills (don't count toward limit)              │
└────────────────────┬────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────────────┐
│ Dependency Resolution (Rust)                                │
│ • Resolve requiredSkills                                    │
│ • Sort by injectionOrder                                    │
│ • Detect circular dependencies                              │
└────────────────────┬────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────────────┐
│ Session State Management (SQLite)                           │
│ • Track loaded skills                                       │
│ • Prevent duplicate injections                              │
│ • Store affinity metadata                                   │
└────────────────────┬────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────────────┐
│ Output Formatter (Rust)                                     │
│ • Banner with injected skills                               │
│ • Show affinity relationships                               │
│ • Display confidence scores (debug mode)                    │
└─────────────────────────────────────────────────────────────┘
```

**Performance Targets:**

| Scenario | Target | How We Achieve It |
|----------|--------|-------------------|
| **Cache Hit** | <10ms | Rust hook reads cache directly |
| **AI Analysis (Cloud)** | <250ms | Axum microservice (concurrent requests) |
| **AI Analysis (Local)** | <100ms | Local LLM (no network latency) |
| **Keyword Fallback** | <5ms | Rust-only path (no AI) |
| **Monthly Cost (Cloud)** | $1-2 | Caching + Haiku pricing |
| **Monthly Cost (Local)** | $0 | Free - runs on your hardware |

---

## Implementation Phases

### Phase 0: Pre-Implementation Research (1-2 days)

**Goal:** Validate assumptions and clarify ambiguities before implementation

**Why First?** Increases confidence from ~70% to 90%+ across all phases

**Status:** Must complete before Phase 1

---

#### Research Tasks

**0.1: Test llama.cpp Integration** [Effort: M] [2 hours]

**Goal:** Validate that local LLMs work with OpenAI-compatible API via llama.cpp

**Acceptance Criteria:**
- Set up llama.cpp with gpt-oss model
- Test OpenAI-compatible API endpoint
- Verify JSON response format
- Measure response time with full intent analysis prompt (~1000 tokens)
- Test context window (can it handle prompt + all skills?)
- Document any quirks or issues

**Implementation:**
```bash
# Build llama.cpp with server support
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make server

# Download gpt-oss model (GGUF format)
# Model files should be in ~/.cache/llama.cpp/models/

# Start server
./server -m ~/.cache/llama.cpp/models/gpt-oss.gguf -c 4096 --port 8080

# Test API
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-oss",
    "messages": [{
      "role": "user",
      "content": "Analyze this prompt and return JSON..."
    }],
    "response_format": {"type": "json_object"},
    "temperature": 0.0
  }'

# Measure:
# - Response time (target: <200ms)
# - JSON parsing success
# - Confidence score accuracy
```

**Document:**
- Does llama.cpp server support `response_format: json_object`?
- What's the actual response time?
- Are confidence scores reasonable?
- Any special configuration needed?
- Compare gpt-oss vs qwen3-coder performance

---

**0.2: Clarify Affinity Algorithm** [Effort: S] [1 hour]

**Goal:** Define precise algorithm for affinity injection to prevent infinite loops

**Acceptance Criteria:**
- Document algorithm with pseudocode
- Define cycle detection logic
- Specify hard limits (max depth, max total skills)
- Add examples with edge cases
- Update plan.md with detailed algorithm

**Algorithm to Define:**
```rust
// Pseudocode
fn find_affinity_injections(
    to_inject: &[String],
    acknowledged: &[String],
    rules: &SkillRules
) -> Vec<String> {
    let mut result = HashSet::new();
    let mut visited = HashSet::new();  // Cycle detection

    for skill in to_inject {
        collect_affinities(skill, &mut result, &mut visited, rules, 0);
    }

    result.into_iter().collect()
}

fn collect_affinities(
    skill: &str,
    result: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    rules: &SkillRules,
    depth: u32
) {
    // Hard limits
    const MAX_DEPTH: u32 = 2;        // 2 levels deep
    const MAX_AFFINITIES: usize = 10; // Max 10 total

    if depth >= MAX_DEPTH || result.len() >= MAX_AFFINITIES {
        return;
    }

    if visited.contains(skill) {
        return;  // Cycle detected
    }

    visited.insert(skill.to_string());

    // Direction 1: skill → its affinities
    for affinity in rules[skill].affinity {
        if !acknowledged.contains(affinity) {
            result.insert(affinity.clone());
            collect_affinities(affinity, result, visited, rules, depth + 1);
        }
    }

    // Direction 2: other skills → skill (reverse)
    for (other_skill, other_config) in rules {
        if other_config.affinity.contains(skill) {
            if !acknowledged.contains(other_skill) {
                result.insert(other_skill.clone());
                collect_affinities(other_skill, result, visited, rules, depth + 1);
            }
        }
    }

    visited.remove(skill);
}
```

**Edge Cases to Document:**
1. Circular affinity: A↔B
2. Chain affinity: A→B→C→D
3. Diamond affinity: A→B, A→C, B→D, C→D
4. Max limit reached: 10 skills before traversal complete

**Update:** Add algorithm to plan.md Phase 1.3

---

**0.3: Define Configuration Precedence** [Effort: S] [30 min]

**Goal:** Document clear precedence order for configuration sources

**Acceptance Criteria:**
- Define precedence: CLI > Env > Config File > Default
- Document behavior when multiple sources conflict
- Add examples for common scenarios
- Update context.md with precedence rules

**Precedence Order (Highest to Lowest):**
```
1. CLI flags          catalyst ai start --provider openai
2. Environment vars   CATALYST_AI_PROVIDER=anthropic
3. Config file        ~/.claude-hooks/config.toml
4. Built-in defaults  local (llama.cpp)
```

**Examples:**
```bash
# Example 1: CLI wins
export CATALYST_AI_PROVIDER=anthropic
catalyst ai start --provider local  # Uses local (CLI flag)

# Example 2: Env wins
# config.toml: provider = "openai"
export CATALYST_AI_PROVIDER=anthropic
catalyst ai start  # Uses anthropic (env var over config)

# Example 3: Config wins
# config.toml: provider = "openai"
catalyst ai start  # Uses openai (config over default)

# Example 4: Default
catalyst ai start  # Uses local (default)
```

**Implementation:**
```rust
fn resolve_provider() -> Provider {
    // 1. Check CLI flag
    if let Some(provider) = cli_args.provider {
        return provider;
    }

    // 2. Check environment variable
    if let Ok(provider) = env::var("CATALYST_AI_PROVIDER") {
        return provider.parse()?;
    }

    // 3. Check config file
    if let Ok(config) = read_config_file() {
        if let Some(provider) = config.ai.provider {
            return provider;
        }
    }

    // 4. Default
    Provider::Local
}
```

**Update:** Add to context.md Decision 2 (Pluggable Provider System)

---

**0.4: Add Error Handling Spec** [Effort: S] [1 hour]

**Goal:** Define how AI failures are communicated to users

**Acceptance Criteria:**
- Specify error banner format
- Define retry logic (attempts, delays)
- Document cache behavior on errors
- Specify logging levels
- Update plan.md Phase 2.5 with error handling

**Error Scenarios:**

**1. API Timeout:**
```rust
match timeout(Duration::from_millis(200), call_ai_api(&prompt)).await {
    Ok(Ok(result)) => result,
    Ok(Err(e)) => {
        eprintln!("⚠️  AI analysis failed: {}", e);
        eprintln!("    Falling back to keyword matching");
        keyword_matching(&prompt)?
    }
    Err(_) => {
        eprintln!("⚠️  AI analysis timed out (>200ms)");
        eprintln!("    Falling back to keyword matching");
        keyword_matching(&prompt)?
    }
}
```

**2. API Key Invalid:**
```
⚠️  AI Provider Error: Anthropic
    401 Unauthorized - Invalid API key

    Fix: export ANTHROPIC_API_KEY=sk-ant-...
    Or: catalyst ai setup --provider anthropic

    Falling back to keyword matching
```

**3. Service Unavailable:**
```
⚠️  Intent analyzer service not responding
    Is it running? Check: catalyst ai status

    Falling back to keyword matching
```

**Retry Logic:**
```rust
const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_MS: u64 = 50;

for attempt in 0..=MAX_RETRIES {
    match call_ai_api(&prompt).await {
        Ok(result) => return Ok(result),
        Err(e) if attempt < MAX_RETRIES => {
            tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

**Cache Behavior:**
- ✅ Cache successful AI results
- ❌ Don't cache errors (retry next time)
- ✅ Cache keyword fallback results (1 hour TTL)

**Banner Indicators:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📚 AUTO-LOADED SKILLS (keyword matching)  ← Shows method used
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Update:** Add to plan.md Phase 2.5 implementation

---

**0.5: Research Windows Process Management** [Effort: S] [1 hour]

**Goal:** Ensure `catalyst ai start` works cross-platform

**Acceptance Criteria:**
- Test process spawning on Windows
- Verify background process detachment
- Document platform-specific code needed
- Test PID file creation/reading
- Update plan.md Phase 5.1 with Windows support

**Test on Windows:**
```powershell
# Does this detach properly?
$process = Start-Process -FilePath "intent-analyzer.exe" -WindowStyle Hidden -PassThru
$process.Id  # Write to PID file

# Can we read the PID back and kill it?
$pid = Get-Content "intent-analyzer.pid"
Stop-Process -Id $pid
```

**Platform-Specific Code:**
```rust
#[cfg(windows)]
fn start_intent_analyzer() -> Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const DETACHED_PROCESS: u32 = 0x00000008;

    let child = Command::new(&binary)
        .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
        .spawn()?;

    write_pid_file(child.id())?;
    Ok(())
}

#[cfg(unix)]
fn start_intent_analyzer() -> Result<()> {
    let child = Command::new(&binary)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    write_pid_file(child.id())?;
    Ok(())
}
```

**Test:**
- Does background process survive terminal close?
- Can we stop it via PID file?
- Does it show in task manager (Windows) / ps (Unix)?

**Update:** Add platform-specific code to plan.md Phase 5.1

---

**0.6: SQLite Concurrency Testing** [Effort: S] [1 hour]

**Goal:** Validate concurrent access patterns work correctly

**Acceptance Criteria:**
- Test WAL mode enables concurrent reads
- Verify write locking behavior
- Test retry logic for SQLITE_BUSY
- Document multi-session support
- Update plan.md Phase 3.1 with concurrency handling

**Test Scenarios:**
```rust
// Test 1: Concurrent reads (should work)
let db1 = Connection::open(&db_path)?;
let db2 = Connection::open(&db_path)?;

db1.execute("PRAGMA journal_mode=WAL", [])?;

// Both should read simultaneously
let thread1 = thread::spawn(|| db1.query_row(...));
let thread2 = thread::spawn(|| db2.query_row(...));

// Test 2: Concurrent writes (should serialize)
let thread1 = thread::spawn(|| db1.execute("INSERT ...", []));
let thread2 = thread::spawn(|| db2.execute("INSERT ...", []));

// Test 3: Read while writing (should work with WAL)
let thread1 = thread::spawn(|| db1.execute("INSERT ...", []));
let thread2 = thread::spawn(|| db2.query_row(...));
```

**Retry Logic:**
```rust
pub fn with_retry<T, F>(mut f: F) -> Result<T>
where
    F: FnMut() -> rusqlite::Result<T>,
{
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 10;

    for attempt in 0..MAX_RETRIES {
        match f() {
            Ok(result) => return Ok(result),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::DatabaseBusy
                && attempt < MAX_RETRIES - 1 =>
            {
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    unreachable!()
}
```

**WAL Mode Configuration:**
```rust
impl SessionStateManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for concurrent access
        conn.execute("PRAGMA journal_mode=WAL", [])?;
        conn.execute("PRAGMA busy_timeout=5000", [])?; // 5 sec timeout

        Ok(Self { conn })
    }
}
```

**Document:**
- Multiple Claude Code sessions can run simultaneously
- WAL mode enables concurrent reads + serialized writes
- Retry logic handles transient SQLITE_BUSY errors

**Update:** Add to plan.md Phase 3.1 implementation

---

**0.7: Test Prompt Across Providers** [Effort: M] [2 hours]

**Goal:** Validate intent analysis prompt works with all providers

**Acceptance Criteria:**
- Test prompt with Anthropic Haiku (baseline)
- Test prompt with llama.cpp gpt-oss
- Test prompt with llama.cpp qwen3-coder
- Test prompt with OpenAI GPT-3.5 (if API key available)
- Document required adjustments per provider
- Create provider-specific prompt variants if needed
- Update plan.md Phase 2.3 with findings

**Test Matrix:**

| Provider | Model | Expected Behavior | Adjustments Needed? |
|----------|-------|-------------------|---------------------|
| Anthropic | Haiku 4.5 | ✅ Works (designed for Claude) | None |
| llama.cpp | gpt-oss | ❓ Test JSON output, confidence scores | TBD |
| llama.cpp | qwen3-coder | ❓ Alternative if gpt-oss struggles | TBD |
| OpenAI | GPT-3.5 | ❓ Test structured output | TBD |

**Test Script:**
```bash
# Test with each provider
for provider in anthropic llamacpp-gpt-oss llamacpp-qwen3-coder openai-gpt35; do
    echo "Testing $provider..."

    # Send same prompt
    curl -X POST http://localhost:3030/analyze \
      -H "Content-Type: application/json" \
      -d '{
        "prompt": "Help me write a Python function",
        "skills": {...},
        "provider": "'$provider'"
      }'

    # Check:
    # 1. Returns valid JSON
    # 2. Confidence scores 0.0-1.0
    # 3. Reasonable skill selection
    # 4. Response time acceptable
done
```

**Document Findings:**
```markdown
## Provider-Specific Behavior

### Anthropic Haiku
- ✅ Works as expected
- Response time: ~150ms
- Confidence scores: Accurate

### llama.cpp gpt-oss
- ❓ Test JSON output format
- Response time: ~100ms (target)
- Confidence scores: TBD
- Notes: May need structured output guidance

### llama.cpp qwen3-coder
- ❓ Test JSON compliance
- Response time: ~120ms (target)
- Confidence scores: TBD
- Recommendation: May be preferred local model for code analysis

### OpenAI GPT-3.5
- ✅ Works well with response_format: json_object
- Response time: ~200ms
- Confidence scores: Very accurate
- Cost: Higher than Haiku ($0.002/1K tokens)
```

**If Adjustments Needed:**
```rust
// intent-analyzer/src/prompt_template.rs
pub fn build_analysis_prompt(
    prompt: &str,
    skills: &SkillRules,
    provider: &Provider
) -> String {
    let base_prompt = include_str!("../config/intent-analysis-prompt.txt");

    // Provider-specific adjustments
    match provider {
        Provider::Local => {
            // Simpler language for local models
            format!("{}\n\nIMPORTANT: You must respond with valid JSON only.", base_prompt)
        }
        Provider::Anthropic | Provider::OpenAI => {
            // Original prompt works fine
            base_prompt.to_string()
        }
    }
}
```

**Update:** Add findings to plan.md Phase 2.3, adjust prompt template if needed

---

#### Phase 0 Deliverables

**Documentation:**
- [ ] Ollama integration guide
- [ ] Affinity algorithm specification
- [ ] Configuration precedence rules
- [ ] Error handling specification
- [ ] Platform-specific code notes
- [ ] SQLite concurrency documentation
- [ ] Provider compatibility matrix

**Code:**
- [ ] Affinity algorithm pseudocode
- [ ] Error handling examples
- [ ] Platform-specific spawn code
- [ ] SQLite retry logic
- [ ] Provider-specific prompt templates (if needed)

**Validation:**
- [ ] llama.cpp responds <200ms (test with gpt-oss and qwen3-coder)
- [ ] Affinity algorithm handles all edge cases
- [ ] Configuration precedence clear
- [ ] Windows process detachment works
- [ ] SQLite concurrent access works
- [ ] All providers return valid JSON

---

#### Phase 0 Success Criteria

**Ready to proceed to Phase 1 when:**
- ✅ llama.cpp integration proven (or alternative identified)
- ✅ Affinity algorithm specified with no ambiguities
- ✅ Configuration precedence documented
- ✅ Error handling strategy defined
- ✅ Platform-specific code tested
- ✅ SQLite concurrency validated
- ✅ Prompt works across providers (or variants created)

**Confidence Level:**
- Before Phase 0: ~70%
- After Phase 0: ~95%

---

### Phase 1: Foundation - Caching & Affinity (Week 1-2)

**Goal:** Add caching and affinity without AI dependency

**Why First?** Immediate value, no API key required, tests core concepts

#### Tasks

**1.1: Implement Cache Manager (Rust)** [Effort: M]

**Acceptance Criteria:**
- Cache stored in `~/.claude-hooks/cache/intent-analysis/`
- MD5 hash of (prompt + skill_rules_hash)
- 1-hour TTL with automatic expiration
- Cache invalidated when skill-rules.json changes
- Atomic cache writes (tempfile + rename)

**Files to Create:**
- `catalyst-core/src/cache.rs` - Core cache logic
- Cache format: JSON with `{timestamp, result: {required, suggested}}`

**Implementation:**
```rust
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn read(&self, key: &str) -> Option<CachedResult> {
        // Read from cache_dir/{key}.json
        // Check TTL (1 hour)
        // Return None if expired
    }

    pub fn write(&self, key: &str, result: &IntentResult) -> Result<()> {
        // Atomic write (tempfile + rename)
        // Include timestamp
    }

    pub fn compute_key(prompt: &str, skills_hash: &str) -> String {
        // MD5 hash of prompt + skills_hash
    }
}
```

**Testing:**
- Unit tests for TTL expiration
- Test cache invalidation on skill-rules.json changes
- Test atomic writes
- Benchmark: cache read <1ms

---

**1.2: Add Affinity Field to skill-rules.json** [Effort: S]

**Acceptance Criteria:**
- `affinity` field added to SkillRule struct
- Maximum 2 affinity skills per skill
- Validation during parsing
- Documentation of bidirectional semantics

**Schema Change:**
```json
{
  "frontend-dev-guidelines": {
    "type": "domain",
    "affinity": ["backend-dev-guidelines", "rust-developer"],
    "description": "...",
    "promptTriggers": {...}
  }
}
```

**Files to Modify:**
- `catalyst-cli/src/types.rs` - Add affinity to SkillRule
- `catalyst-cli/src/validation.rs` - Validate max 2 affinities

**Testing:**
- Parse skill-rules.json with affinity field
- Reject >2 affinities
- Accept 0-2 affinities

---

**1.3: Implement Affinity Injection Logic (Rust)** [Effort: L]

**Acceptance Criteria:**
- Bidirectional affinity detection (both directions)
- Affinity skills don't count toward 2-skill limit
- Already-acknowledged skills filtered out
- Respects autoInject flag

**Algorithm:**
```rust
fn find_affinity_injections(
    to_inject: &[String],
    acknowledged: &[String],
    skill_rules: &SkillRules
) -> Vec<String> {
    // For each skill in to_inject:
    //   1. Check skill's affinity array → add those skills
    //   2. Check other skills' affinity arrays → if they list this skill, add them
    // Filter out:
    //   - Already acknowledged
    //   - Already in to_inject
    //   - autoInject: false
}
```

**Files to Create:**
- `catalyst-core/src/affinity.rs` - Affinity injection logic

**Testing:**
- Unit tests for bidirectional detection
- Test with circular affinities (A↔B)
- Test with chains (A→B, B→C)
- Verify free slot cost

---

**1.4: Integrate Cache + Affinity into skill-activation-prompt** [Effort: M]

**Acceptance Criteria:**
- Hook checks cache before keyword matching
- Affinity injection runs after skill filtration
- Cache stores affinity-enhanced results
- Debug logging shows affinity decisions

**Flow:**
```rust
fn main() -> Result<()> {
    let input = read_stdin()?;
    let rules = load_skill_rules()?;

    // Cache key
    let skills_hash = compute_skills_hash(&rules);
    let cache_key = CacheManager::compute_key(&input.prompt, &skills_hash);

    // Check cache
    if let Some(cached) = cache.read(&cache_key) {
        return output_banner(cached.result);
    }

    // Keyword matching (existing)
    let (required, suggested) = match_skills(&input.prompt, &rules);

    // Filtration (existing)
    let filtered = filter_skills(required, suggested, &acknowledged);

    // NEW: Affinity injection
    let affinity_skills = find_affinity_injections(&filtered.to_inject, &acknowledged, &rules);
    let all_skills = [filtered.to_inject, affinity_skills].concat();

    // Write cache
    cache.write(&cache_key, &IntentResult { required, suggested })?;

    output_banner(all_skills)
}
```

**Files to Modify:**
- `catalyst-cli/src/bin/skill_activation_prompt.rs`

**Testing:**
- Integration test: prompt → cache → affinity → output
- Test cache hit scenario
- Test cache miss scenario

---

**1.5: Update skill-rules.json with Affinity Relationships** [Effort: S]

**Acceptance Criteria:**
- All catalyst skills have meaningful affinities defined
- Bidirectional where appropriate
- Maximum 2 per skill

**Affinities to Define:**
```json
{
  "frontend-dev-guidelines": {
    "affinity": ["backend-dev-guidelines"]
  },
  "backend-dev-guidelines": {
    "affinity": ["frontend-dev-guidelines"]
  },
  "rust-developer": {
    "affinity": ["skill-developer"]
  },
  "route-tester": {
    "affinity": ["backend-dev-guidelines"]
  },
  "error-tracking": {
    "affinity": ["backend-dev-guidelines", "rust-developer"]
  }
}
```

---

### Phase 2: AI Intent Analysis Service (Week 3)

**Goal:** Build Axum microservice for AI-powered intent analysis

**Why Axum?** Tokio-based async, type-safe, excellent performance (~500 req/s)

#### Tasks

**2.1: Create intent-analyzer Crate** [Effort: M]

**Acceptance Criteria:**
- New Cargo workspace member: `intent-analyzer/`
- Axum HTTP server on localhost:3030
- Health check endpoint
- Graceful shutdown

**Project Structure:**
```
catalyst/
├── intent-analyzer/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── anthropic.rs       # API client
│   │   ├── analysis.rs        # Intent analysis logic
│   │   └── prompt_template.rs # Intent analysis prompt
│   └── tests/
│       └── integration.rs
```

**Dependencies:**
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
```

**Files to Create:**
- `intent-analyzer/Cargo.toml`
- `intent-analyzer/src/main.rs`

**Basic Server:**
```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(|| async { "OK" }));

    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**Testing:**
- Server starts and responds to /health
- Graceful shutdown works

---

**2.2: Implement Anthropic API Client** [Effort: L]

**Acceptance Criteria:**
- POST to `https://api.anthropic.com/v1/messages`
- Supports Claude Haiku 4.5 (`claude-haiku-4-5`)
- Environment variable: `ANTHROPIC_API_KEY`
- Error handling for API failures
- Timeout: 10 seconds

**API Request:**
```rust
pub struct AnthropicClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl AnthropicClient {
    pub async fn analyze_intent(
        &self,
        prompt: &str,
        skills: &HashMap<String, SkillRule>
    ) -> Result<IntentAnalysis> {
        let request_body = json!({
            "model": "claude-haiku-4-5",
            "max_tokens": 1024,
            "messages": [{
                "role": "user",
                "content": build_analysis_prompt(prompt, skills)
            }]
        });

        let response = self.http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        parse_response(response).await
    }
}
```

**Files to Create:**
- `intent-analyzer/src/anthropic.rs`

**Testing:**
- Mock API responses
- Test error handling (timeout, 401, 500)
- Test successful response parsing

---

**2.3: Port Intent Analysis Prompt** [Effort: M]

**Acceptance Criteria:**
- Rust template equivalent to claude-skills-supercharged's prompt
- Replaces `{{USER_PROMPT}}` and `{{SKILL_DESCRIPTIONS}}`
- Clear confidence threshold guidance
- Multi-domain work detection rules

**Template:**
```rust
pub fn build_analysis_prompt(prompt: &str, skills: &HashMap<String, SkillRule>) -> String {
    let skill_descriptions = skills
        .iter()
        .map(|(name, rule)| format!("- {}: {}", name, rule.description))
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"
Analyze this user prompt and determine which domain skills are relevant for the PRIMARY task.

User prompt: "{}"

Available skills:
{}

IMPORTANT SCORING GUIDANCE:
Confidence Thresholds:
- > 0.65: REQUIRED (auto-injected as critical skill)
- 0.50 to 0.65: SUGGESTED (recommended but not auto-injected)
- < 0.50: IGNORED (not considered relevant)

[... rest of prompt from claude-skills-supercharged ...]

Return JSON with confidence scores (0.0-1.0) for each skill's relevance.

Response format:
{{
  "primary_intent": "brief description of main task",
  "skills": [
    {{"name": "skill-name", "confidence": 0.95, "reason": "why this skill is relevant"}}
  ]
}}
"#, prompt, skill_descriptions)
}
```

**Files to Create:**
- `intent-analyzer/src/prompt_template.rs`
- `intent-analyzer/config/intent-analysis-prompt.txt` (reference)

**Testing:**
- Test prompt generation with sample skills
- Verify template variables replaced correctly

---

**2.4: Implement /analyze Endpoint** [Effort: L]

**Acceptance Criteria:**
- POST `/analyze` accepts JSON: `{prompt, skills, cache_key}`
- Returns JSON: `{required, suggested, scores, from_cache}`
- Checks cache before calling API
- Writes result to cache
- Error responses with proper status codes

**Endpoint:**
```rust
#[derive(Deserialize)]
struct AnalyzeRequest {
    prompt: String,
    skills: HashMap<String, SkillRule>,
    cache_key: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    required: Vec<String>,
    suggested: Vec<String>,
    scores: HashMap<String, f64>,
    from_cache: bool,
}

async fn analyze_handler(
    State(client): State<Arc<AnthropicClient>>,
    Json(req): Json<AnalyzeRequest>
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    // Check cache
    if let Some(cached) = cache.read(&req.cache_key) {
        return Ok(Json(cached.with_cache_flag(true)));
    }

    // Call Anthropic API
    let analysis = client.analyze_intent(&req.prompt, &req.skills).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Categorize by confidence
    let (required, suggested) = categorize_skills(&analysis);

    // Cache result
    cache.write(&req.cache_key, &AnalyzeResponse { required, suggested, scores: analysis.scores, from_cache: false })?;

    Ok(Json(AnalyzeResponse { required, suggested, scores: analysis.scores, from_cache: false }))
}
```

**Files to Modify:**
- `intent-analyzer/src/main.rs`

**Files to Create:**
- `intent-analyzer/src/analysis.rs`

**Testing:**
- Integration test: POST /analyze → AI call → response
- Test cache hit scenario
- Test error handling

---

**2.5: Integrate Axum Service into skill-activation-prompt** [Effort: L]

**Acceptance Criteria:**
- Hook detects if intent-analyzer is running (port 3030)
- Falls back to keyword matching if not running
- HTTP timeout: 200ms (prefer speed over AI)
- Debug flag shows AI vs keyword path

**Detection & Fallback:**
```rust
fn main() -> Result<()> {
    // Check if AI service is available
    let use_ai = env::var("CATALYST_USE_AI").unwrap_or_default() == "1"
        && is_service_available("http://127.0.0.1:3030/health");

    let (required, suggested) = if use_ai {
        analyze_with_ai(&input.prompt, &rules)?
    } else {
        match_skills(&input.prompt, &rules)
    };

    // Rest of logic unchanged
}

fn analyze_with_ai(prompt: &str, rules: &SkillRules) -> Result<(Vec<String>, Vec<String>)> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://127.0.0.1:3030/analyze")
        .timeout(Duration::from_millis(200))
        .json(&json!({
            "prompt": prompt,
            "skills": rules.skills,
            "cache_key": compute_cache_key(prompt, rules)
        }))
        .send()?;

    let result: AnalyzeResponse = response.json()?;
    Ok((result.required, result.suggested))
}
```

**Environment Variable:**
- `CATALYST_USE_AI=1` - Enable AI analysis (default: disabled)

**Files to Modify:**
- `catalyst-cli/src/bin/skill_activation_prompt.rs`

**Testing:**
- Test with AI service running
- Test with AI service not running (fallback)
- Test with AI service timeout (fallback)

---

### Phase 3: Enhanced Session State Management (Week 4)

**Goal:** Improve session state tracking with banners, affinity metadata

#### Tasks

**3.1: Enhance SessionState Schema** [Effort: M]

**Acceptance Criteria:**
- SQLite table tracks acknowledged skills per session
- Tracks affinity injections (distinguish from direct matches)
- Tracks confidence scores (for debug mode)
- Tracks timestamp of skill injection

**Schema:**
```sql
CREATE TABLE acknowledged_skills (
    session_id TEXT NOT NULL,
    skill_name TEXT NOT NULL,
    injected_at INTEGER NOT NULL,        -- Unix timestamp
    injection_type TEXT NOT NULL,        -- 'direct' | 'affinity' | 'promoted'
    confidence REAL,                     -- Optional, for AI path
    PRIMARY KEY (session_id, skill_name)
);

CREATE INDEX idx_session ON acknowledged_skills(session_id);
```

**Files to Create:**
- `catalyst-core/src/session_state.rs`

**Implementation:**
```rust
pub struct SessionStateManager {
    db: Connection,
}

impl SessionStateManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let db = Connection::open(db_path)?;
        db.execute(CREATE_TABLE_SQL, [])?;
        Ok(Self { db })
    }

    pub fn get_acknowledged(&self, session_id: &str) -> Result<Vec<String>> {
        // Query for session_id
    }

    pub fn add_skill(&self, session_id: &str, skill: &AcknowledgedSkill) -> Result<()> {
        // Insert or ignore
    }
}

pub struct AcknowledgedSkill {
    pub name: String,
    pub injection_type: InjectionType,
    pub confidence: Option<f64>,
}

pub enum InjectionType {
    Direct,    // Matched by keywords/AI
    Affinity,  // Injected via affinity
    Promoted,  // Suggested → Required
}
```

**Testing:**
- CRUD operations on acknowledged_skills table
- Query acknowledged skills for session
- Test duplicate handling (PRIMARY KEY)

---

**3.2: Implement Banner Formatting** [Effort: M]

**Acceptance Criteria:**
- Banner shows just-injected skills (with affinity indicators)
- Shows already-loaded skills (if any matched again)
- Shows remaining suggested skills (if any)
- Confidence scores in debug mode

**Banner Format:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📚 AUTO-LOADED SKILLS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎯 INJECTED (2):
  → frontend-dev-guidelines (confidence: 0.92)
  → backend-dev-guidelines (affinity: free bonus)

✅ ALREADY LOADED:
  → rust-developer (loaded earlier this session)

💡 SUGGESTED (optional):
  → error-tracking (confidence: 0.58)
    Use: Skill tool 'error-tracking'

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Files to Create:**
- `catalyst-core/src/output_formatter.rs`

**Implementation:**
```rust
pub struct OutputFormatter;

impl OutputFormatter {
    pub fn format_banner(
        just_injected: &[InjectedSkill],
        already_loaded: &[String],
        suggested: &[(String, f64)],
        debug: bool
    ) -> String {
        let mut output = String::new();

        // Banner header
        output.push_str("━".repeat(50));
        output.push_str("\n📚 AUTO-LOADED SKILLS\n");
        output.push_str("━".repeat(50));

        // Just injected
        if !just_injected.is_empty() {
            output.push_str(&format!("\n\n🎯 INJECTED ({}):\n", just_injected.len()));
            for skill in just_injected {
                let indicator = match skill.injection_type {
                    InjectionType::Affinity => " (affinity: free bonus)",
                    InjectionType::Promoted => " (promoted from suggested)",
                    _ => ""
                };

                let confidence = if debug && skill.confidence.is_some() {
                    format!(" (confidence: {:.2})", skill.confidence.unwrap())
                } else {
                    String::new()
                };

                output.push_str(&format!("  → {}{}{}\n", skill.name, confidence, indicator));
            }
        }

        // Already loaded
        if !already_loaded.is_empty() {
            output.push_str(&format!("\n✅ ALREADY LOADED:\n"));
            for skill in already_loaded {
                output.push_str(&format!("  → {} (loaded earlier this session)\n", skill));
            }
        }

        // Suggested
        if !suggested.is_empty() {
            output.push_str(&format!("\n💡 SUGGESTED (optional):\n"));
            for (skill, confidence) in suggested {
                let confidence_str = if debug {
                    format!(" (confidence: {:.2})", confidence)
                } else {
                    String::new()
                };
                output.push_str(&format!("  → {}{}\n", skill, confidence_str));
                output.push_str(&format!("    Use: Skill tool '{}'\n", skill));
            }
        }

        output.push_str("\n");
        output.push_str("━".repeat(50));
        output
    }
}

pub struct InjectedSkill {
    pub name: String,
    pub injection_type: InjectionType,
    pub confidence: Option<f64>,
}
```

**Testing:**
- Test banner with only injected skills
- Test banner with all sections
- Test debug mode (confidence scores shown)
- Test empty sections

---

**3.3: Update skill-activation-prompt with State Management** [Effort: M]

**Acceptance Criteria:**
- Hook queries session state before matching
- Filters out already-acknowledged skills
- Writes newly injected skills to state
- Outputs formatted banner

**Integration:**
```rust
fn main() -> Result<()> {
    let input = read_stdin()?;
    let rules = load_skill_rules()?;

    // Session state
    let state_dir = dirs::home_dir().unwrap().join(".claude-hooks/state");
    let state = SessionStateManager::new(&state_dir.join("sessions.db"))?;
    let acknowledged = state.get_acknowledged(&input.session_id)?;

    // Intent analysis (AI or keywords)
    let (required, suggested) = analyze_intent(&input.prompt, &rules)?;

    // Filtration
    let filtered = filter_skills(required, suggested, &acknowledged)?;

    // Affinity injection
    let affinity_skills = find_affinity_injections(&filtered.to_inject, &acknowledged, &rules)?;

    // Track injected skills
    let just_injected = [
        filtered.to_inject.iter().map(|s| (s, InjectionType::Direct)),
        affinity_skills.iter().map(|s| (s, InjectionType::Affinity)),
    ].concat();

    for (skill, injection_type) in &just_injected {
        state.add_skill(&input.session_id, &AcknowledgedSkill {
            name: skill.clone(),
            injection_type: *injection_type,
            confidence: None, // TODO: track from AI
        })?;
    }

    // Output banner
    let banner = OutputFormatter::format_banner(
        &just_injected,
        &already_loaded_this_prompt,
        &filtered.remaining_suggested,
        env::var("CATALYST_DEBUG").is_ok()
    );
    println!("{}", banner);

    // Inject skill content (existing logic)
    for skill in just_injected {
        inject_skill_content(&skill.name)?;
    }

    Ok(())
}
```

**Files to Modify:**
- `catalyst-cli/src/bin/skill_activation_prompt.rs`

**Testing:**
- Integration test: prompt → state query → injection → state update → banner
- Test duplicate skill handling (should show in "already loaded")

---

### Phase 4: Comprehensive Testing (Week 5)

**Goal:** Port 120 tests from claude-skills-supercharged, ensure reliability

#### Tasks

**4.1: Set Up Test Infrastructure** [Effort: M]

**Acceptance Criteria:**
- Cargo workspace tests run all unit/integration tests
- Test fixtures directory with sample skill-rules.json
- Mock Anthropic API server for integration tests
- Test coverage reporting (tarpaulin)

**Structure:**
```
catalyst/
├── tests/
│   ├── fixtures/
│   │   ├── skill-rules-test.json
│   │   ├── sample-prompts.json
│   │   └── mock-api-responses.json
│   ├── integration/
│   │   ├── skill_activation.rs
│   │   ├── intent_analyzer.rs
│   │   └── session_state.rs
│   └── common/
│       └── mod.rs  # Test utilities
```

**Files to Create:**
- `tests/common/mod.rs`
- `tests/fixtures/*`
- Mock API server in `intent-analyzer/tests/mock_api.rs`

**Testing:**
- `cargo test` runs all tests
- Mock API responds to /v1/messages

---

**4.2: Cache Manager Tests** [Effort: M]

**Test Cases:**
- Cache write → read returns same result
- TTL expiration (after 1 hour, returns None)
- Cache invalidation on skill-rules.json change
- Atomic writes (no partial files)
- Concurrent access (multiple hooks)

**Files to Create:**
- `catalyst-core/src/cache.rs` (with #[cfg(test)] module)

**Test Count:** ~15 tests

---

**4.3: Affinity Injection Tests** [Effort: L]

**Test Cases:**
- Bidirectional affinity (A→B, loading A injects B)
- Reverse affinity (B lists A, loading A injects B)
- Circular affinity (A↔B)
- Already acknowledged (don't re-inject)
- autoInject: false (don't inject)
- Max affinity limit (2 per skill)
- Affinity chains (A→B→C)
- Free slot cost (doesn't count toward limit)

**Files to Create:**
- `catalyst-core/tests/affinity.rs`

**Test Count:** ~20 tests

---

**4.4: Intent Analysis Tests** [Effort: L]

**Test Cases:**
- Anthropic API success response
- Anthropic API error (401, 500, timeout)
- Confidence threshold categorization (>0.65, 0.50-0.65, <0.50)
- Multi-domain prompts (2+ skills with high confidence)
- Short prompts (<10 words, fallback to keywords)
- Cache hit vs miss
- Skill-developer detection (meta-level prompts)
- Keyword soup handling (ambiguous intent)

**Files to Create:**
- `intent-analyzer/tests/anthropic.rs`
- `intent-analyzer/tests/analysis.rs`

**Test Count:** ~25 tests

---

**4.5: Session State Tests** [Effort: M]

**Test Cases:**
- Add skill to session
- Get acknowledged skills for session
- Duplicate skill (INSERT OR IGNORE)
- Multiple sessions (isolation)
- Injection type tracking (direct vs affinity)
- Confidence score storage
- Cleanup old sessions (>7 days)

**Files to Create:**
- `catalyst-core/tests/session_state.rs`

**Test Count:** ~15 tests

---

**4.6: Output Formatter Tests** [Effort: S]

**Test Cases:**
- Banner with just-injected skills
- Banner with already-loaded skills
- Banner with suggested skills
- Banner with all sections
- Banner with empty sections
- Debug mode (confidence scores)
- Affinity indicators

**Files to Create:**
- `catalyst-core/tests/output_formatter.rs`

**Test Count:** ~10 tests

---

**4.7: Integration Tests** [Effort: XL]

**Test Cases:**
- End-to-end: prompt → AI analysis → affinity → state → banner → injection
- Fallback path: AI unavailable → keyword matching
- Cache hit path: <10ms response
- Multiple prompts in same session (acknowledged tracking)
- skill-rules.json change (cache invalidation)

**Files to Create:**
- `tests/integration/end_to_end.rs`

**Test Count:** ~20 tests

---

**4.8: Performance Benchmarks** [Effort: M]

**Benchmarks:**
- Cache read latency (<1ms)
- Keyword matching (<5ms)
- AI analysis (with mock, <10ms)
- Affinity injection (<1ms)
- Full pipeline (cache hit, <10ms)
- Full pipeline (cache miss + AI, <250ms)

**Files to Create:**
- `benches/skill_activation.rs` (criterion benchmarks)

**Dependencies:**
```toml
[dev-dependencies]
criterion = "0.5"
```

**Running:**
```bash
cargo bench
```

**Test Count:** 6 benchmarks

---

**Total Test Count:** ~120 tests (matching claude-skills-supercharged)

---

### Phase 5: Tooling & UX (Week 6)

**Goal:** CLI commands, /wrap, documentation

**Note on Database Access:** Phase 1-5 use direct SQLite access for performance and reliability. Phase 6 (optional) adds analytics endpoints.

#### Tasks

**5.1: Add `catalyst ai` Subcommand** [Effort: M]

**Acceptance Criteria:**
- `catalyst ai start` - Start intent-analyzer service
- `catalyst ai stop` - Stop service
- `catalyst ai status` - Check if running
- `catalyst ai test` - Test API key

**Commands:**
```bash
# Start service (background)
catalyst ai start

# Check status
catalyst ai status
# Output: ✓ Intent analyzer running (PID 12345)
#         ✓ API key configured

# Stop service
catalyst ai stop

# Test API connection
catalyst ai test --prompt "test prompt"
# Output: ✓ API key valid
#         ✓ Analysis completed (0.92: python-best-practices)
```

**Implementation:**
```rust
pub enum AiCommand {
    Start,
    Stop,
    Status,
    Test { prompt: String },
}

impl AiCommand {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Start => start_intent_analyzer(),
            Self::Stop => stop_intent_analyzer(),
            Self::Status => show_status(),
            Self::Test { prompt } => test_api(prompt),
        }
    }
}

fn start_intent_analyzer() -> Result<()> {
    // Check if already running
    if is_service_running() {
        println!("✓ Intent analyzer already running");
        return Ok(());
    }

    // Spawn intent-analyzer binary
    let binary = dirs::home_dir().unwrap().join(".claude-hooks/bin/intent-analyzer");
    let child = Command::new(binary)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Write PID file
    let pid_file = dirs::home_dir().unwrap().join(".claude-hooks/intent-analyzer.pid");
    fs::write(pid_file, child.id().to_string())?;

    println!("✓ Intent analyzer started (PID {})", child.id());
    Ok(())
}
```

**Files to Modify:**
- `catalyst-cli/src/bin/catalyst.rs` - Add `ai` subcommand

**Files to Create:**
- `catalyst-cli/src/ai_service.rs`

**Testing:**
- Start/stop/status commands
- PID file creation/cleanup
- Test with/without API key

---

**5.2: Create /wrap Slash Command** [Effort: L]

**Acceptance Criteria:**
- `/wrap` updates skill documentation with recent code changes
- Analyzes edited files from session
- Updates skill keywords based on actual usage
- Checks skill file size (<500 lines)
- Updates resource files if needed

**Command:**
```markdown
---
description: Wrap up session and update skill docs
---

Wrap up the current work by following this checklist:

**Skill Maintenance:**
1. Check which files were edited this session (query session state)
2. Identify which skills are relevant to those files
3. For each relevant skill:
   - Check if SKILL.md is still <500 lines
   - Check if keywords match actual code patterns
   - Check if examples are up-to-date
   - Update resource files if needed
4. Run skill activation tests to verify triggers still work

**Code Quality:**
5. Lint the code (pre-commit checks)
6. Run related tests
7. Update documentation

**Commit:**
8. Generate commit message summarizing changes
```

**Files to Create:**
- `.claude/commands/wrap.md`

**Testing:**
- Run /wrap after editing files
- Verify skill recommendations

---

**5.3: Update Documentation** [Effort: L]

**Acceptance Criteria:**
- README.md updated with AI features
- New doc: `docs/ai-intent-analysis.md`
- New doc: `docs/affinity-injection.md`
- Updated: `CLAUDE_INTEGRATION_GUIDE.md`
- Updated: `.claude/hooks/CONFIG.md`

**Documentation Outline:**

**docs/ai-intent-analysis.md:**
- How AI intent analysis works
- Configuration (ANTHROPIC_API_KEY, CATALYST_USE_AI)
- Cost analysis ($1-2/month)
- Fallback behavior
- Prompt engineering tips

**docs/affinity-injection.md:**
- What is affinity injection?
- How to define affinities
- Bidirectional semantics
- Best practices (max 2, meaningful relationships)
- Examples

**Files to Modify:**
- `README.md` - Add AI features section
- `CLAUDE_INTEGRATION_GUIDE.md` - Add AI setup instructions
- `.claude/hooks/CONFIG.md` - Document CATALYST_USE_AI

**Files to Create:**
- `docs/ai-intent-analysis.md`
- `docs/affinity-injection.md`

---

**5.4: Environment Setup Script** [Effort: S]

**Acceptance Criteria:**
- `setup-ai.sh` script configures AI features
- Prompts for ANTHROPIC_API_KEY
- Installs intent-analyzer binary
- Configures environment
- Tests connection

**Script:**
```bash
#!/bin/bash
# setup-ai.sh - Configure AI intent analysis

echo "🤖 Catalyst AI Intent Analysis Setup"
echo ""

# Check if API key exists
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "Enter your Anthropic API key:"
    read -s ANTHROPIC_API_KEY
    export ANTHROPIC_API_KEY

    # Save to ~/.bashrc or ~/.zshrc
    echo "export ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY" >> ~/.bashrc
    echo "✓ API key saved to ~/.bashrc"
fi

# Build intent-analyzer
echo "Building intent-analyzer service..."
cd intent-analyzer && cargo build --release
cp target/release/intent-analyzer ~/.claude-hooks/bin/

# Enable AI in catalyst
echo "export CATALYST_USE_AI=1" >> ~/.bashrc
echo "✓ AI features enabled"

# Test connection
echo "Testing API connection..."
catalyst ai test --prompt "test"

echo ""
echo "✓ Setup complete!"
echo "Start service with: catalyst ai start"
```

**Files to Create:**
- `setup-ai.sh`

**Testing:**
- Run setup script
- Verify API key saved
- Verify binary installed

---

## Risk Assessment and Mitigation Strategies

### Technical Risks

**Risk 1: AI API Cost Overrun**

**Likelihood:** Medium
**Impact:** Medium

**Mitigation:**
- Smart caching (1-hour TTL)
- Keyword fallback for short prompts (<10 words)
- Cost monitoring in `catalyst ai status`
- Environment variable to disable AI (CATALYST_USE_AI=0)
- Use Haiku (cheapest model)

**Cost Estimate:**
- Haiku pricing: $0.25 per 1M input tokens, $1.25 per 1M output tokens
- Average prompt: ~1000 tokens input, ~100 tokens output
- 100 prompts/day: ~$0.15/day
- **Monthly: $1-2** (with caching reducing by ~80%)

---

**Risk 2: AI Service Latency**

**Likelihood:** Low
**Impact:** High (user experience)

**Mitigation:**
- 200ms timeout (fall back to keywords)
- Cache reduces latency to <10ms after first hit
- Axum service can handle concurrent requests
- Optional: deploy service locally (Docker)
- Always show keyword results immediately if AI takes >100ms

---

**Risk 3: Cache Invalidation Bugs**

**Likelihood:** Medium
**Impact:** Low

**Mitigation:**
- Include skill-rules.json hash in cache key
- 1-hour TTL prevents stale cache
- `catalyst cache clear` command
- Comprehensive cache tests

---

**Risk 4: Affinity Circular Dependencies**

**Likelihood:** Low
**Impact:** Low

**Mitigation:**
- Detection in affinity injection logic
- Validation in skill-rules.json parser
- Unit tests for circular cases
- Document best practices (avoid circular)

---

**Risk 5: SQLite Database Corruption**

**Likelihood:** Very Low
**Impact:** Medium

**Mitigation:**
- Atomic writes (WAL mode)
- Regular cleanup (delete sessions >7 days old)
- Backup/restore commands
- Graceful degradation (fallback to in-memory)

---

### Project Risks

**Risk 6: Scope Creep**

**Likelihood:** High
**Impact:** High

**Mitigation:**
- Clear phase boundaries (1-6 weeks)
- MVP first (Phase 1-3), enhancements later (Phase 4-5)
- Defer nice-to-haves (e.g., web dashboard)
- Time-box each task

---

**Risk 7: TypeScript→Rust Translation Complexity**

**Likelihood:** Medium
**Impact:** Medium

**Mitigation:**
- Port tests first (establish correctness baseline)
- Reference claude-skills-supercharged implementation
- Start with simple modules (cache, affinity)
- Leverage Rust's type system (catch bugs at compile time)

---

**Risk 8: Integration Issues with Existing Hooks**

**Likelihood:** Medium
**Impact:** Medium

**Mitigation:**
- Backward compatibility (CATALYST_USE_AI=0 uses old path)
- Feature flags for gradual rollout
- Integration tests covering both paths (AI + keywords)
- Beta testing period before full release

---

## Success Metrics

### Performance Metrics

**Target:** Cache hit <10ms
**Measurement:** Benchmark suite, criterion

**Target:** AI analysis <250ms (p95)
**Measurement:** Request timing in intent-analyzer

**Target:** Keyword fallback <5ms
**Measurement:** Benchmark suite

---

### Accuracy Metrics

**Target:** 95% skill detection accuracy (AI path)
**Measurement:** Manual review of 100 sample prompts

**Target:** 70% skill detection accuracy (keyword path, baseline)
**Measurement:** Manual review

**Target:** Zero false positives (guardrail skills)
**Measurement:** Review frontend-dev-guidelines blocks

---

### Reliability Metrics

**Target:** 120 tests passing
**Measurement:** `cargo test`

**Target:** >80% test coverage
**Measurement:** `cargo tarpaulin`

**Target:** Zero regressions (existing hooks)
**Measurement:** Integration tests

---

### Cost Metrics

**Target:** $1-2/month API cost @ 100 prompts/day
**Measurement:** Anthropic API dashboard

**Target:** <50MB disk space (cache + state)
**Measurement:** `du -sh ~/.claude-hooks/`

---

## Required Resources and Dependencies

### Human Resources

**Developer Time:**
- Week 1-2: 20 hours (Phase 1)
- Week 3: 15 hours (Phase 2)
- Week 4: 12 hours (Phase 3)
- Week 5: 20 hours (Phase 4)
- Week 6: 10 hours (Phase 5)

**Total:** ~77 hours (~2 weeks full-time equivalent)

---

### External Dependencies

**Crates:**
- Axum 0.7 (HTTP server)
- Tokio 1.x (async runtime)
- Reqwest 0.11 (HTTP client)
- Rusqlite 0.31 (SQLite)
- Criterion 0.5 (benchmarking)

**Services:**
- Anthropic API (Claude Haiku 4.5)
- GitHub (for reference implementation)

**API Key:**
- ANTHROPIC_API_KEY (free tier: $5 credit)

---

### Infrastructure

**Development:**
- Rust toolchain 1.70+
- SQLite 3.40+
- 4GB RAM minimum

**Runtime:**
- ~/.claude-hooks/bin/ (binaries)
- ~/.claude-hooks/cache/ (intent analysis)
- ~/.claude-hooks/state/ (session state SQLite)

---

## Timeline Estimates

### Aggressive Timeline (4 weeks)

**Week 1:** Phase 1 (Cache + Affinity)
**Week 2:** Phase 2 (AI Service)
**Week 3:** Phase 3 (Session State) + Phase 4 (Testing)
**Week 4:** Phase 5 (Tooling) + Polish

**Risk:** Tight schedule, requires no blockers

---

### Realistic Timeline (6 weeks)

**Week 1-2:** Phase 1 (Cache + Affinity)
**Week 3:** Phase 2 (AI Service)
**Week 4:** Phase 3 (Session State)
**Week 5:** Phase 4 (Testing)
**Week 6:** Phase 5 (Tooling) + Documentation

**Risk:** Low, allows buffer for debugging

---

### Conservative Timeline (8 weeks)

Add 2 weeks buffer for:
- Unexpected technical challenges
- Performance optimization
- User feedback integration
- Beta testing period

**Recommended:** Start with realistic (6 weeks), extend if needed

---

## Dependencies Between Tasks

### Critical Path

```
Phase 1.1 (Cache)
    ↓
Phase 1.4 (Integrate Cache)
    ↓
Phase 2.1-2.4 (AI Service) ← [Can start in parallel with Phase 1.2-1.3]
    ↓
Phase 2.5 (Integrate AI)
    ↓
Phase 3.1-3.3 (Session State)
    ↓
Phase 4.x (Testing) ← [Blocks release]
    ↓
Phase 5.x (Tooling)
```

**Parallelizable:**
- Phase 1.2-1.3 (Affinity) can run in parallel with Phase 2.1-2.2 (AI Service setup)
- Phase 4.x (Tests) can be written incrementally alongside implementation
- Phase 5.3 (Documentation) can start during Phase 4

---

## Appendix: Architecture Diagrams

### Cache Flow

```
[Prompt] → [Compute MD5 key] → [Check cache]
                                     ↓
                              [Cache exists?]
                              ┌─────┴─────┐
                            YES            NO
                              ↓             ↓
                      [Return cached]  [AI Analysis]
                                             ↓
                                      [Write cache]
                                             ↓
                                        [Return]
```

---

### Affinity Injection Flow

```
[Skills to inject: A, B]
       ↓
[For each skill, check affinity array]
       ↓
[A.affinity = [C, D]]  → Add C, D (if not acknowledged)
       ↓
[Check OTHER skills: do they list A or B?]
       ↓
[E.affinity = [A]] → Add E (reverse affinity)
       ↓
[Return: A, B, C, D, E]
```

---

### AI Intent Analysis Request

```
POST https://api.anthropic.com/v1/messages
Headers:
  x-api-key: $ANTHROPIC_API_KEY
  anthropic-version: 2023-06-01
Body:
{
  "model": "claude-haiku-4-5",
  "max_tokens": 1024,
  "messages": [{
    "role": "user",
    "content": "Analyze this prompt: ...\n\nAvailable skills: ..."
  }]
}

Response:
{
  "content": [{
    "text": "{
      \"primary_intent\": \"...\",
      \"skills\": [
        {\"name\": \"frontend-dev-guidelines\", \"confidence\": 0.92},
        {\"name\": \"backend-dev-guidelines\", \"confidence\": 0.58}
      ]
    }"
  }]
}
```

---

### Session State Database

```
sessions.db (SQLite)

Table: acknowledged_skills
┌────────────┬────────────┬──────────────┬─────────────────┬────────────┐
│ session_id │ skill_name │ injected_at  │ injection_type  │ confidence │
├────────────┼────────────┼──────────────┼─────────────────┼────────────┤
│ sess-123   │ frontend   │ 1699123456   │ direct          │ 0.92       │
│ sess-123   │ backend    │ 1699123456   │ affinity        │ NULL       │
│ sess-123   │ rust-dev   │ 1699123500   │ direct          │ 0.85       │
└────────────┴────────────┴──────────────┴─────────────────┴────────────┘

Indexes:
- PRIMARY KEY (session_id, skill_name)
- INDEX idx_session ON acknowledged_skills(session_id)
```

---

## Phase 6: Analytics API (Optional Future Enhancement)

**Goal:** Add optional analytics endpoints to intent-analyzer API

**Status:** Optional - Not required for MVP
**Estimated Duration:** 1 week (if implemented)

**Context:** Phase 1-5 use direct SQLite access for performance. This phase adds read-only analytics endpoints for optional features like web dashboards.

### Architecture Decision: Hybrid Approach

**Critical Path (Always Direct):**
```rust
skill-activation-prompt (Rust hook)
    ↓ Direct SQLite (~1-2ms)
Session State DB
```

**Optional Analytics (Through API):**
```rust
Web Dashboard / CLI stats
    ↓ HTTP
intent-analyzer (Axum API)
    ↓ Read-only SQLite
Session State DB
```

**Why Hybrid?**
- ✅ Hook stays fast (<50ms) with direct access
- ✅ Works offline (no API dependency for core functionality)
- ✅ API enables future features (web dashboard, cross-machine sync)
- ✅ Non-breaking enhancement (add later without changes to hook)

---

### Tasks

**6.1: Design Analytics Schema** [Effort: S]

**Acceptance Criteria:**
- Document read-only access pattern
- Define analytics queries (most-used skills, usage trends)
- Plan indexes for efficient analytics queries

**Queries to Support:**
```sql
-- Most-used skills (last 7 days)
SELECT skill_name, COUNT(*) as uses
FROM acknowledged_skills
WHERE injected_at > datetime('now', '-7 days')
GROUP BY skill_name
ORDER BY uses DESC;

-- Session activity (last 30 days)
SELECT DATE(injected_at) as date, COUNT(DISTINCT session_id) as sessions
FROM acknowledged_skills
WHERE injected_at > datetime('now', '-30 days')
GROUP BY date;

-- Average confidence scores by skill
SELECT skill_name, AVG(confidence) as avg_confidence, COUNT(*) as uses
FROM acknowledged_skills
WHERE confidence IS NOT NULL
GROUP BY skill_name;
```

---

**6.2: Implement Read-Only Database Connection** [Effort: S]

**Acceptance Criteria:**
- API opens database in read-only mode
- Proper error handling if DB locked
- Connection pooling for analytics queries

**Implementation:**
```rust
// intent-analyzer/src/db.rs
use rusqlite::{Connection, OpenFlags};

pub struct AnalyticsDb {
    conn: Connection,
}

impl AnalyticsDb {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY
        )?;

        // Enable query optimization
        conn.execute("PRAGMA query_only = ON", [])?;

        Ok(Self { conn })
    }
}
```

---

**6.3: Implement Analytics Endpoints** [Effort: M]

**Acceptance Criteria:**
- GET /api/stats/skills - Most-used skills
- GET /api/stats/sessions - Session activity over time
- GET /api/stats/confidence - Average confidence by skill
- GET /api/sessions/:id - Detail view of specific session
- JSON responses with proper error handling

**Implementation:**
```rust
// intent-analyzer/src/analytics.rs

#[derive(Serialize)]
pub struct SkillStats {
    pub skill_name: String,
    pub total_uses: u32,
    pub avg_confidence: Option<f64>,
    pub last_used: i64, // Unix timestamp
}

#[derive(Serialize)]
pub struct SessionActivity {
    pub date: String,
    pub sessions: u32,
    pub skills_injected: u32,
}

// GET /api/stats/skills
pub async fn skills_stats(
    State(db): State<Arc<AnalyticsDb>>
) -> Result<Json<Vec<SkillStats>>, StatusCode> {
    let stats = db.query_skills_stats()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stats))
}

// GET /api/stats/sessions?days=30
pub async fn sessions_stats(
    State(db): State<Arc<AnalyticsDb>>,
    Query(params): Query<StatsParams>
) -> Result<Json<Vec<SessionActivity>>, StatusCode> {
    let days = params.days.unwrap_or(30);
    let stats = db.query_session_activity(days)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stats))
}

// GET /api/sessions/:id
pub async fn session_detail(
    State(db): State<Arc<AnalyticsDb>>,
    Path(session_id): Path<String>
) -> Result<Json<SessionDetail>, StatusCode> {
    let detail = db.query_session_detail(&session_id)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(detail))
}
```

**Files to Create:**
- `intent-analyzer/src/db.rs`
- `intent-analyzer/src/analytics.rs`

---

**6.4: Add Analytics CLI Commands** [Effort: S]

**Acceptance Criteria:**
- `catalyst stats skills` - Show skill usage
- `catalyst stats sessions` - Show session activity
- `catalyst stats export` - Export to JSON/CSV

**Implementation:**
```rust
// catalyst-cli/src/stats.rs

pub enum StatsCommand {
    Skills { days: u32 },
    Sessions { days: u32 },
    Export { format: ExportFormat, output: PathBuf },
}

impl StatsCommand {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Skills { days } => {
                // Read directly from SQLite (no API needed)
                let db = SessionStateManager::new_direct(&db_path)?;
                let stats = db.get_skill_stats(*days)?;
                display_skills_table(stats);
            }
            Self::Sessions { days } => {
                let db = SessionStateManager::new_direct(&db_path)?;
                let stats = db.get_session_stats(*days)?;
                display_sessions_chart(stats);
            }
            Self::Export { format, output } => {
                let db = SessionStateManager::new_direct(&db_path)?;
                let data = db.export_all_data()?;
                write_export(data, format, output)?;
            }
        }
        Ok(())
    }
}
```

**Example Output:**
```bash
$ catalyst stats skills --days 7

📊 Skill Usage (Last 7 Days)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Skill                         Uses  Avg Confidence
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
rust-developer                  45      0.89
backend-dev-guidelines          32      0.85
frontend-dev-guidelines         28      0.87
skill-developer                 12      0.92
error-tracking                   8      0.78
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total Sessions: 67
```

---

**6.5: Web Dashboard (Optional)** [Effort: XL]

**Acceptance Criteria:**
- Simple HTML/JS dashboard served by API
- Real-time skill usage visualization
- Session timeline view
- Export functionality

**Tech Stack:**
- Serve static HTML from API
- Chart.js for visualizations
- Fetch data from analytics endpoints

**Implementation:**
```rust
// intent-analyzer/src/dashboard.rs

pub fn dashboard_routes() -> Router {
    Router::new()
        .route("/", get(serve_dashboard_html))
        .route("/static/*path", get(serve_static))
        .nest("/api", analytics_routes())
}

async fn serve_dashboard_html() -> Html<String> {
    Html(include_str!("../static/dashboard.html").to_string())
}
```

**Dashboard Features:**
- 📊 Most-used skills (bar chart)
- 📈 Usage trends (line chart)
- 🔍 Session search
- 💾 Export to CSV/JSON

**Access:**
```bash
$ catalyst ai start
$ open http://localhost:3030/dashboard
```

---

### Benefits of Phase 6

**User Benefits:**
- 📊 Understand which skills are most useful
- 📈 Track skill adoption over time
- 🔍 Debug why certain skills aren't activating
- 💾 Export data for external analysis

**Developer Benefits:**
- 📚 Understand catalyst usage patterns
- 🎯 Prioritize skill improvements based on data
- 🐛 Debug skill activation issues with real data

**Enterprise Benefits:**
- 👥 Team-wide skill analytics
- 📊 ROI tracking for AI features
- 🔍 Compliance reporting (what skills are used where)

---

### Implementation Priority

**Required (Phase 1-5):**
- ✅ Direct SQLite access for hook (performance)
- ✅ AI intent analysis (core feature)
- ✅ Caching and affinity (optimization)
- ✅ Session state tracking (basic)

**Optional (Phase 6):**
- ⭐ Analytics endpoints (nice-to-have)
- ⭐ CLI stats commands (useful for debugging)
- ⭐⭐ Web dashboard (visual, but not essential)

**Recommendation:** Implement Phase 6 only after Phase 1-5 are proven and stable. The core system doesn't need it, but it enables powerful insights for advanced users.

---

## Updated Architecture with Phase 6

```
┌──────────────────────────────────────────────┐
│ Users                                        │
└────┬─────────────────────────────────────┬──┘
     │                                      │
     │ CLI / Hook (Critical Path)           │ Browser (Optional)
     ↓                                      ↓
┌─────────────────────┐          ┌─────────────────────┐
│ skill-activation    │          │ Web Dashboard       │
│ (Direct DB)         │          │ (Phase 6 - Optional)│
│ • Fast path         │          │ • Charts/graphs     │
│ • Works offline     │          │ • Export data       │
│ • <50ms latency     │          │ • Search sessions   │
└──────────┬──────────┘          └──────────┬──────────┘
           │                                 │
           │ HTTP (AI only)                  │ HTTP (analytics)
           │ Phase 2                         │ Phase 6
           ↓                                 ↓
┌──────────────────────────────────────────────────────┐
│ intent-analyzer (Axum API)                           │
│                                                      │
│ Phase 2-5 (Required):                                │
│ ├─ POST /analyze (AI providers)                      │
│ └─ Pluggable provider system                         │
│                                                      │
│ Phase 6 (Optional):                                  │
│ ├─ GET /api/stats/skills                            │
│ ├─ GET /api/stats/sessions                          │
│ ├─ GET /api/sessions/:id                            │
│ └─ GET /dashboard                                    │
└──────────────┬───────────────────────────────────────┘
               │
               ↓ (read-only, Phase 6)
┌──────────────────────────────────────────────────────┐
│ Session State DB (SQLite)                            │
│                                                      │
│ Primary writes: Hook (direct) - Phase 3             │
│ Optional reads: API (analytics) - Phase 6           │
└──────────────────────────────────────────────────────┘
```

---

**End of Plan**
**Last Updated:** 2025-11-11

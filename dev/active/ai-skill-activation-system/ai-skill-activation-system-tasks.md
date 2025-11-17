# AI-Powered Skill Activation System - Task Checklist

**Last Updated:** 2025-11-11
**Status:** Planning Phase

---

## Phase 0: Pre-Implementation Research ⏳ NOT STARTED

**Goal:** Validate assumptions and clarify ambiguities before implementation
**Estimated Duration:** 1-2 days
**Why First:** Increases confidence from ~70% to 95%+ across all phases

### 0.1: Test llama.cpp Integration [Effort: M] [2 hours]

- [ ] Set up llama.cpp with server support
- [ ] Load gpt-oss model (GGUF format)
- [ ] Load qwen3-coder model (GGUF format)
- [ ] Test OpenAI-compatible API endpoint
- [ ] Verify JSON response format
- [ ] Measure response time (<200ms target)
- [ ] Test context window with full intent prompt
- [ ] Document findings (response_format support, quirks)
- [ ] Compare gpt-oss vs qwen3-coder performance

**Acceptance Criteria:**

- ✓ llama.cpp responds <200ms
- ✓ JSON output validated
- ✓ Quirks documented
- ✓ Both models tested

---

### 0.2: Clarify Affinity Algorithm [Effort: S] [1 hour]

- [ ] Write detailed algorithm pseudocode
- [ ] Define cycle detection logic (HashSet)
- [ ] Specify hard limits (MAX_DEPTH=2, MAX_AFFINITIES=10)
- [ ] Document edge cases
  - [ ] Circular: A↔B
  - [ ] Chain: A→B→C→D
  - [ ] Diamond: A→B,C B,C→D
  - [ ] Max limit reached
- [ ] Add algorithm to plan.md Phase 1.3

**Acceptance Criteria:**

- ✓ Algorithm handles all edge cases
- ✓ No infinite loop potential
- ✓ Hard limits defined

---

### 0.3: Define Configuration Precedence [Effort: S] [30 min]

- [ ] Document precedence order (CLI > Env > Config > Default)
- [ ] Add examples for each scenario
- [ ] Write resolve_provider() pseudocode
- [ ] Update context.md Decision 2

**Acceptance Criteria:**

- ✓ Precedence clear and unambiguous
- ✓ Examples cover all cases

---

### 0.4: Add Error Handling Spec [Effort: S] [1 hour]

- [ ] Define error banner formats
- [ ] Specify retry logic (MAX_RETRIES=2, DELAY=50ms)
- [ ] Document cache behavior on errors
- [ ] Add banner indicators (keyword vs AI)
- [ ] Update plan.md Phase 2.5

**Acceptance Criteria:**

- ✓ All error scenarios covered
- ✓ Retry logic specified
- ✓ Cache behavior clear

---

### 0.5: Research Windows Process Management [Effort: S] [1 hour]

- [ ] Test process spawning on Windows
- [ ] Verify background detachment
- [ ] Test PID file creation/reading
- [ ] Test process survives terminal close
- [ ] Write platform-specific code (#[cfg(windows)])
- [ ] Update plan.md Phase 5.1

**Acceptance Criteria:**

- ✓ Windows detachment works
- ✓ Cross-platform code written
- ✓ PID management works

---

### 0.6: SQLite Concurrency Testing [Effort: S] [1 hour]

- [ ] Test WAL mode concurrent reads
- [ ] Test write locking behavior
- [ ] Implement retry logic for SQLITE_BUSY
- [ ] Test multi-session scenarios
- [ ] Document WAL configuration
- [ ] Update plan.md Phase 3.1

**Acceptance Criteria:**

- ✓ Concurrent access validated
- ✓ Retry logic works
- ✓ Multi-session supported

---

### 0.7: Test Prompt Across Providers [Effort: M] [2 hours]

- [ ] Test with Anthropic Haiku (baseline)
- [ ] Test with llama.cpp gpt-oss
- [ ] Test with llama.cpp qwen3-coder
- [ ] Test with OpenAI GPT-3.5 (if available)
- [ ] Document findings per provider
- [ ] Create provider-specific variants if needed
- [ ] Update plan.md Phase 2.3

**Acceptance Criteria:**

- ✓ All providers return valid JSON
- ✓ Response times measured
- ✓ Quirks documented
- ✓ Adjustments identified

---

## Phase 1: Foundation - Caching & Affinity ⏳ NOT STARTED

**Goal:** Add caching and affinity without AI dependency
**Estimated Duration:** 1-2 weeks

### 1.1: Implement Cache Manager (Rust) [Effort: M]

- [ ] Create `catalyst-core/src/cache.rs` module
- [ ] Implement `CacheManager` struct with read/write methods
- [ ] Implement MD5-based cache key generation
- [ ] Implement 1-hour TTL expiration checking
- [ ] Implement atomic writes using tempfile
- [ ] Add cache directory creation (~/.claude-hooks/cache/intent-analysis/)
- [ ] Write unit tests for cache operations
  - [ ] Test cache write → read
  - [ ] Test TTL expiration
  - [ ] Test skill-rules.json hash invalidation
  - [ ] Test atomic writes
  - [ ] Test concurrent access
- [ ] Benchmark cache read (<1ms target)
- [ ] Document cache format and location

**Acceptance Criteria:**

- ✓ Cache stored in `~/.claude-hooks/cache/intent-analysis/`
- ✓ MD5 hash of (prompt + skill_rules_hash)
- ✓ 1-hour TTL with automatic expiration
- ✓ Cache invalidated when skill-rules.json changes
- ✓ Atomic cache writes (tempfile + rename)
- ✓ 15 tests passing
- ✓ Cache read <1ms

---

### 1.2: Add Affinity Field to skill-rules.json [Effort: S]

- [ ] Update `SkillRule` struct in `catalyst-cli/src/types.rs`
- [ ] Add `affinity: Option<Vec<String>>` field
- [ ] Update serde deserialization
- [ ] Add validation in `catalyst-cli/src/validation.rs`
  - [ ] Maximum 2 affinity skills per skill
  - [ ] Affinity skills must exist in skill-rules.json
- [ ] Write unit tests
  - [ ] Parse skill-rules.json with affinity field
  - [ ] Reject >2 affinities
  - [ ] Accept 0-2 affinities
  - [ ] Reject non-existent skill names
- [ ] Document affinity semantics in code comments

**Acceptance Criteria:**

- ✓ `affinity` field added to SkillRule
- ✓ Maximum 2 affinity skills validated
- ✓ 4 tests passing
- ✓ Code documentation added

---

### 1.3: Implement Affinity Injection Logic (Rust) [Effort: L]

- [ ] Create `catalyst-core/src/affinity.rs` module
- [ ] Implement `find_affinity_injections()` function
- [ ] Implement bidirectional affinity detection
  - [ ] Direction 1: skill → affinity array
  - [ ] Direction 2: other skills → skill (reverse lookup)
- [ ] Filter out already-acknowledged skills
- [ ] Filter out skills with autoInject: false
- [ ] Write unit tests
  - [ ] Bidirectional detection (A→B)
  - [ ] Reverse detection (B lists A in affinity)
  - [ ] Circular affinity (A↔B)
  - [ ] Affinity chains (A→B, B→C)
  - [ ] Already acknowledged filtering
  - [ ] autoInject: false filtering
  - [ ] Free slot cost verification
  - [ ] Max affinity limit (2)
- [ ] Benchmark affinity injection (<1ms target)
- [ ] Document algorithm in code comments

**Acceptance Criteria:**

- ✓ Bidirectional affinity detection works
- ✓ Already-acknowledged skills filtered out
- ✓ autoInject flag respected
- ✓ Affinity skills don't count toward 2-skill limit
- ✓ 20 tests passing
- ✓ Affinity injection <1ms

---

### 1.4: Integrate Cache + Affinity into skill-activation-prompt [Effort: M]

- [ ] Add cache manager to `catalyst-cli/src/bin/skill_activation_prompt.rs`
- [ ] Compute skills hash from skill-rules.json
- [ ] Check cache before keyword matching
- [ ] Integrate affinity injection after skill filtration
- [ ] Write cache on keyword matching results
- [ ] Add debug logging for cache hits/misses
- [ ] Add debug logging for affinity decisions
- [ ] Write integration tests
  - [ ] Test cache hit scenario
  - [ ] Test cache miss scenario
  - [ ] Test affinity injection
  - [ ] Test combined cache + affinity
- [ ] Update hook to show affinity skills in output

**Acceptance Criteria:**

- ✓ Hook checks cache before matching
- ✓ Cache stores keyword matching results
- ✓ Affinity injection runs after filtration
- ✓ Debug logging shows decisions
- ✓ 4 integration tests passing
- ✓ Hook output shows affinity indicators

---

### 1.5: Update skill-rules.json with Affinity Relationships [Effort: S]

- [ ] Define affinity for frontend-dev-guidelines
- [ ] Define affinity for backend-dev-guidelines
- [ ] Define affinity for rust-developer
- [ ] Define affinity for route-tester
- [ ] Define affinity for error-tracking
- [ ] Define affinity for skill-developer (if appropriate)
- [ ] Ensure bidirectional where appropriate
- [ ] Validate all affinities (max 2 per skill)
- [ ] Test skill activation with new affinities
- [ ] Document affinity rationale in comments

**Acceptance Criteria:**

- ✓ All catalyst skills have meaningful affinities
- ✓ Bidirectional where appropriate
- ✓ Maximum 2 per skill
- ✓ Validation passes
- ✓ Skills activate with affinity as expected

---

## Phase 2: AI Intent Analysis Service ⏳ NOT STARTED

**Goal:** Build Axum microservice for AI-powered intent analysis
**Estimated Duration:** 1 week

### 2.1: Create intent-analyzer Crate [Effort: M]

- [ ] Create `intent-analyzer/` directory
- [ ] Create `intent-analyzer/Cargo.toml`
- [ ] Add to workspace in root `Cargo.toml`
- [ ] Create `intent-analyzer/src/main.rs` with basic Axum server
- [ ] Implement `/health` endpoint
- [ ] Implement graceful shutdown
- [ ] Test server starts and responds
- [ ] Add tracing/logging
- [ ] Write integration tests
  - [ ] Server starts successfully
  - [ ] /health endpoint responds
  - [ ] Graceful shutdown works

**Acceptance Criteria:**

- ✓ New workspace member created
- ✓ Axum server runs on localhost:3030
- ✓ /health endpoint returns "OK"
- ✓ 3 tests passing

---

### 2.2: Implement Anthropic API Client [Effort: L]

- [ ] Create `intent-analyzer/src/anthropic.rs`
- [ ] Implement `AnthropicClient` struct
- [ ] Implement `analyze_intent()` method
- [ ] Handle API request/response
- [ ] Implement error handling (timeout, 401, 500)
- [ ] Read ANTHROPIC_API_KEY from environment
- [ ] Add 10-second timeout
- [ ] Write unit tests with mocked API
  - [ ] Successful response parsing
  - [ ] Timeout handling
  - [ ] 401 Unauthorized handling
  - [ ] 500 Server Error handling
  - [ ] Invalid JSON response handling
- [ ] Document API client usage

**Acceptance Criteria:**

- ✓ POST to Anthropic API works
- ✓ Supports Claude Haiku 4.5
- ✓ Environment variable for API key
- ✓ Error handling for failures
- ✓ 10-second timeout
- ✓ 5 tests passing

---

### 2.3: Port Intent Analysis Prompt [Effort: M]

- [ ] Create `intent-analyzer/src/prompt_template.rs`
- [ ] Implement `build_analysis_prompt()` function
- [ ] Port prompt template from claude-skills-supercharged
- [ ] Implement template variable replacement
  - [ ] {{USER_PROMPT}} → actual prompt
  - [ ] {{SKILL_DESCRIPTIONS}} → formatted skills
- [ ] Add confidence threshold guidance
- [ ] Add multi-domain work detection rules
- [ ] Add skill-developer detection rules
- [ ] Write unit tests
  - [ ] Test prompt generation with sample skills
  - [ ] Test template variable replacement
- [ ] Store reference prompt in `config/intent-analysis-prompt.txt`

**Acceptance Criteria:**

- ✓ Rust template equivalent to TypeScript version
- ✓ Template variables replaced correctly
- ✓ Guidance included in prompt
- ✓ 2 tests passing

---

### 2.4: Implement /analyze Endpoint [Effort: L]

- [ ] Create `intent-analyzer/src/analysis.rs`
- [ ] Define `AnalyzeRequest` struct
- [ ] Define `AnalyzeResponse` struct
- [ ] Implement `analyze_handler()` function
- [ ] Integrate cache checking
- [ ] Call Anthropic API via client
- [ ] Categorize skills by confidence thresholds
  - [ ] >0.65 → required
  - [ ] 0.50-0.65 → suggested
  - [ ] <0.50 → ignored
- [ ] Write result to cache
- [ ] Return JSON response
- [ ] Write integration tests
  - [ ] POST /analyze → AI call → response
  - [ ] Cache hit scenario
  - [ ] Cache miss scenario
  - [ ] Error handling (API failure)
- [ ] Add request/response logging

**Acceptance Criteria:**

- ✓ POST /analyze endpoint works
- ✓ Returns JSON with required/suggested/scores
- ✓ Checks cache before API call
- ✓ Writes result to cache
- ✓ 4 tests passing

---

### 2.5: Integrate Axum Service into skill-activation-prompt [Effort: L]

- [ ] Add HTTP client dependency (reqwest)
- [ ] Implement service availability check (port 3030)
- [ ] Implement `analyze_with_ai()` function
- [ ] Add 200ms timeout for AI requests
- [ ] Implement fallback to keyword matching
- [ ] Add CATALYST_USE_AI environment variable
- [ ] Add debug flag for AI vs keyword path
- [ ] Write integration tests
  - [ ] Test with AI service running
  - [ ] Test with AI service not running (fallback)
  - [ ] Test with AI service timeout (fallback)
  - [ ] Test cache integration
- [ ] Document environment variables

**Acceptance Criteria:**

- ✓ Hook detects if intent-analyzer running
- ✓ Falls back to keywords if unavailable
- ✓ 200ms timeout enforced
- ✓ Debug flag shows which path used
- ✓ 4 tests passing

---

## Phase 3: Enhanced Session State Management ⏳ NOT STARTED

**Goal:** Improve session state tracking with banners, affinity metadata
**Estimated Duration:** 1 week

### 3.1: Enhance SessionState Schema [Effort: M]

- [ ] Create `catalyst-core/src/session_state.rs`
- [ ] Define SQLite schema for acknowledged_skills table
  - [ ] session_id, skill_name (PRIMARY KEY)
  - [ ] injected_at (timestamp)
  - [ ] injection_type (direct/affinity/promoted)
  - [ ] confidence (optional)
- [ ] Create indexes (session_id)
- [ ] Implement `SessionStateManager` struct
- [ ] Implement `get_acknowledged()` method
- [ ] Implement `add_skill()` method
- [ ] Implement database initialization
- [ ] Write unit tests
  - [ ] CRUD operations
  - [ ] Query acknowledged skills
  - [ ] Duplicate handling (PRIMARY KEY)
  - [ ] Multiple sessions (isolation)
  - [ ] Injection type tracking
  - [ ] Confidence score storage
- [ ] Implement cleanup (>7 days old)

**Acceptance Criteria:**

- ✓ SQLite schema defined
- ✓ Indexes created
- ✓ SessionStateManager implemented
- ✓ 15 tests passing

---

### 3.2: Implement Banner Formatting [Effort: M]

- [ ] Create `catalyst-core/src/output_formatter.rs`
- [ ] Implement `OutputFormatter` struct
- [ ] Implement `format_banner()` method
- [ ] Format just-injected skills section
  - [ ] Show affinity indicators
  - [ ] Show promoted indicators
  - [ ] Show confidence scores (debug mode)
- [ ] Format already-loaded skills section
- [ ] Format suggested skills section
- [ ] Add emoji indicators (📚 🎯 ✅ 💡)
- [ ] Add debug mode flag
- [ ] Write unit tests
  - [ ] Banner with only injected
  - [ ] Banner with all sections
  - [ ] Banner with empty sections
  - [ ] Debug mode (confidence shown)
  - [ ] Affinity indicators shown
- [ ] Test output formatting

**Acceptance Criteria:**

- ✓ Banner shows just-injected skills
- ✓ Banner shows already-loaded skills
- ✓ Banner shows suggested skills
- ✓ Affinity/promoted indicators work
- ✓ Debug mode shows confidence
- ✓ 10 tests passing

---

### 3.3: Update skill-activation-prompt with State Management [Effort: M]

- [ ] Integrate SessionStateManager into hook
- [ ] Query acknowledged skills for session
- [ ] Filter out acknowledged before injection
- [ ] Track newly injected skills to state
- [ ] Track injection type (direct/affinity/promoted)
- [ ] Track confidence scores (if AI path)
- [ ] Detect already-loaded skills in current prompt
- [ ] Format banner with OutputFormatter
- [ ] Output banner before skill injection
- [ ] Write integration tests
  - [ ] Prompt → state query → injection → state update → banner
  - [ ] Duplicate skill handling (already loaded)
  - [ ] Multiple prompts in same session
  - [ ] Affinity metadata tracking
- [ ] Add CATALYST_DEBUG environment variable

**Acceptance Criteria:**

- ✓ Hook queries session state
- ✓ Filters acknowledged skills
- ✓ Writes newly injected to state
- ✓ Outputs formatted banner
- ✓ 4 integration tests passing

---

## Phase 4: Comprehensive Testing ⏳ NOT STARTED

**Goal:** Port 120 tests from claude-skills-supercharged, ensure reliability
**Estimated Duration:** 1 week

### 4.1: Set Up Test Infrastructure [Effort: M]

- [ ] Create `tests/` directory structure
- [ ] Create `tests/fixtures/` with sample data
  - [ ] skill-rules-test.json
  - [ ] sample-prompts.json
  - [ ] mock-api-responses.json
- [ ] Create `tests/common/mod.rs` with utilities
- [ ] Create mock Anthropic API server in `intent-analyzer/tests/`
- [ ] Set up test coverage reporting (tarpaulin)
- [ ] Configure CI/CD to run tests
- [ ] Verify `cargo test` runs all tests

**Acceptance Criteria:**

- ✓ Test directory structure created
- ✓ Fixtures available
- ✓ Mock API server works
- ✓ `cargo test` runs successfully

---

### 4.2: Cache Manager Tests [Effort: M]

- [ ] Test cache write → read
- [ ] Test TTL expiration (after 1 hour)
- [ ] Test skill-rules.json hash invalidation
- [ ] Test atomic writes (no partial files)
- [ ] Test concurrent access
- [ ] Test cache directory creation
- [ ] Test malformed cache files
- [ ] Test cache cleanup

**Acceptance Criteria:**

- ✓ 15 cache tests passing
- ✓ All edge cases covered

---

### 4.3: Affinity Injection Tests [Effort: L]

- [ ] Test bidirectional affinity (A→B)
- [ ] Test reverse affinity (B lists A)
- [ ] Test circular affinity (A↔B)
- [ ] Test affinity chains (A→B→C)
- [ ] Test already acknowledged filtering
- [ ] Test autoInject: false filtering
- [ ] Test max affinity limit (2)
- [ ] Test free slot cost
- [ ] Test multiple affinities
- [ ] Test empty affinities
- [ ] Test non-existent affinity skills

**Acceptance Criteria:**

- ✓ 20 affinity tests passing
- ✓ All bidirectional cases covered

---

### 4.4: Intent Analysis Tests [Effort: L]

- [ ] Test Anthropic API success response
- [ ] Test API errors (401, 500, timeout)
- [ ] Test confidence threshold categorization
- [ ] Test multi-domain prompts
- [ ] Test short prompts (<10 words, fallback)
- [ ] Test cache hit vs miss
- [ ] Test skill-developer detection
- [ ] Test keyword soup handling
- [ ] Test prompt template generation
- [ ] Test confidence scoring edge cases

**Acceptance Criteria:**

- ✓ 25 intent analysis tests passing
- ✓ Mock API covers all scenarios

---

### 4.5: Session State Tests [Effort: M]

- [ ] Test add skill to session
- [ ] Test get acknowledged skills
- [ ] Test duplicate skill (INSERT OR IGNORE)
- [ ] Test multiple sessions (isolation)
- [ ] Test injection type tracking
- [ ] Test confidence score storage
- [ ] Test cleanup old sessions (>7 days)
- [ ] Test database initialization
- [ ] Test database corruption handling

**Acceptance Criteria:**

- ✓ 15 session state tests passing
- ✓ SQLite edge cases covered

---

### 4.6: Output Formatter Tests [Effort: S]

- [ ] Test banner with just-injected
- [ ] Test banner with already-loaded
- [ ] Test banner with suggested
- [ ] Test banner with all sections
- [ ] Test banner with empty sections
- [ ] Test debug mode (confidence scores)
- [ ] Test affinity indicators
- [ ] Test promoted indicators

**Acceptance Criteria:**

- ✓ 10 output formatter tests passing
- ✓ All banner sections tested

---

### 4.7: Integration Tests [Effort: XL]

- [ ] Test end-to-end: prompt → AI → affinity → state → banner → injection
- [ ] Test fallback path: AI unavailable → keyword matching
- [ ] Test cache hit path (<10ms response)
- [ ] Test multiple prompts in same session
- [ ] Test skill-rules.json change (cache invalidation)
- [ ] Test AI timeout (fallback)
- [ ] Test session state persistence
- [ ] Test affinity injection in full pipeline

**Acceptance Criteria:**

- ✓ 20 integration tests passing
- ✓ Both AI and keyword paths tested

---

### 4.8: Performance Benchmarks [Effort: M]

- [ ] Set up Criterion benchmarks
- [ ] Benchmark cache read (<1ms)
- [ ] Benchmark keyword matching (<5ms)
- [ ] Benchmark AI analysis (mock, <10ms)
- [ ] Benchmark affinity injection (<1ms)
- [ ] Benchmark full pipeline (cache hit, <10ms)
- [ ] Benchmark full pipeline (AI, <250ms)
- [ ] Add CI/CD benchmark tracking
- [ ] Document benchmark results

**Acceptance Criteria:**

- ✓ 6 benchmarks defined
- ✓ All performance targets met

---

**Total Phase 4 Tests:** ~120 tests

---

## Phase 5: Tooling & UX ⏳ NOT STARTED

**Goal:** CLI commands, /wrap, documentation
**Estimated Duration:** 1 week

### 5.1: Add `catalyst ai` Subcommand [Effort: M]

- [ ] Add `ai` subcommand to `catalyst-cli/src/bin/catalyst.rs`
- [ ] Create `catalyst-cli/src/ai_service.rs` module
- [ ] Implement `catalyst ai start` command
- [ ] Implement `catalyst ai stop` command
- [ ] Implement `catalyst ai status` command
- [ ] Implement `catalyst ai test` command
- [ ] Add PID file tracking
- [ ] Implement service detection
- [ ] Write integration tests
  - [ ] Start/stop/status commands
  - [ ] PID file creation/cleanup
  - [ ] Test with/without API key
- [ ] Document commands

**Acceptance Criteria:**

- ✓ All `ai` subcommands work
- ✓ Service management functional
- ✓ Tests passing

---

### 5.2: Create /wrap Slash Command [Effort: L]

- [ ] Create `.claude/commands/wrap.md`
- [ ] Define wrap workflow
  - [ ] Query session state for edited files
  - [ ] Identify relevant skills
  - [ ] Check SKILL.md size (<500 lines)
  - [ ] Check keyword relevance
  - [ ] Update resource files if needed
  - [ ] Verify triggers still work
- [ ] Document wrap checklist
- [ ] Test /wrap command

**Acceptance Criteria:**

- ✓ /wrap command created
- ✓ Skill maintenance workflow defined
- ✓ Command works as expected

---

### 5.3: Update Documentation [Effort: L]

- [ ] Update README.md with AI features
- [ ] Create `docs/ai-intent-analysis.md`
  - [ ] How AI works
  - [ ] Configuration
  - [ ] Cost analysis
  - [ ] Fallback behavior
- [ ] Create `docs/affinity-injection.md`
  - [ ] What is affinity
  - [ ] How to define
  - [ ] Bidirectional semantics
  - [ ] Best practices
- [ ] Update `CLAUDE_INTEGRATION_GUIDE.md`
  - [ ] AI setup instructions
  - [ ] Environment variables
  - [ ] Troubleshooting
- [ ] Update `.claude/hooks/CONFIG.md`
  - [ ] Document CATALYST_USE_AI
  - [ ] Document CATALYST_DEBUG
- [ ] Add examples to docs
- [ ] Review all documentation

**Acceptance Criteria:**

- ✓ All documentation updated
- ✓ New docs created
- ✓ Examples included

---

### 5.4: Environment Setup Script [Effort: S]

- [ ] Create `setup-ai.sh` script
- [ ] Prompt for ANTHROPIC_API_KEY
- [ ] Save API key to ~/.bashrc or ~/.zshrc
- [ ] Build intent-analyzer binary
- [ ] Install to ~/.claude-hooks/bin/
- [ ] Enable CATALYST_USE_AI
- [ ] Test API connection
- [ ] Add error handling
- [ ] Document setup process

**Acceptance Criteria:**

- ✓ Setup script works
- ✓ API key saved correctly
- ✓ Binary installed
- ✓ Connection tested

---

## Phase 6: Analytics API (Optional Future Enhancement) ⏳ NOT STARTED

**Goal:** Add optional analytics endpoints to intent-analyzer API
**Status:** Optional - Not required for MVP
**Estimated Duration:** 1 week (if implemented)

**Note:** Phase 1-5 use direct SQLite for performance. Phase 6 adds read-only analytics endpoints for web dashboard and CLI stats.

### 6.1: Design Analytics Schema [Effort: S]

- [ ] Document read-only access pattern
- [ ] Define analytics queries
  - [ ] Most-used skills (last 7/30 days)
  - [ ] Session activity over time
  - [ ] Average confidence scores by skill
  - [ ] Session detail view
- [ ] Plan indexes for efficient analytics
- [ ] Document query performance targets

**Acceptance Criteria:**

- ✓ Analytics queries documented
- ✓ Read-only access pattern defined
- ✓ Indexes planned

---

### 6.2: Implement Read-Only Database Connection [Effort: S]

- [ ] Create `intent-analyzer/src/db.rs`
- [ ] Implement `AnalyticsDb` struct
- [ ] Open database in read-only mode
- [ ] Enable query optimization (PRAGMA query_only)
- [ ] Implement connection pooling
- [ ] Add error handling for locked database
- [ ] Write unit tests
  - [ ] Test read-only mode
  - [ ] Test query optimization
  - [ ] Test error handling

**Acceptance Criteria:**

- ✓ Database opens read-only
- ✓ PRAGMA query_only enabled
- ✓ Error handling works
- ✓ 3 tests passing

---

### 6.3: Implement Analytics Endpoints [Effort: M]

- [ ] Create `intent-analyzer/src/analytics.rs`
- [ ] Define response structs (SkillStats, SessionActivity)
- [ ] Implement GET /api/stats/skills endpoint
- [ ] Implement GET /api/stats/sessions endpoint
- [ ] Implement GET /api/stats/confidence endpoint
- [ ] Implement GET /api/sessions/:id endpoint
- [ ] Add query parameters (days filter)
- [ ] Implement proper error responses
- [ ] Write integration tests
  - [ ] Test each endpoint
  - [ ] Test query parameters
  - [ ] Test error cases (not found, invalid params)
- [ ] Add request logging

**Acceptance Criteria:**

- ✓ 4 analytics endpoints implemented
- ✓ JSON responses with proper structure
- ✓ Error handling works
- ✓ 4 integration tests passing

---

### 6.4: Add Analytics CLI Commands [Effort: S]

- [ ] Create `catalyst-cli/src/stats.rs`
- [ ] Implement `catalyst stats skills` command
- [ ] Implement `catalyst stats sessions` command
- [ ] Implement `catalyst stats export` command
- [ ] Add table/chart formatting
- [ ] Support export formats (JSON, CSV)
- [ ] Add --days parameter
- [ ] Write tests
  - [ ] Test each command
  - [ ] Test output formatting
  - [ ] Test export formats
- [ ] Document commands

**Acceptance Criteria:**

- ✓ 3 stats commands work
- ✓ Pretty table output
- ✓ Export functionality works
- ✓ 3 tests passing

---

### 6.5: Web Dashboard (Optional) [Effort: XL]

- [ ] Create `intent-analyzer/src/dashboard.rs`
- [ ] Create static HTML/CSS/JS dashboard
- [ ] Implement GET /dashboard endpoint
- [ ] Implement static file serving
- [ ] Add Chart.js visualizations
  - [ ] Most-used skills bar chart
  - [ ] Usage trends line chart
  - [ ] Confidence scores scatter plot
- [ ] Add session search functionality
- [ ] Add export buttons
- [ ] Make responsive (mobile-friendly)
- [ ] Test in multiple browsers
- [ ] Document dashboard features

**Acceptance Criteria:**

- ✓ Dashboard accessible at <http://localhost:3030/dashboard>
- ✓ Charts display correctly
- ✓ Search works
- ✓ Export works
- ✓ Mobile-friendly

---

## Summary

### Phase Completion Status

- **Phase 0 (Research):** ⏳ NOT STARTED (0/7 tasks) **← Must complete first**
- **Phase 1:** ⏳ NOT STARTED (0/5 tasks)
- **Phase 2:** ⏳ NOT STARTED (0/5 tasks)
- **Phase 3:** ⏳ NOT STARTED (0/3 tasks)
- **Phase 4:** ⏳ NOT STARTED (0/8 tasks)
- **Phase 5:** ⏳ NOT STARTED (0/4 tasks)
- **Phase 6 (Optional):** ⏳ NOT STARTED (0/5 tasks)

### Overall Progress

**0/37 major tasks completed (0%)** (32 required + 5 optional)

### Test Counts

- **Phase 1:** ~40 tests
- **Phase 2:** ~14 tests
- **Phase 3:** ~29 tests
- **Phase 4:** ~120 tests (comprehensive)
- **Phase 5:** ~5 tests
- **Phase 6 (Optional):** ~10 tests
- **Total:** ~218 tests (208 required + 10 optional)

### Next Immediate Steps

1. ✅ Review plan and get approval
2. ⏳ Set up development environment
3. ⏳ Start Phase 1.1: Implement Cache Manager

---

**End of Tasks**
**Last Updated:** 2025-11-11

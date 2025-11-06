# Amazon Q Developer CLI Compatibility Research

**Last Updated:** 2025-01-05
**Status:** Research/Feasibility Study
**Conclusion:** HIGHLY FEASIBLE with architectural differences

---

## Executive Summary

### Feasibility Verdict

**✅ HIGHLY FEASIBLE** - Catalyst's skill activation system can work with Amazon Q Developer CLI, but with a different architecture than Claude Code.

### Key Findings

1. **Auto-Activation: SOLVED** ✅
   - Amazon Q's `chat.defaultAgent` setting enables automatic hook execution
   - `userPromptSubmit` hook is nearly identical to Claude Code's `UserPromptSubmit`
   - Catalyst's Rust binaries work with minimal adaptation

2. **Progressive Disclosure: DIFFERENT APPROACH** ⚠️
   - Q doesn't load files progressively like Claude Code
   - Knowledge Bases provide on-demand content via semantic search
   - May actually be MORE efficient than progressive disclosure

3. **Implementation Effort: 2-3 Weeks** 📅
   - Week 1: Adapt hooks and create catalyst-agent
   - Week 2: Knowledge Base integration
   - Week 3: Installation tooling and documentation

### Quick Comparison

| Feature | Claude Code | Amazon Q | Status |
|---------|-------------|----------|--------|
| **Auto-activation** | UserPromptSubmit hook | userPromptSubmit hook + default agent | ✅ Compatible |
| **File tracking** | PostToolUse hook | postToolUse hook | ✅ Compatible |
| **Configuration** | .claude/settings.json (per-project) | Default agent (global) | ⚠️ Different scope |
| **Skill content** | SKILL.md files loaded progressively | Knowledge Bases (semantic search) | ⚠️ Different mechanism |
| **Hook input** | JSON via STDIN | JSON via STDIN | ✅ Compatible |
| **Hook output** | Text to STDOUT | Text to STDOUT | ✅ Compatible |
| **Binary location** | ~/.claude-hooks/bin/ | Can reuse same location | ✅ Compatible |

---

## 1. Amazon Q Hook System

### Available Hooks

Amazon Q provides 5 hook trigger points:

| Hook Event | Trigger Point | Input | Output Scope | Can Block? |
|------------|---------------|-------|--------------|------------|
| **agentSpawn** | Agent initialization | Basic event data | Entire session | No |
| **userPromptSubmit** | User submits input | `prompt`, `cwd` | Current prompt only | No |
| **preToolUse** | Before tool execution | `tool_name`, `tool_input` | Tool execution | Yes (exit 2) |
| **postToolUse** | After tool execution | `tool_response` | Current prompt | No |
| **stop** | Assistant finishes | Event data | Current prompt | No |

### Configuration Format

Agent hooks are defined in JSON files:

```json
{
  "name": "my-agent",
  "description": "Agent description",
  "hooks": {
    "userPromptSubmit": [
      {
        "command": "bash /path/to/script.sh",
        "timeout_ms": 30000,
        "cache_ttl_seconds": 30,
        "max_output_size": 10240
      }
    ],
    "postToolUse": [
      {
        "matcher": "fs_write",
        "command": "validate_write.sh"
      }
    ]
  }
}
```

### Input/Output Model

**Input (STDIN):**
```json
{
  "hook_event_name": "userPromptSubmit",
  "cwd": "/path/to/project",
  "prompt": "user's message here"
}
```

**Output (STDOUT):**
Plain text that becomes context for the current interaction.

**Exit Codes:**
- `0` - Success
- `2` - Block execution (preToolUse only)
- Other - Warning/error (execution continues)

### Comparison with Claude Code

| Aspect | Claude Code | Amazon Q |
|--------|-------------|----------|
| **Event naming** | PascalCase (`UserPromptSubmit`) | camelCase (`userPromptSubmit`) |
| **Input fields** | `session_id`, `transcript_path`, `permission_mode`, `prompt`, `cwd` | `hook_event_name`, `prompt`, `cwd` |
| **Configuration** | Centralized `.claude/settings.json` | Per-agent JSON files |
| **Timeout** | Not specified | 30 seconds default, configurable |
| **Caching** | Not available | `cache_ttl_seconds` supported |
| **Output limits** | Unlimited | 10KB default |

**Key Insight:** The core model is identical - JSON in, text out, same execution points. Only minor syntax differences.

---

## 2. Auto-Activation Solution

### The Problem

How do you make skills auto-activate in Amazon Q when Q requires explicit agent selection?

### The Solution: Default Agent

Amazon Q supports setting a default agent that automatically activates for all chat sessions:

```bash
q settings chat.defaultAgent catalyst
```

Once set, every `q chat` command automatically uses the catalyst agent and runs its hooks.

### Architecture

```
User runs: q chat
    ↓
catalyst agent spawns (automatically, no user action)
    ↓
agentSpawn hook: Initialize session context
    ↓
User types: "How do I create an Express route?"
    ↓
userPromptSubmit hook: Analyze prompt
    ↓
Hook checks skill-rules.json
    ↓
Hook outputs: "💡 Detected Express/backend question - consider @backend-dev-guidelines"
    ↓
User sees suggestion in context
    ↓
User activates: @backend-dev-guidelines
    ↓
Specialized agent provides guidance
    ↓
postToolUse hook: Track file changes (if any)
```

### How It Works

1. **One-time setup:** User runs `q settings chat.defaultAgent catalyst`
2. **Automatic activation:** Every Q session starts with catalyst agent
3. **Hook execution:** catalyst agent's hooks run automatically
4. **Skill suggestion:** Hooks analyze prompts and suggest relevant skills
5. **Natural workflow:** User follows suggestions to activate specialized agents

### Agent Selection Priority

Amazon Q uses a 3-tier priority system:

1. **Command-line specified:** `q chat --agent my-agent` (highest priority)
2. **User-configured default:** `q settings chat.defaultAgent catalyst`
3. **Built-in default:** Fallback agent with all tools

This means:
- ✅ catalyst runs by default for all sessions
- ✅ Users can override with `--agent` flag when needed
- ✅ Provides consistent auto-activation experience

### Comparison with Claude Code

**Claude Code:**
```json
// .claude/settings.json (per-project)
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude-hooks/bin/skill-activation-prompt"
          }
        ]
      }
    ]
  }
}
```

**Amazon Q:**
```bash
# Global setting (applies to all projects)
q settings chat.defaultAgent catalyst

# ~/.aws/amazonq/cli-agents/catalyst.json
{
  "name": "catalyst",
  "hooks": {
    "userPromptSubmit": [
      {
        "command": "~/.claude-hooks/bin/skill-activation-prompt"
      }
    ]
  }
}
```

**Key Difference:**
- Claude Code: Per-project configuration (`.claude/settings.json`)
- Amazon Q: Global default agent (applies to all projects)

**Workaround for per-project behavior:**
Hook scripts can check for `.catalyst/config.json` in current directory and behave accordingly.

---

## 3. Progressive Disclosure Challenge

### The Problem Explained

Catalyst's skills are designed with progressive disclosure to minimize token usage:

**Example: backend-dev-guidelines**
```
backend-dev-guidelines/
├── SKILL.md                    (~500 lines)
│   Overview + navigation
└── resources/
    ├── error-handling.md       (~600 lines)
    ├── database-patterns.md    (~550 lines)
    ├── testing-strategies.md   (~500 lines)
    └── api-design.md          (~450 lines)
```

**Total:** ~2,600 lines

**Claude Code approach:**
1. Load SKILL.md initially (500 lines)
2. User asks about error handling
3. Load resources/error-handling.md (600 lines)
4. **Total context:** 1,100 lines (42% of full skill)

**Goal:** Load only what's needed, when needed.

### Amazon Q's Three Context Mechanisms

#### A. Agent Resources (Static File Loading)

**Configuration:**
```json
{
  "resources": [
    "file://.claude/skills/backend-dev-guidelines/SKILL.md",
    "file://.claude/skills/*/resources/*.md"
  ]
}
```

**Characteristics:**
- ✅ Supports markdown files
- ✅ Supports glob patterns
- ✅ Can load from .claude/skills/
- ❌ **ALL files load at agent startup**
- ❌ Consumes tokens whether used or not
- ❌ Up to 75% of context window consumed by resources

**Verdict:** ❌ **Does NOT achieve progressive disclosure**

**Token consumption:** All 2,600 lines loaded upfront, every session.

---

#### B. Agent Hooks (Dynamic Command Output)

**Configuration:**
```json
{
  "hooks": {
    "userPromptSubmit": [
      {
        "command": "~/.claude-hooks/bin/skill-activation-prompt",
        "max_output_size": 10240
      }
    ]
  }
}
```

**Characteristics:**
- ✅ Can run shell commands
- ✅ Output becomes context
- ✅ Supports caching
- ❌ **Output is plain text only** (max 10KB)
- ❌ **Cannot reference files to be loaded**
- ❌ No special syntax for dynamic file loading

**Example of what DOESN'T work:**
```bash
#!/bin/bash
# This hook outputs text, but Q won't load the file
echo "LOAD_FILE:.claude/skills/backend-dev-guidelines/SKILL.md"
```

Q treats this as literal text, not a file-loading instruction.

**Verdict:** ❌ **Cannot achieve progressive disclosure**

**Token consumption:** Hook output (up to 10KB of text) but cannot dynamically load skill files.

---

#### C. Knowledge Bases (On-Demand Search)

**Setup:**
```bash
q chat
> /knowledge add --name backend --path .claude/skills/backend-dev-guidelines/
```

**Characteristics:**
- ✅ **Indexes markdown files from path**
- ✅ **On-demand only** - doesn't consume tokens until searched
- ✅ **Semantic + lexical search**
- ✅ **Only relevant chunks consume tokens**
- ✅ Agent-isolated (separate knowledge per agent)
- ⚠️ **Experimental/Beta feature**
- ❌ **Requires manual setup** (user must run /knowledge add)
- ❌ **No hook integration** (can't programmatically trigger)
- ❌ **No automatic activation** based on prompt analysis

**How it works:**
1. User runs `/knowledge add` to index skill directories
2. Q indexes all markdown files
3. When user asks a question, Q searches knowledge base
4. Only relevant chunks are loaded and consume tokens

**Example:**
```
User: "How do I handle Express errors?"
    ↓
Q searches 'backend' knowledge base
    ↓
Finds relevant chunks from:
  - SKILL.md (section on error handling)
  - resources/error-handling.md (Express-specific content)
    ↓
Loads ~200-400 lines of relevant content
    ↓
Much more efficient than loading all 2,600 lines!
```

**Verdict:** ⚠️ **PARTIAL SUCCESS - Different but potentially better**

**Token consumption:** Only relevant chunks (~200-400 lines for specific questions) vs full files (1,100+ lines).

---

### Progressive Disclosure Comparison

| Approach | Initial Load | Follow-up Load | Total | Efficiency |
|----------|--------------|----------------|-------|------------|
| **Claude Code (progressive)** | SKILL.md (500 lines) | error-handling.md (600 lines) | 1,100 lines | Baseline |
| **Q Resources (static)** | All files (2,600 lines) | N/A | 2,600 lines | ❌ 2.4x worse |
| **Q Hooks (output)** | Hook text (~100 lines) | Cannot load files | 100 lines | ❌ Incomplete |
| **Q Knowledge Bases (search)** | Nothing | Relevant chunks (~300 lines) | 300 lines | ✅ 3.7x better! |

**Surprising finding:** Amazon Q's Knowledge Bases may actually be MORE efficient than progressive disclosure because they load only relevant chunks via semantic search, not entire files.

---

### Why None Perfectly Replicate Claude Code

**Claude Code's strength:**
- Explicit control over what loads when
- Full file content (including structure/navigation)
- Progressive expansion of context

**Amazon Q's strength:**
- Semantic understanding finds exact relevant content
- No need to load full files
- More efficient token usage for targeted questions

**Amazon Q's limitation:**
- Can't replicate "load full SKILL.md then navigate to resources" workflow
- Search-based (finds content) vs instruction-based (loads structure)
- Requires manual knowledge base setup

---

## 4. Recommended Architecture

### Catalyst-Agent as Default Agent

**Core concept:** Create a single "catalyst-agent" that acts as the auto-activation layer for all skills.

**Agent definition (`~/.aws/amazonq/cli-agents/catalyst.json`):**

```json
{
  "name": "catalyst",
  "description": "Auto-activation system for Catalyst skills",
  "prompt": "You are an intelligent agent that helps developers by suggesting relevant skills and knowledge. When a user asks a question, analyze the intent and suggest appropriate specialized agents or knowledge bases that can help.",

  "hooks": {
    "agentSpawn": [
      {
        "command": "~/.claude-hooks/bin/catalyst-agent-init",
        "timeout_ms": 5000
      }
    ],
    "userPromptSubmit": [
      {
        "command": "~/.claude-hooks/bin/skill-activation-prompt",
        "cache_ttl_seconds": 0,
        "timeout_ms": 10000
      }
    ],
    "postToolUse": [
      {
        "command": "~/.claude-hooks/bin/post-tool-use-tracker",
        "timeout_ms": 5000
      }
    ]
  },

  "resources": [],

  "tools": ["*"]
}
```

**Hook adaptations required:**

1. **skill-activation-prompt** - Minimal changes:
   ```rust
   // Add Amazon Q input schema support
   #[derive(Debug, Deserialize)]
   #[serde(untagged)]
   enum HookInput {
       ClaudeCode {
           session_id: String,
           transcript_path: String,
           cwd: String,
           permission_mode: String,
           prompt: String,
       },
       AmazonQ {
           hook_event_name: String,
           cwd: String,
           prompt: String,
       },
   }
   ```

2. **post-tool-use-tracker** - Similar adaptation for Q's JSON format

3. **catalyst-agent-init** (new) - Initialize session:
   - Check for `.catalyst/config.json` in current directory
   - Load project-specific skill-rules.json
   - Output welcome message

### Skills as Knowledge Bases

Instead of loading skills as agents, index them as knowledge bases:

**Installation script approach:**

```bash
#!/bin/bash
# install-catalyst-q.sh

# Create catalyst agent
mkdir -p ~/.aws/amazonq/cli-agents/catalyst
cp catalyst-agent.json ~/.aws/amazonq/cli-agents/catalyst/

# Set as default
q settings chat.defaultAgent catalyst

# Add skills as knowledge bases
q chat << EOF
/knowledge add --name skill-developer --path $PWD/.claude/skills/skill-developer/
/knowledge add --name backend-dev --path $PWD/.claude/skills/backend-dev-guidelines/
/knowledge add --name frontend-dev --path $PWD/.claude/skills/frontend-dev-guidelines/
/knowledge add --name route-tester --path $PWD/.claude/skills/route-tester/
/knowledge add --name error-tracking --path $PWD/.claude/skills/error-tracking/
/knowledge add --name rust-developer --path $PWD/.claude/skills/rust-developer/
EOF

echo "✅ Catalyst installed for Amazon Q!"
```

**Per-project customization:**

```bash
# Optional: Create project-specific config
mkdir .catalyst
cp /path/to/catalyst/.catalyst/skill-rules.json .catalyst/

# Edit pathPatterns for your project structure
vim .catalyst/skill-rules.json
```

### User Experience Flow

**Installation (one-time):**
```bash
cd ~/catalyst-repo
./install-catalyst-q.sh
```

**Daily usage:**
```bash
cd ~/my-project
q chat

User: "I need to create a new Express API endpoint"

# catalyst-agent's userPromptSubmit hook analyzes prompt
# Hook checks skill-rules.json for backend-related keywords
# Hook outputs:
# "💡 Detected backend development question
#  Keywords matched: Express, API, endpoint
#  Recommended: Search 'backend-dev' knowledge base"

User: [Q automatically searches backend-dev knowledge]
# Q finds relevant content about Express routing, API design
# Loads ~300 lines of targeted guidance

User: "How do I validate the request body?"

# Q searches again, finds validation patterns
# Loads ~200 lines about request validation
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────┐
│  User runs: q chat                                   │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  catalyst-agent spawns (default agent)              │
│  ├─ agentSpawn hook: Initialize session             │
│  └─ Load project-specific config (if exists)        │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  User: "How do I create an Express route?"          │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  userPromptSubmit hook runs                         │
│  ├─ Read .catalyst/skill-rules.json                 │
│  ├─ Match keywords: Express, route, API             │
│  ├─ Check pathPatterns: *.ts files in backend/      │
│  └─ Output: "💡 backend-dev knowledge recommended"  │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  Q searches backend-dev knowledge base              │
│  ├─ Semantic search: "Express route creation"       │
│  ├─ Finds relevant chunks from SKILL.md             │
│  └─ Loads ~300 lines of routing guidance            │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  Q provides answer with code examples               │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  User edits files (using Q's tools)                 │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  postToolUse hook runs                              │
│  ├─ Track file changes                              │
│  └─ Update context with modifications               │
└─────────────────────────────────────────────────────┘
```

---

## 5. Implementation Roadmap

### MVP (Week 1)

**Goal:** Basic catalyst-agent with hook-based suggestions

**Tasks:**
1. Adapt Rust hooks for Amazon Q JSON schema (4 hours)
   - Update input structs to handle both Claude Code and Q formats
   - Add platform detection logic
   - Test with sample Q hook input

2. Create catalyst-agent.json definition (2 hours)
   - Define agent with userPromptSubmit and postToolUse hooks
   - Write agent description and instructions
   - Configure hook timeouts and caching

3. Create installation script (3 hours)
   - Build/copy Rust binaries to ~/.claude-hooks/bin/
   - Install catalyst-agent to Q's agents directory
   - Set as default agent
   - Test installation flow

4. Test auto-activation (1 day)
   - Verify hook execution on prompts
   - Test skill suggestion output
   - Validate prompt analysis logic
   - Ensure cross-platform compatibility

**Deliverable:** Working catalyst-agent that suggests skills based on prompts

### Phase 2: Knowledge Base Integration (Week 2)

**Goal:** Integrate skills as searchable knowledge bases

**Tasks:**
1. Knowledge base setup automation (1 day)
   - Script to add all skills as knowledge bases
   - Handle errors and provide feedback
   - Document manual setup process

2. Test knowledge base search (2 days)
   - Verify Q finds relevant content
   - Test with various question types
   - Measure token efficiency
   - Compare with Claude Code's progressive disclosure

3. Refine hook suggestions (1 day)
   - Update hook output to mention knowledge bases
   - Add guidance on when to search which knowledge
   - Test suggestion quality

**Deliverable:** Full skill content accessible via knowledge bases with auto-suggestions

### Phase 3: Installation & Documentation (Week 3)

**Goal:** Production-ready installation and user documentation

**Tasks:**
1. Installation tooling (2 days)
   - Cross-platform install script (Linux, macOS, Windows)
   - Error handling and validation
   - Uninstall script
   - Update script

2. User documentation (2 days)
   - Installation guide
   - Usage examples
   - Troubleshooting section
   - Comparison with Claude Code setup

3. Per-project customization (1 day)
   - Document .catalyst/config.json format
   - Provide example configurations
   - Explain pathPattern customization

**Deliverable:** Complete Amazon Q support with installation and docs

### Full Timeline

**Total: 2-3 weeks**
- Week 1: Core functionality (MVP)
- Week 2: Knowledge integration
- Week 3: Polish and documentation

---

## 6. Technical Requirements

### Rust Hook Adaptations

**Changes needed in `skill-activation-prompt` binary:**

```rust
// src/bin/skill_activation_prompt.rs

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum HookInput {
    ClaudeCode {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: String,
        prompt: String,
    },
    AmazonQ {
        hook_event_name: String,
        cwd: String,
        prompt: String,
    },
}

impl HookInput {
    fn get_prompt(&self) -> &str {
        match self {
            HookInput::ClaudeCode { prompt, .. } => prompt,
            HookInput::AmazonQ { prompt, .. } => prompt,
        }
    }

    fn get_cwd(&self) -> &str {
        match self {
            HookInput::ClaudeCode { cwd, .. } => cwd,
            HookInput::AmazonQ { cwd, .. } => cwd,
        }
    }
}

fn main() -> Result<()> {
    // Read from stdin
    let input: HookInput = serde_json::from_reader(io::stdin())?;

    // Platform detection (optional)
    let platform = match &input {
        HookInput::ClaudeCode { .. } => "claude-code",
        HookInput::AmazonQ { .. } => "amazon-q",
    };

    // Rest of logic remains the same
    let prompt = input.get_prompt();
    let cwd = input.get_cwd();

    // Analyze prompt, check skill-rules.json, output suggestions
    // ...
}
```

**Effort:** 2-3 hours per binary (3 binaries total)

### Agent JSON Configuration

**Template: `catalyst-agent.json`**

```json
{
  "name": "catalyst",
  "version": "0.1.0",
  "description": "Intelligent skill activation system for Claude Code workflows in Amazon Q",

  "prompt": "You are the Catalyst agent, designed to help developers by suggesting relevant skills and knowledge bases. When users ask questions:\n\n1. Analyze the prompt for technical keywords and intent\n2. Check which Catalyst skills might be relevant (backend, frontend, testing, etc.)\n3. Suggest searching appropriate knowledge bases\n4. Provide helpful context based on hook output\n\nYou have access to knowledge bases for:\n- skill-developer: Creating custom skills\n- backend-dev: Node.js/Express/Prisma patterns\n- frontend-dev: React/MUI/TanStack patterns  \n- route-tester: Authentication testing\n- error-tracking: Sentry integration\n- rust-developer: Rust best practices\n\nWhen hooks suggest specific skills, search the corresponding knowledge base.",

  "hooks": {
    "agentSpawn": [
      {
        "command": "~/.claude-hooks/bin/catalyst-agent-init",
        "timeout_ms": 5000,
        "cache_ttl_seconds": 3600
      }
    ],
    "userPromptSubmit": [
      {
        "command": "~/.claude-hooks/bin/skill-activation-prompt",
        "timeout_ms": 10000,
        "cache_ttl_seconds": 0,
        "max_output_size": 10240
      }
    ],
    "postToolUse": [
      {
        "command": "~/.claude-hooks/bin/post-tool-use-tracker",
        "timeout_ms": 5000,
        "cache_ttl_seconds": 30
      }
    ]
  },

  "resources": [],

  "tools": ["*"]
}
```

### Knowledge Base Setup

**Commands for each skill:**

```bash
# Run in Q chat session
/knowledge add --name skill-developer --path ~/.claude/skills/skill-developer/
/knowledge add --name backend-dev --path ~/.claude/skills/backend-dev-guidelines/
/knowledge add --name frontend-dev --path ~/.claude/skills/frontend-dev-guidelines/
/knowledge add --name route-tester --path ~/.claude/skills/route-tester/
/knowledge add --name error-tracking --path ~/.claude/skills/error-tracking/
/knowledge add --name rust-developer --path ~/.claude/skills/rust-developer/
```

**Installation script approach:**

```bash
#!/bin/bash
# install-knowledge-bases.sh

SKILLS_DIR="${HOME}/.claude/skills"

if [ ! -d "$SKILLS_DIR" ]; then
    echo "Error: Catalyst skills not found at $SKILLS_DIR"
    exit 1
fi

echo "Adding Catalyst skills as knowledge bases..."

q chat << 'EOF'
/knowledge add --name skill-developer --path ~/.claude/skills/skill-developer/
/knowledge add --name backend-dev --path ~/.claude/skills/backend-dev-guidelines/
/knowledge add --name frontend-dev --path ~/.claude/skills/frontend-dev-guidelines/
/knowledge add --name route-tester --path ~/.claude/skills/route-tester/
/knowledge add --name error-tracking --path ~/.claude/skills/error-tracking/
/knowledge add --name rust-developer --path ~/.claude/skills/rust-developer/
/exit
EOF

echo "✅ Knowledge bases installed!"
```

### Cross-Platform Considerations

**Hook script wrappers (if needed):**

**Linux/macOS:** Direct binary execution
```json
{
  "command": "~/.claude-hooks/bin/skill-activation-prompt"
}
```

**Windows:** May need wrapper
```json
{
  "command": "powershell -File ~/.claude-hooks/bin/skill-activation-prompt.ps1"
}
```

**Binary compatibility:**
- Rust compiles to native executables for each platform
- Same binaries work for both Claude Code and Amazon Q
- No additional dependencies required

---

## 7. Limitations & Trade-offs

### What Works

| Feature | Status | Notes |
|---------|--------|-------|
| **Auto-activation** | ✅ Full support | Via default agent + userPromptSubmit hook |
| **Skill suggestions** | ✅ Full support | Hook analyzes prompts, outputs recommendations |
| **File tracking** | ✅ Full support | postToolUse hook works identically |
| **Cross-platform** | ✅ Full support | Rust binaries work on Linux/macOS/Windows |
| **Binary reuse** | ✅ Full support | Same binaries for Claude Code and Q |
| **Hook caching** | ✅ Better than Claude | Q supports cache_ttl_seconds |
| **Token efficiency** | ✅ Better than Claude | Knowledge Bases load only relevant chunks |

### What Doesn't Work

| Feature | Status | Workaround |
|---------|--------|------------|
| **Per-project config** | ⚠️ Limited | Hooks check for .catalyst/config.json in cwd |
| **Progressive disclosure** | ❌ Different approach | Use Knowledge Bases (search-based) instead |
| **Dynamic file loading** | ❌ Not supported | Q doesn't support hook-triggered file loading |
| **Skill tool invocation** | ❌ No equivalent | Knowledge Bases provide alternative |
| **Auto knowledge setup** | ⚠️ Manual step | User must run /knowledge add commands |

### Trade-offs Analysis

#### Global vs Per-Project Configuration

**Claude Code:**
- Each project has `.claude/settings.json`
- Hooks configured per-project
- Easy to customize for specific projects

**Amazon Q:**
- Default agent is global
- Applies to all Q sessions
- Per-project behavior via hook scripts checking `.catalyst/config.json`

**Impact:** Medium - works well but requires hook logic to check current directory

**Mitigation:**
```bash
# Hook script checks for project config
if [ -f "$PWD/.catalyst/config.json" ]; then
    # Use project-specific rules
    cat | skill-activation-prompt --config "$PWD/.catalyst/config.json"
else
    # Use default behavior or skip
    echo ""
fi
```

#### Manual Knowledge Base Setup

**Claude Code:**
- Skills install automatically via `catalyst init`
- SKILL.md files in place immediately

**Amazon Q:**
- User must run `/knowledge add` for each skill
- One-time setup but manual step

**Impact:** Low - can be automated with script, but requires user action once

**Mitigation:** Installation script runs /knowledge add commands automatically

#### Progressive Disclosure → Semantic Search

**Claude Code:**
- Explicit file loading (SKILL.md, then resources/*.md)
- Full file content always loaded
- ~1,100 lines for typical question

**Amazon Q:**
- Semantic search finds relevant chunks
- Only pertinent content loaded
- ~300 lines for typical question

**Impact:** Neutral to Positive - different approach but potentially more efficient

**Not a true limitation** - Amazon Q's approach may actually be better for token efficiency

#### Experimental Knowledge Bases Feature

**Risk:** Knowledge Bases are marked as "experimental" in Q documentation

**Implications:**
- API may change
- Feature may be removed or significantly modified
- Stability not guaranteed

**Impact:** Medium - core functionality depends on experimental feature

**Mitigation:**
- Document fallback approaches (inline resources)
- Monitor Q changelog for Knowledge Base updates
- Be prepared to adapt if feature changes

---

## 8. Testing Checklist

Before implementing full Amazon Q support, verify these capabilities:

### Phase 1: Amazon Q Installation & Setup

- [ ] Install Amazon Q Developer CLI
- [ ] Verify `q` command works
- [ ] Check version: `q --version`
- [ ] Ensure AWS account configured (if required)

### Phase 2: Agent Creation & Configuration

- [ ] Create test agent: `q chat` → `/agent create --name test`
- [ ] Verify agent JSON created in `~/.aws/amazonq/cli-agents/test/`
- [ ] Edit test agent with simple hook
- [ ] Test agent activation: `q chat --agent test`

### Phase 3: Hook Execution

**Test userPromptSubmit hook:**

```json
{
  "name": "test-agent",
  "hooks": {
    "userPromptSubmit": [
      {
        "command": "echo '🎯 Hook executed! Prompt received.'"
      }
    ]
  }
}
```

- [ ] Hook output appears in Q response
- [ ] Hook executes on every prompt
- [ ] Timeout behavior works (set low timeout, use slow command)
- [ ] Caching works (set cache_ttl_seconds, verify repeat prompts use cache)

**Test with Rust binary:**

- [ ] Build skill-activation-prompt binary
- [ ] Configure hook to call binary
- [ ] Pipe test JSON to binary: `echo '{"hook_event_name":"userPromptSubmit","prompt":"test","cwd":"/tmp"}' | ./skill-activation-prompt`
- [ ] Verify binary output appears in Q

### Phase 4: Default Agent

- [ ] Set default agent: `q settings chat.defaultAgent test`
- [ ] Verify setting persists: `q settings chat.defaultAgent`
- [ ] Run `q chat` without --agent flag
- [ ] Confirm test agent activates automatically
- [ ] Verify hooks run without explicit agent selection

### Phase 5: Knowledge Bases

- [ ] Create test knowledge directory with markdown files
- [ ] Add knowledge base: `/knowledge add --name test-kb --path /path/to/test/`
- [ ] List knowledge bases: `/knowledge list`
- [ ] Ask question that should match knowledge content
- [ ] Verify relevant content is found and used in response
- [ ] Check token usage (should be minimal)

### Phase 6: Integration Test

**Complete workflow:**

- [ ] Install catalyst-agent with hooks
- [ ] Set as default agent
- [ ] Add skill-developer as knowledge base
- [ ] Run Q chat
- [ ] Type: "How do I create a new skill?"
- [ ] Verify hook analyzes prompt
- [ ] Verify knowledge base is searched
- [ ] Verify relevant guidance appears

### Critical Test Cases

**Test 1: Auto-activation**
```bash
q chat
> "I need to validate Express request bodies"
# Expected: Hook detects backend/Express keywords
# Expected: Suggests backend-dev knowledge
# Expected: Q searches backend knowledge, finds validation patterns
```

**Test 2: Per-project config**
```bash
cd project-with-catalyst-config
q chat
> "How do I structure my routes?"
# Expected: Hook reads .catalyst/config.json
# Expected: Uses project-specific pathPatterns
# Expected: Tailored suggestions based on project structure
```

**Test 3: File tracking**
```bash
q chat
> "Create a new user route in routes/users.ts"
# Q uses tools to create file
# Expected: postToolUse hook detects file creation
# Expected: Hook outputs file change context
```

**Test 4: Knowledge base efficiency**
```bash
q chat
> "What's the error handling pattern?"
# Expected: Q loads only error-handling chunks (~300 lines)
# NOT: Entire backend-dev-guidelines skill (~2,600 lines)
```

### Performance Benchmarks

**Measure and compare:**

| Metric | Claude Code | Amazon Q | Target |
|--------|-------------|----------|--------|
| Hook execution time | < 100ms | ? | < 200ms |
| Initial load tokens | 500 (SKILL.md) | ? | < 400 (relevant chunks) |
| Follow-up load tokens | 600 (resource file) | ? | < 300 (relevant chunks) |
| Total tokens (typical question) | 1,100 | ? | < 700 |

---

## 9. Open Questions

### 1. Knowledge Base Stability

**Question:** How stable are Knowledge Bases given their "experimental" status?

**Research needed:**
- Check Q changelog for Knowledge Base updates
- Monitor Q community for deprecation warnings
- Test Knowledge Base behavior across Q versions

**Risk level:** Medium
**Mitigation:** Document fallback approach (inline resources)

### 2. Knowledge Base Performance

**Question:** How well do Knowledge Bases scale with large skill repositories?

**Research needed:**
- Test with all 6 Catalyst skills (~15,000 total lines)
- Measure search latency
- Test with skill repositories > 50MB

**Risk level:** Low
**Impact:** May need to optimize skill content or split into smaller knowledge bases

### 3. Multi-Project Workflows

**Question:** How does global default agent work when switching between projects?

**Scenarios:**
- Project A: Uses backend-dev-guidelines (TypeScript)
- Project B: Uses rust-developer (Rust)
- Project C: No Catalyst configuration

**Research needed:**
- Test rapid project switching
- Verify hook correctly detects project context
- Ensure graceful behavior in non-Catalyst projects

**Risk level:** Low
**Mitigation:** Hooks check for `.catalyst/config.json`, behave appropriately if missing

### 4. Agent Switching UX

**Question:** If user switches from catalyst to another agent, do they lose auto-activation?

**Scenario:**
```bash
q chat --agent my-custom-agent
# Does skill auto-activation still work?
```

**Expected:** No - hooks are per-agent, not global

**Research needed:**
- Test agent switching behavior
- Determine if multiple agents can be active
- Explore "agent inheritance" or shared hooks

**Risk level:** Low
**Impact:** Users explicitly switching agents likely don't want catalyst auto-activation

### 5. Hook Output Formatting

**Question:** Does Q have special formatting for hook output to make suggestions more visible?

**Current approach:** Plain text output mixed with conversation

**Potential enhancement:**
- Syntax highlighting for hook suggestions
- Dedicated UI section for skill recommendations
- Dismissible suggestion cards

**Research needed:**
- Review Q documentation for hook output formatting
- Test with ANSI color codes
- Test with markdown formatting

**Risk level:** Very Low
**Impact:** UX improvement, not functional requirement

### 6. Windows Compatibility

**Question:** Do Rust binaries work directly in Q hooks on Windows?

**Potential issues:**
- Path separators (\ vs /)
- PowerShell vs CMD execution
- Binary permissions

**Research needed:**
- Test hooks on Windows 10/11
- Verify binary execution (no .exe confusion)
- Test pathPattern matching with Windows paths

**Risk level:** Low
**Mitigation:** Catalyst already supports Windows for Claude Code

---

## 10. Decision Framework

### When to Use Claude Code vs Amazon Q

**Choose Claude Code when:**
- ✅ Per-project configuration is critical
- ✅ You need explicit control over skill loading
- ✅ Progressive disclosure of full file content is important
- ✅ You prefer the "Skill tool" invocation model

**Choose Amazon Q when:**
- ✅ Semantic search is more valuable than structured loading
- ✅ Token efficiency is critical (Knowledge Bases load less)
- ✅ Global configuration across projects is acceptable
- ✅ You prefer Q's agent model over Claude's assistant model

**Use both when:**
- ✅ You want consistency across AI coding assistants
- ✅ Team members use different tools
- ✅ You value Catalyst's skill content regardless of platform

### Migration Considerations

**From Claude Code to Amazon Q:**

1. **Skills:** Already compatible (markdown format works in Knowledge Bases)
2. **Hooks:** Need Rust binary adaptation (minor JSON schema changes)
3. **Configuration:** Need to set up catalyst-agent and Knowledge Bases
4. **Workflow:** Adjust to global agent vs per-project settings

**Effort:** 1-2 days setup, minimal learning curve

**From Amazon Q to Claude Code:**

1. **Skills:** Already compatible
2. **Hooks:** Same Rust binaries work
3. **Configuration:** Create `.claude/settings.json` per project
4. **Workflow:** Adjust to explicit skill loading vs search

**Effort:** 1-2 hours setup per project

### Multi-Platform Support Strategy

**Option 1: Platform-Specific Branches**
- Maintain separate catalyst-claude and catalyst-q repositories
- Optimize each for its platform
- Higher maintenance overhead

**Option 2: Unified Codebase (RECOMMENDED)**
- Single Catalyst repository
- Rust hooks detect platform via input schema
- Installation scripts for both Claude Code and Q
- Skills work on both platforms unchanged

**Advantages:**
- ✅ Single source of truth for skills
- ✅ Bug fixes apply to both platforms
- ✅ Skill updates don't need platform-specific changes
- ✅ Users can switch platforms easily

**Implementation:**
```
catalyst/
├── .claude/
│   ├── skills/              # Platform-agnostic
│   │   ├── skill-developer/
│   │   ├── backend-dev-guidelines/
│   │   └── ...
│   └── hooks/
│       └── RustHooks/       # Detect platform from input
├── install-claude-code.sh   # Claude Code installation
├── install-amazon-q.sh      # Amazon Q installation
└── catalyst-agent.json      # Amazon Q agent template
```

---

## 11. References

### Amazon Q Developer CLI Documentation

**Core Documentation:**
- [Hooks Documentation](https://github.com/aws/amazon-q-developer-cli/blob/main/docs/hooks.md)
- [Agent Format - Hooks Field](https://github.com/aws/amazon-q-developer-cli/blob/main/docs/agent-format.md#hooks-field)
- [Agent Format - Tools Field](https://github.com/aws/amazon-q-developer-cli/blob/main/docs/agent-format.md#tools-field)
- [Command Line Context](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-context.html)
- [Default Agent Behavior](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-agents-default-behavior.html)
- [Custom Agents - Defining](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-custom-agents-defining.html)

**Related Resources:**
- [Amazon Q Developer CLI GitHub](https://github.com/aws/amazon-q-developer-cli)
- [Knowledge Bases (Experimental)](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-knowledge.html)

### Catalyst Documentation

**Internal References:**
- `catalyst-cli-plan.md` - Overall CLI roadmap
- `catalyst-cli-tasks.md` - Task breakdown and status
- `CLAUDE.md` - Integration guide for Claude Code
- `docs/rust-hooks.md` - Rust hook implementation details
- `.claude/skills/*/SKILL.md` - Skill content structure

### Comparison Resources

**Claude Code:**
- [Hooks Documentation](https://docs.claude.com/en/docs/claude-code/hooks)
- [Skills System](https://docs.claude.com/en/docs/claude-code/skills)
- [Settings Configuration](https://docs.claude.com/en/docs/claude-code/settings)

---

## Appendix A: Sample Hook Outputs

### skill-activation-prompt (Amazon Q format)

**Input:**
```json
{
  "hook_event_name": "userPromptSubmit",
  "cwd": "/home/user/my-project",
  "prompt": "How do I validate Express request bodies with Zod?"
}
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 SKILL ACTIVATION CHECK
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📚 DETECTED PATTERNS:

Keywords matched:
  • validate (backend-dev-guidelines)
  • Express (backend-dev-guidelines)
  • Zod (backend-dev-guidelines)

Intent patterns matched:
  • validation-setup (backend-dev-guidelines)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💡 RECOMMENDED ACTION:

Search 'backend-dev' knowledge base for:
  • Request validation patterns
  • Zod integration examples
  • Express middleware setup

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Appendix B: Installation Script Template

```bash
#!/bin/bash
# install-catalyst-amazon-q.sh

set -e

CATALYST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_DIR="${HOME}/.claude-hooks/bin"
Q_AGENTS_DIR="${HOME}/.aws/amazonq/cli-agents"
CATALYST_AGENT_DIR="${Q_AGENTS_DIR}/catalyst"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Catalyst - Amazon Q Installation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if Amazon Q is installed
if ! command -v q &> /dev/null; then
    echo "❌ Amazon Q Developer CLI not found"
    echo "   Please install from: https://aws.amazon.com/q/developer/"
    exit 1
fi

echo "✓ Amazon Q CLI found"
echo ""

# Build Rust hooks
echo "🔨 Building hooks..."
cd "${CATALYST_DIR}/.claude/hooks/RustHooks"
cargo build --release

# Install binaries
echo "📦 Installing binaries to ${HOOKS_DIR}..."
mkdir -p "${HOOKS_DIR}"
cp target/release/skill-activation-prompt "${HOOKS_DIR}/"
cp target/release/file-analyzer "${HOOKS_DIR}/"
cp target/release/file-change-tracker "${HOOKS_DIR}/"
chmod +x "${HOOKS_DIR}"/*

echo "✓ Binaries installed"
echo ""

# Create catalyst agent
echo "🤖 Creating catalyst agent..."
mkdir -p "${CATALYST_AGENT_DIR}"
cp "${CATALYST_DIR}/catalyst-agent.json" "${CATALYST_AGENT_DIR}/"

echo "✓ Agent created"
echo ""

# Set as default agent
echo "⚙️  Setting catalyst as default agent..."
q settings chat.defaultAgent catalyst

echo "✓ Default agent configured"
echo ""

# Add skills as knowledge bases
echo "📚 Adding skills as knowledge bases..."
q chat << 'EOF'
/knowledge add --name skill-developer --path ~/.claude/skills/skill-developer/
/knowledge add --name backend-dev --path ~/.claude/skills/backend-dev-guidelines/
/knowledge add --name frontend-dev --path ~/.claude/skills/frontend-dev-guidelines/
/knowledge add --name route-tester --path ~/.claude/skills/route-tester/
/knowledge add --name error-tracking --path ~/.claude/skills/error-tracking/
/knowledge add --name rust-developer --path ~/.claude/skills/rust-developer/
/exit
EOF

echo "✓ Knowledge bases added"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✅ Installation Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "  1. Run: q chat"
echo "  2. Ask a question about your project"
echo "  3. Catalyst will suggest relevant skills automatically"
echo ""
echo "Per-project customization (optional):"
echo "  cd your-project"
echo "  mkdir .catalyst"
echo "  cp ~/.claude/skills/skill-rules.json .catalyst/"
echo "  # Edit pathPatterns for your project structure"
echo ""
```

---

## Conclusion

Amazon Q Developer CLI compatibility with Catalyst is **highly feasible** with the following approach:

1. **Auto-activation:** ✅ Solved via default agent + userPromptSubmit hook
2. **Skill content:** ✅ Delivered via Knowledge Bases (potentially more efficient than progressive disclosure)
3. **Binary reuse:** ✅ Same Rust hooks work for both platforms with minor adaptations
4. **Implementation:** 2-3 weeks for full integration
5. **User experience:** Different but potentially better workflow

**Recommendation:** If you use or plan to use Amazon Q, implementing Catalyst support is worthwhile. The architecture is sound, implementation is straightforward, and the Knowledge Base approach may offer advantages over Claude Code's progressive disclosure.

**Next actions:**
- Test Amazon Q's Knowledge Bases with sample Catalyst skill
- Validate default agent + hook execution model
- Prototype hook adaptation for Q's JSON schema
- Decide whether to implement based on test results

---

**Document Version:** 1.0
**Last Updated:** 2025-01-05
**Status:** Research Complete - Ready for Implementation Decision

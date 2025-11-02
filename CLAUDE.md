# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## What This Repository Is

**A reference library of Claude Code infrastructure** - NOT a working application.

This showcase contains production-tested skills, hooks, agents, and slash commands extracted from 6 months of real-world use managing TypeScript microservices. Users will ask you to help them integrate these components into their own projects.

**Critical:** When users ask to "add [component]", they mean copy it to THEIR project, not modify this showcase.

---

## Your Role: Integration Assistant

**Primary tasks:**
1. Help users integrate components from this showcase into their projects
2. Customize configurations for their specific project structure
3. Verify integrations work correctly
4. Explain how components function

**Before starting ANY integration:**
- Read `CLAUDE_INTEGRATION_GUIDE.md` - contains detailed integration instructions
- Ask about user's project structure
- Verify tech stack compatibility for skills
- Never assume directory structures

---

## Repository Architecture

```
.claude/
├── skills/                 # 5 production skills
│   ├── backend-dev-guidelines/     (Node.js/Express/Prisma)
│   ├── frontend-dev-guidelines/    (React/MUI v7/TanStack)
│   ├── skill-developer/            (Meta-skill, framework-agnostic)
│   ├── route-tester/               (JWT cookie auth testing)
│   ├── error-tracking/             (Sentry integration)
│   └── skill-rules.json            (Activation configuration)
├── hooks/                  # Rust-based hooks for automation
│   ├── RustHooks/                  (Rust implementation)
│   ├── skill-activation-prompt.sh  (ESSENTIAL - auto-activates skills)
│   ├── post-tool-use-tracker.sh    (ESSENTIAL - tracks file changes)
│   ├── tsc-check.sh                (Optional - monorepo TypeScript checks)
│   └── ... (other optional hooks)
├── agents/                 # 10 specialized agents (standalone)
│   ├── code-architecture-reviewer.md
│   ├── refactor-planner.md
│   ├── frontend-error-fixer.md
│   └── ... (7 more)
└── commands/               # 3 slash commands
    ├── dev-docs.md
    ├── dev-docs-update.md
    └── route-research-for-testing.md

dev/
└── README.md               # Dev docs pattern documentation
```

---

## Core Concepts

### 1. Skill Auto-Activation System

**The breakthrough feature** that solves "skills don't activate automatically":

**Components:**
- `skill-activation-prompt` hook (UserPromptSubmit)
- `post-tool-use-tracker` hook (PostToolUse)
- `skill-rules.json` configuration

**How it works:**
1. Hook runs on every user prompt
2. Checks skill-rules.json for trigger patterns (keywords, file paths, intent)
3. Automatically suggests relevant skills
4. Skills load only when needed

**These hooks work for any project without customization.**

### 2. Modular Skills (500-Line Rule)

Skills use progressive disclosure to avoid context limits:

```
skill-name/
  SKILL.md                  # <500 lines, overview + navigation
  resources/
    topic-1.md              # <500 lines each, deep dives
    topic-2.md
```

Claude loads main skill first, resources only when needed.

### 3. Dev Docs Pattern

Three-file structure for maintaining context across resets:

```
dev/active/[task-name]/
├── [task-name]-plan.md      # Strategic plan
├── [task-name]-context.md   # Current state, decisions
└── [task-name]-tasks.md     # Checklist
```

See `dev/README.md` for full pattern documentation.

---

## Rust Hook Implementation

**All hooks are implemented in Rust** for maximum performance and zero runtime dependencies.

### Performance Benefits:

| Metric | Rust Performance |
|--------|-----------------|
| **Startup Time** | ~2ms (60x faster than interpreted languages) |
| **Memory Usage** | 3-5MB (10x less than managed runtimes) |
| **Binary Size** | 1.8-2.4MB (self-contained) |
| **Runtime Required** | None (zero dependencies) |

### Installation Options:

**Recommended: Standalone Installation**

Linux / macOS:
```bash
# Build once, use everywhere
./install.sh
# Binaries installed to ~/.claude-hooks/bin/

# Or with SQLite support
./install.sh --sqlite
```

Windows:
```powershell
# Build once, use everywhere
.\install.ps1
# Binaries installed to %USERPROFILE%\.claude-hooks\bin\

# Or with SQLite support
.\install.ps1 -Sqlite
```

**Benefits:**
- ✅ Compile once (45s), use everywhere (0s per project)
- ✅ Update in one place, all projects benefit
- ✅ Tiny per-project footprint (50 bytes vs 2MB)
- ✅ Consistent version across all projects
- ✅ Idiomatic Cargo features (no multiple Cargo files)
- ✅ Works on Linux, macOS, and Windows

**See Complete Documentation:**
- `docs/rust-hooks.md` - Full implementation guide
- `docs/performance-comparison.md` - Performance analysis
- `docs/databases.md` - SQLite state management
- `docs/standalone-installation.md` - Installation details

---

## Integration Workflows

### When User Says: "Add [skill] to my project"

**Steps:**
1. **Ask clarifying questions:**
   - "What's your project structure? Monorepo or single app?"
   - "Where is your [backend/frontend] code located?"
   - For backend-dev-guidelines: "Do you use Express and Prisma?"
   - For frontend-dev-guidelines: "Do you use React with MUI v7?"

2. **Check tech stack compatibility:**
   - backend-dev-guidelines requires: Node.js/Express/Prisma/TypeScript
   - frontend-dev-guidelines requires: React 18+/MUI v7/TanStack Query/Router
   - skill-developer: ✅ No requirements (framework-agnostic)
   - route-tester: JWT cookie-based auth
   - error-tracking: Sentry

3. **If tech stack doesn't match:**
   - Offer to adapt the skill for their stack
   - Extract framework-agnostic patterns only
   - Recommend skipping if too different

4. **Copy and customize:**
   - Copy skill directory to their `.claude/skills/`
   - Update `pathPatterns` in skill-rules.json for THEIR structure
   - Never keep example paths (blog-api, frontend/, etc.)

5. **Verify:**
   - Check skill copied correctly
   - Validate skill-rules.json syntax
   - Test activation suggestion

### When User Says: "Set up skill auto-activation"

**Steps:**
1. **Check if they have settings.json:**
   - If YES: Read it, merge hook configurations
   - If NO: Create new settings.json

2. **Install Rust hooks:**

   Linux / macOS:
   ```bash
   # From catalyst directory
   ./install.sh

   # Or with SQLite support
   ./install.sh --sqlite
   ```

   Windows:
   ```powershell
   # From catalyst directory
   .\install.ps1

   # Or with SQLite support
   .\install.ps1 -Sqlite
   ```

3. **Create hook wrappers in their project:**

   Linux / macOS:
   ```bash
   cd your-project/.claude/hooks/

   cat > skill-activation-prompt.sh << 'EOF'
   #!/bin/bash
   cat | ~/.claude-hooks/bin/skill-activation-prompt
   EOF

   cp catalyst/.claude/hooks/post-tool-use-tracker.sh .

   chmod +x *.sh
   ```

   Windows:
   ```powershell
   cd your-project\.claude\hooks\

   # Copy PowerShell wrappers from catalyst
   Copy-Item catalyst\.claude\hooks\*.ps1 .
   ```

4. **Update settings.json** with hook configurations:
   - UserPromptSubmit → skill-activation-prompt.sh/.ps1 (depending on OS)
   - PostToolUse → post-tool-use-tracker.sh/.ps1 (depending on OS)
   - Preserve any existing config

5. **DO NOT copy Stop hooks** unless user has monorepo and specifically requests them

### When User Says: "Add [agent]"

**Steps:**
1. Copy agent file to their `.claude/agents/`
2. Check for hardcoded paths in agent file
3. Replace with `$CLAUDE_PROJECT_DIR` or relative paths
4. Explain how to invoke the agent

**Note:** Agents are standalone - no configuration needed!

### When User Says: "Add [slash command]"

**Steps:**
1. Copy command file to their `.claude/commands/`
2. Ask about paths (e.g., "Where do you want dev docs stored?")
3. Update path references in command file
4. Command is immediately available as `/command-name`

---

## Tech Stack Compatibility Matrix

| Component | Requirements | Action if Mismatch |
|-----------|-------------|-------------------|
| **backend-dev-guidelines** | Express/Prisma/Node | Offer to adapt OR extract architecture patterns |
| **frontend-dev-guidelines** | React/MUI v7/TanStack | Offer to adapt for their framework |
| **skill-developer** | None | ✅ Copy as-is |
| **route-tester** | JWT cookie auth | Adapt for their auth or skip |
| **error-tracking** | Sentry | Adapt for their error tracking |
| **All hooks** | Rust (or compile from source) | ✅ Install via install.sh (Linux/macOS) or install.ps1 (Windows) |
| **All agents** | None | ✅ Copy as-is |

---

## Critical Rules for Integration

### ❌ NEVER Do This:

1. **Don't copy settings.json as-is** - Extract only the hooks user needs
2. **Don't keep example paths** - blog-api/, frontend/, services/ are examples
3. **Don't assume monorepo** - Most projects are single-service
4. **Don't skip tech stack check** - Skills have framework requirements
5. **Don't copy Stop hooks blindly** - They're heavy and project-specific
6. **Don't modify this showcase** - Help users integrate into THEIR project

### ✅ ALWAYS Do This:

1. **Read CLAUDE_INTEGRATION_GUIDE.md first** - Complete integration instructions
2. **Ask about project structure** - Never assume
3. **Verify tech stack compatibility** - Especially for framework-specific skills
4. **Customize pathPatterns** - Match user's actual directory structure
5. **Make hooks executable** - chmod +x after copying
6. **Test after integration** - Verify skill activates correctly
7. **Explain what you did** - Show commands, explain why

---

## Common Integration Patterns

### Monorepo pathPatterns:
```json
{
  "pathPatterns": [
    "packages/*/src/**/*.ts",
    "apps/*/src/**/*.tsx",
    "services/*/src/**/*.ts"
  ]
}
```

### Single-app pathPatterns:
```json
{
  "pathPatterns": [
    "src/**/*.ts",
    "backend/**/*.ts"
  ]
}
```

### Safe generic patterns (when unsure):
```json
{
  "pathPatterns": [
    "**/*.ts",
    "src/**/*.ts"
  ]
}
```

---

## Adapting Skills for Different Tech Stacks

**When user has different framework (e.g., Vue instead of React):**

1. **Ask if they want adaptation:**
   ```
   The frontend-dev-guidelines is for React + MUI v7. I can:
   1. Adapt it for Vue + [their UI library]
   2. Extract framework-agnostic patterns only
   3. Skip it

   Which would you prefer?
   ```

2. **If adapting:**
   - Copy skill as starting point
   - Replace framework-specific examples
   - Keep architecture patterns (file organization, performance principles)
   - Update skill name and triggers

3. **What usually transfers across stacks:**
   - ✅ Layered architecture patterns
   - ✅ File organization strategies
   - ✅ Error handling philosophy
   - ✅ Testing strategies
   - ✅ Performance optimization principles
   - ❌ Framework-specific code/APIs

---

## Verification Checklist

After any integration, verify:

```bash
# 1. Files copied correctly
ls -la .claude/skills/[skill-name]
ls -la .claude/hooks/
ls -la .claude/agents/

# 2. Hooks are executable
ls -la .claude/hooks/*.sh
# Should show: -rwxr-xr-x

# 3. Rust binaries installed
ls -la ~/.claude-hooks/bin/

# 4. skill-rules.json is valid JSON
cat .claude/skills/skill-rules.json | jq .

# 5. settings.json is valid JSON
cat .claude/settings.json | jq .
```

Then ask user to test activation.

---

## How Components Work Together

```
User types prompt
    ↓
skill-activation-prompt hook (UserPromptSubmit)
    ↓
Checks skill-rules.json
    ↓
Matches keywords/paths/intent
    ↓
Suggests relevant skill
    ↓
User accepts → Skill loads
    ↓
Skill provides guidelines + resource files
    ↓
User edits files
    ↓
post-tool-use-tracker hook (PostToolUse)
    ↓
Tracks changes for context management
```

---

## Special Notes

### About This Repository's Structure

- No src/ directory - this is a reference library, not an app
- Examples use generic blog domain (Post/Comment/User)
- Service names (blog-api, auth-service) are examples only
- Hooks implemented in Rust for maximum performance

### Dev Docs Examples

- See `dev/README.md` for dev docs pattern documentation
- Pattern is explained but no active docs exist in this showcase
- Users implement this pattern in their own projects

### Settings.json in This Repo

The `.claude/settings.json` here is an EXAMPLE showing:
- How to configure the two essential hooks
- Optional Stop hooks (monorepo-specific)
- Don't copy this file directly to user projects

---

## When Users Ask "How Do I...?"

**"How do I make skills auto-activate?"**
→ Run ./install.sh (Linux/macOS) or .\install.ps1 (Windows) from catalyst directory, create wrappers in your project, configure settings.json

**"How do I create my own skill?"**
→ Load the skill-developer skill, it teaches skill creation for any tech stack

**"How do I test authenticated routes?"**
→ Use route-tester skill (requires JWT cookie auth) or auth-route-tester agent

**"How do I maintain context across resets?"**
→ Use dev docs pattern (see dev/README.md) with /dev-docs slash command

**"Why isn't my skill activating?"**
→ Check: skill-rules.json pathPatterns, hooks installed, hooks executable, settings.json configured

**"Do I need Rust installed?"**
→ Only if building from source. Prebuilt binaries available via install.sh

---

## Repository Metadata

**Purpose:** Reference library for Claude Code infrastructure
**Origin:** 6 months of production use on TypeScript microservices project
**Domain:** Generic blog examples (Post/Comment/User)
**Hook Implementation:** Rust (maximum performance, zero dependencies)
**Not:** A working application, starter template, or boilerplate

**When helping users:** You're helping them extract and adapt components for THEIR project, not modifying this showcase.

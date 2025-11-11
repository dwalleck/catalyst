# Svelte Skill

A comprehensive Claude Code skill for Svelte 5 development, following the 500-line modular resource pattern.

## Overview

This skill provides complete Svelte 5 documentation extracted from the official Svelte docs (llms.txt), organized into bite-sized, context-efficient resource files.

**Total Coverage:** 9,691 lines of Svelte documentation
**Files:** 24 resource files + 1 main SKILL.md + 1 skill-rules.json
**Max File Size:** 638 lines (all under 700, most under 500)

## Structure

```
svelte-skill/
├── SKILL.md                          (436 lines) - Main navigation & overview
├── skill-rules.json                  - Auto-activation triggers
├── README.md                         - This file
└── resources/                        (24 files, 9,691 total lines)
    ├── Core Concepts (10 files)
    │   ├── components.md             (79 lines)
    │   ├── runes-state.md            (371 lines)
    │   ├── runes-derived-effect.md   (471 lines)
    │   ├── runes-props-utilities.md  (366 lines)
    │   ├── template-control-flow.md  (470 lines)
    │   ├── template-snippets.md      (578 lines)
    │   ├── directives-bind-use.md    (482 lines)
    │   ├── directives-animations.md  (638 lines)
    │   ├── styling.md                (178 lines)
    │   └── special-elements.md       (262 lines)
    ├── Advanced Topics (3 files)
    │   ├── advanced-concepts-1.md    (342 lines)
    │   ├── advanced-concepts-2.md    (338 lines)
    │   └── typescript.md             (282 lines)
    ├── Migration & Upgrades (3 files)
    │   ├── migration-svelte4.md      (245 lines)
    │   ├── migration-svelte5-1.md    (483 lines)
    │   └── migration-svelte5-2.md    (493 lines)
    └── Reference & Troubleshooting (8 files)
        ├── faqs-1.md                 (457 lines)
        ├── faqs-2.md                 (450 lines)
        ├── compiler-errors-1.md      (502 lines)
        ├── compiler-errors-2.md      (350 lines)
        ├── compiler-warnings-1.md    (493 lines)
        ├── compiler-warnings-2.md    (519 lines)
        ├── runtime-errors-1.md       (421 lines)
        └── runtime-errors-2.md       (421 lines)
```

## Usage

### Automatic Activation

The skill auto-activates when:
- Working with `.svelte`, `.svelte.js`, or `.svelte.ts` files
- Mentioning keywords: `svelte`, `runes`, `$state`, `$derived`, `$effect`, `$props`
- User intent indicates Svelte development (e.g., "create svelte component")

### Manual Activation

Load the skill explicitly:
```
/skill svelte-skill
```

### Progressive Disclosure

Start with **SKILL.md** for high-level guidance, then load specific resource files as needed:

**For beginners:**
1. components.md
2. runes-state.md
3. runes-derived-effect.md
4. template-control-flow.md

**For Svelte 4 users:**
1. migration-svelte5-1.md
2. migration-svelte5-2.md
3. All runes-*.md files

**For specific tasks:**
- State management → runes-state.md
- Computed values → runes-derived-effect.md
- Component props → runes-props-utilities.md
- Forms → directives-bind-use.md
- Animations → directives-animations.md
- Troubleshooting → faqs-1.md, compiler-errors-1.md

## Key Features

### Comprehensive Coverage
- ✅ Complete Svelte 5 runes system ($state, $derived, $effect, $props, etc.)
- ✅ All template syntax (control flow, snippets, special tags)
- ✅ All directives (bind, use, transition, animate, class, style)
- ✅ Styling (scoped, global, custom properties)
- ✅ Advanced patterns (stores, context, lifecycle)
- ✅ TypeScript integration
- ✅ Migration guides (Svelte 4 → 5)
- ✅ Complete error/warning reference
- ✅ FAQs and troubleshooting

### Optimized for Context Limits
- All files under 700 lines (most under 500)
- Logical topic boundaries
- No redundancy between files
- Clear cross-references

### Best Practices Included
- Recommended patterns
- Anti-patterns to avoid
- Performance considerations
- Type safety guidelines

## Content Map

### Core Learning Path
1. **Components** (79 lines) - File structure basics
2. **Runes - State** (371 lines) - Reactive state management
3. **Runes - Derived & Effect** (471 lines) - Computed values & side effects
4. **Runes - Props & Utilities** (366 lines) - Component communication
5. **Template Control Flow** (470 lines) - Conditionals & loops
6. **Template Snippets** (578 lines) - Reusable fragments
7. **Directives - Bind & Use** (482 lines) - Two-way binding & actions
8. **Directives - Animations** (638 lines) - Transitions & animations

### Reference Documentation
- **Special Elements** (262 lines) - svelte:window, svelte:head, etc.
- **Styling** (178 lines) - Scoped CSS, globals
- **TypeScript** (282 lines) - Type safety
- **Advanced Concepts** (680 lines total) - Stores, context, lifecycle
- **Errors & Warnings** (2,706 lines total) - Complete diagnostic reference
- **FAQs** (907 lines total) - Common questions

### Migration Resources
- **Svelte 4 Guide** (245 lines) - Svelte 3 → 4
- **Svelte 5 Guide** (976 lines total) - Svelte 4 → 5 (runes migration)

## Integration with Claude Code

This skill follows the **catalyst reference architecture** for skills:

1. **SKILL.md** - Navigation hub (<500 lines)
2. **resources/** - Deep-dive documentation (<700 lines each)
3. **skill-rules.json** - Auto-activation configuration
4. **Progressive disclosure** - Load only what you need

## Source

Extracted from the official Svelte documentation (`llms.txt` - 16,249 lines) and split into modular resources following the 500-line rule for optimal Claude Code performance.

**Original source:** Svelte 5 official documentation
**Extraction date:** 2025
**Svelte version:** 5.x (latest stable)

## Maintenance

To update this skill with new Svelte documentation:

1. Download latest `llms.txt` from Svelte docs
2. Run extraction script (use line ranges from SVELTE_DOCS_CONTENT_MAP.md)
3. Verify all files remain under 700 lines
4. Update SKILL.md resource table if structure changes
5. Test skill activation triggers

## License

This skill contains documentation from the official Svelte project. The Svelte documentation is licensed under the MIT License. This skill packaging is part of the catalyst reference library.

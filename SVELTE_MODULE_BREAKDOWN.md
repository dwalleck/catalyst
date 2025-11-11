# Svelte Documentation Module Breakdown

Visual guide showing how to partition the 16,249-line Svelte documentation into optimal skill resource modules.

---

## Module Distribution Overview

```
llms.txt (16,249 total lines)
│
├─ Foundation & Setup              (Module 1:   1-143,    142 lines)
├─ Runes (Reactivity)              (Module 2: 143-1350,  1207 lines) ★★★ CRITICAL
├─ Template Syntax                 (Module 3: 1351-3518, 2168 lines) ★★★ CRITICAL
├─ Styling                         (Module 4: 3519-3696,  178 lines)
├─ Special Elements                (Module 5: 3697-3958,  262 lines)
├─ State Management                (Module 6: 3959-4638,  680 lines)
├─ Testing & TypeScript            (Module 7: 4639-5243,  605 lines)
├─ Custom Elements                 (Module 8: 5244-5372,  129 lines)
├─ Migration Guide (v4→v5)         (Module 9: 5373-6593, 1221 lines)
├─ FAQs                            (Module 10: 6594-6710, 117 lines)
├─ Component API                   (Module 11: 6746-7341, 596 lines)
├─ Type Definitions                (Module 12: 7365-8318, 954 lines)
├─ Built-in Utilities & Easing     (Module 13: 8319-12398, 4080 lines) ★ LARGE
├─ Error & Warning Reference       (Module 14: 12399-15460, 3062 lines) ★ LARGE
├─ Svelte 4 Legacy API             (Module 15: 15461-16249, 789 lines)
│
└─ TOTAL: 15 major modules
```

---

## Priority-Based Extraction Plan

### PHASE 1: Core Skills (5-6 files, ~3,800 lines)
**Time: 1-2 hours | Impact: 80% of daily needs**

The minimum viable Svelte skill resource library. Focus here if short on time.

```
ESSENTIAL EXTRACTING:
├─ intro-setup.md                  (1-143)        142 lines
├─ runes-intro-state-core.md       (143-288)      145 lines
├─ runes-state-advanced.md         (289-513)      224 lines
├─ runes-derived.md                (514-646)      132 lines
├─ runes-effect.md                 (647-984)      337 lines
├─ runes-props.md                  (985-1205)     220 lines
├─ template-basics.md              (1351-1566)    215 lines
├─ template-control-flow.md        (1567-2099)    532 lines
├─ template-bindings.md            (2399-2880)    481 lines
├─ state-stores.md                 (3959-4264)    305 lines
├─ state-lifecycle.md              (4393-4556)    163 lines
└─ styling-essentials.md           (3519-3696)    177 lines
```

**Why these first?**
- ★★★ Runes are the new Svelte 5 paradigm - essential for modern development
- ★★★ Template syntax is the daily bread of component development
- ★★ State management and styling are fundamental to any app
- Foundation files establish context and getting started

### PHASE 2: Extended Skills (9 files, ~2,100 lines)
**Time: 1-2 hours | Impact: Additional 15% of needs**

High-value resources for intermediate development.

```
HIGH-VALUE ADDITIONS:
├─ special-elements.md             (3697-3958)    262 lines
├─ state-context.md                (4265-4392)    127 lines
├─ testing-strategies.md           (4639-4971)    332 lines
├─ typescript-integration.md       (4972-5243)    271 lines
├─ custom-elements.md              (5244-5372)    129 lines
├─ template-snippets-deep.md       (1821-2099)    279 lines
├─ template-transitions-anim.md    (2881-3329)    448 lines
├─ template-special-tags.md        (2100-2365)    265 lines
└─ faq.md                          (6594-6710)    117 lines
```

**Why add these?**
- Testing and TypeScript are production-essential
- Transitions/animations unlock advanced UI capabilities
- Context and snippets solve common architecture problems

### PHASE 3: API & Reference (19 files, ~7,500 lines)
**Time: 2-3 hours | Impact: Lookup reference material**

Complete reference documentation - lower priority for learning but essential for reference.

```
COMPLETE REFERENCE:
├─ api-component-functions.md      (6746-7341)    595 lines
├─ api-types-1.md                  (7365-7550)    185 lines
├─ api-types-2.md                  (7550-7800)    250 lines
├─ api-types-3.md                  (7800-8000)    200 lines
├─ api-types-4.md                  (8000-8318)    318 lines
├─ api-easing.md                   (9654-10030)   376 lines
├─ api-animations.md               (10433-10893)  460 lines
├─ api-event-handlers.md           (10031-10401)  370 lines
├─ api-dom-utilities.md            (10713-11114)  401 lines
├─ api-svelte-utils.md             (11114-11430)  316 lines
├─ api-store-utils.md              (11430-11841)  411 lines
├─ api-transitions.md              (11826-12398)  572 lines
├─ errors-compiler.md              (12399-13607) 1208 lines
├─ warnings-compiler.md            (13608-14619) 1011 lines
├─ errors-runtime.md               (14620-15051)  431 lines
└─ (AST & advanced compiler docs)  (8319-9653)  1334 lines
```

**Why separate?**
- These are lookup references, not learning material
- Users consult these when they know what they need
- Can be deferred until skill core is solid

### PHASE 4: Migration & Legacy (8 files, ~2,005 lines)
**Time: 1 hour | Impact: Maintenance & legacy code**

Lower priority for new projects, essential for upgrading existing Svelte 4 projects.

```
MIGRATION & LEGACY:
├─ migration-overview.md           (5373-5500)    127 lines
├─ migration-reactivity.md         (5501-6077)    576 lines
├─ migration-components.md         (6078-6593)    515 lines
├─ legacy-reactive-decl.md         (15461-15592)  131 lines
├─ legacy-component-exports.md     (15593-15663)   70 lines
├─ legacy-events-slots.md          (15664-15946)  282 lines
├─ legacy-component-elements.md    (15947-16051)  104 lines
└─ legacy-imperative-api.md        (16052-16249)  197 lines
```

**Why last?**
- Only needed when working with Svelte 4 codebases
- Migration is a one-time task per project
- Can be added on-demand

---

## Detailed Module Deep-Dives

### Module 2: RUNES (Lines 143-1350, 1207 lines)

**Critical module - heart of Svelte 5**

```
Runes Module Breakdown:
│
├─ What are runes? (143-165)           23 lines  [INTRO]
│   └─ Sets context, explains paradigm shift
│
├─ $state Rune (166-513)              348 lines [★★★]
│   ├─ Core concept (166-288)         122 lines
│   │   └─ Basic usage, reactive updates
│   ├─ Variants (289-347)              59 lines
│   │   ├─ $state.raw        (289-314)
│   │   ├─ $state.snapshot   (315-331)
│   │   └─ $state.eager      (332-347)
│   └─ Advanced patterns (347-513)     167 lines
│       ├─ Passing into functions (347-433)
│       └─ Passing across modules   (434-513)
│
├─ $derived Rune (514-646)            132 lines [★★]
│   ├─ Core concept (514-536)          23 lines
│   ├─ $derived.by variant (537-559)   23 lines
│   ├─ Understanding dependencies (560-595)
│   └─ Advanced patterns (596-646)
│
├─ $effect Rune (647-984)             338 lines [★★★]
│   ├─ Core concept (647-812)         166 lines
│   ├─ Variants (813-902)              90 lines
│   │   ├─ $effect.pre        (813-849)
│   │   ├─ $effect.tracking   (850-867)
│   │   ├─ $effect.pending    (868-882)
│   │   └─ $effect.root       (883-901)
│   └─ When NOT to use (902-984)      119 lines
│
├─ $props Rune (985-1205)             220 lines [★★]
│   ├─ Core concept (985-1029)         45 lines
│   ├─ Patterns (1020-1150)           131 lines
│   │   ├─ Fallback values    (1020-1029)
│   │   ├─ Renaming props     (1030-1037)
│   │   ├─ Rest props         (1038-1045)
│   │   └─ Updating props     (1046-1150)
│   └─ Type safety (1150-1205)         56 lines
│
├─ $bindable Rune (1206-1258)          53 lines [★]
│   └─ Two-way binding in components
│
├─ $inspect Rune (1259-1314)           56 lines [★]
│   └─ Debugging utility
│
└─ $host Rune (1315-1350)              36 lines [★]
    └─ Custom element styling
```

**Extraction Strategy for Runes:**
1. **Create intro file** - What are runes + $state core (~280 lines)
2. **Create state file** - $state variants + advanced patterns (~340 lines)
3. **Create derived file** - Full $derived module (~390 lines)
4. **Create effect file** - Full $effect module (~430 lines)
5. **Create props file** - $props + type safety (~420 lines)
6. **Create utility file** - $bindable + $inspect + $host (~180 lines)

---

### Module 3: TEMPLATE SYNTAX (Lines 1351-3518, 2168 lines)

**Second most critical module - how to write Svelte markup**

```
Template Syntax Module Breakdown:
│
├─ Basic Markup (1351-1566)           215 lines [★★]
│   ├─ Tags & attributes (1355-1422)
│   ├─ Component props (1423-1432)
│   ├─ Event handling (1443-1498)
│   ├─ Text expressions (1499-1528)
│   └─ Comments (1529-1566)
│
├─ Control Flow Blocks (1567-2099)    532 lines [★★★] COMPLEX
│   ├─ {#if} (1567-1605)               39 lines
│   ├─ {#each} (1606-1720)            115 lines
│   │   ├─ Basic each
│   │   ├─ Keyed blocks    (1637-1679)
│   │   └─ Else blocks     (1704-1720)
│   ├─ {#key} (1721-1743)              23 lines
│   ├─ {#await} (1744-1820)            77 lines
│   └─ {#snippet} (1821-2099)         279 lines [★★]
│       ├─ Snippet scope (1876-1930)
│       ├─ Passing snippets (1931-2036)
│       ├─ Type safety (2037-2074)
│       └─ Snippets & slots (2075-2099)
│
├─ Special Tags (2100-2365)           265 lines [★]
│   ├─ {@render} (2100-2137)           38 lines
│   ├─ {@html} (2138-2187)             50 lines
│   ├─ {@attach} (2188-2352)          165 lines
│   ├─ {@const} (2353-2365)            13 lines
│   └─ {@debug} (2366-2398)            33 lines
│
├─ Binding Directive (2399-2880)      481 lines [★★★] CRITICAL
│   ├─ Overview (2399-2438)
│   ├─ Input bindings (2439-2589)     151 lines
│   │   ├─ bind:value
│   │   ├─ bind:checked
│   │   ├─ bind:group
│   │   └─ bind:files
│   ├─ Form elements (2590-2680)       91 lines
│   │   ├─ bind:value (select)
│   │   ├─ bind:value (audio/video)
│   │   └─ bind:open (details)
│   └─ Advanced bindings (2681-2880)
│       ├─ Dimensions & contenteditable
│       ├─ bind:this
│       └─ bind:_property_
│
├─ Transitions (2881-3051)            171 lines [★★]
│   ├─ Built-in transitions (2917-2920)
│   ├─ Custom functions (2933-3028)
│   └─ Events (3028-3051)
│
├─ Animations (3072-3184)             113 lines [★★]
│   └─ animate: directive
│
└─ Styling Directives (3185-3329)     144 lines [★]
    ├─ style: (3185-3226)
    └─ class: (3227-3329)
```

**Extraction Strategy for Templates:**
1. **Create basics file** - Tags, attributes, events, comments (~350 lines)
2. **Create control flow file** - if/each/key/await (~480 lines)
3. **Create snippets file** - Deep dive into {#snippet} (~320 lines)
4. **Create special tags file** - {@render}, {@html}, {@attach}, etc. (~220 lines)
5. **Create bindings file** - All bind: variants (~420 lines)
6. **Create transitions file** - Transitions and animations (~400 lines)

---

### Module 13: BUILT-IN UTILITIES (Lines 8319-12398, 4080 lines)

**Large reference module with many small functions**

```
Built-in Utilities Module Breakdown:
│
├─ Easing Functions (9654-10030)      376 lines
│   └─ 20+ easing types for animations
│
├─ Event Handlers (10031-10401)       371 lines
│   └─ 'on' directive and modifiers
│
├─ Animation System (10433-10893)     460 lines
│   ├─ spring()
│   ├─ tweened()
│   ├─ Spring type
│   └─ Tweened type
│
├─ DOM Media Queries (10713-11114)    401 lines
│   └─ Device pixel ratio, dimensions, scrolling
│
├─ Svelte Utilities (11114-11430)     316 lines
│   ├─ SvelteDate
│   ├─ SvelteMap
│   ├─ SvelteSet
│   ├─ SvelteURL
│   └─ SvelteURLSearchParams
│
├─ Store Utilities (11430-11841)      411 lines
│   ├─ readable()
│   ├─ writable()
│   ├─ derived()
│   ├─ get()
│   └─ Store types
│
└─ Transitions (11826-12398)          572 lines
    ├─ fade, fly, slide, scale, draw, crossfade
    ├─ blur, customTransition
    └─ Transition config types
```

**Extraction Strategy for Built-ins:**
- Each section is relatively self-contained
- Can create individual files per category
- 7-8 reference files total
- Lower learning priority, high lookup value

---

## Optimal Resource File Sizes

Based on analysis of content density:

```
OPTIMAL FILE SIZES FOR SKILL RESOURCES:
├─ Too Small     (< 100 lines) ❌
│   └─ Fragments (need combining)
│
├─ Small         (100-250 lines) ⚠️
│   └─ Can combine with similar topics
│
├─ Ideal         (250-450 lines) ✅
│   └─ Optimal for focused deep dives
│
├─ Good          (450-550 lines) ✅
│   └─ Can work if very coherent
│
└─ Large         (> 550 lines) ❌
    └─ Should split into multiple files
```

**Distribution for Svelte skill:**
- Small files (100-250): 5 files (intro, faq, custom elements, special elements, legacy sections)
- Ideal files (250-450): 25 files (most runes, templates, state management, testing)
- Good files (450-550): 8 files (large template sections, migration guides)
- Split large (> 550): 4 files (errors/warnings reference split into 4)

---

## Cross-Reference Map

How modules relate to each other:

```
LEARNING FLOW:
1. Start: Foundation & Setup (Module 1)
   ↓
2. Master: Runes (Module 2) ← Must understand for everything after
   ↓
3. Apply: Template Syntax (Module 3)
   ↓
4. Style: Styling (Module 4)
   ↓
5. Structure: State Management (Module 6)
   ↓
6. Enhance: Special Elements (Module 5), Transitions (in Module 3)
   ↓
7. Level Up: Testing (Module 7), TypeScript (Module 7)
   ↓
8. Reference: APIs (Modules 11-14), Errors (Module 14)
   ↓
9. Legacy: If upgrading from Svelte 4 (Module 15)


THEMATIC RELATIONSHIPS:
Runes (Module 2)
  ├─ Used everywhere in Template Syntax (Module 3)
  ├─ Foundation for State Management (Module 6)
  └─ Essential for TypeScript (Module 7)

Template Syntax (Module 3)
  ├─ Uses Runes (Module 2)
  ├─ Combined with Styling (Module 4)
  └─ With Special Elements (Module 5)

State Management (Module 6)
  ├─ Uses Runes (Module 2)
  ├─ With Context patterns
  └─ With Testing (Module 7)

Testing (Module 7)
  ├─ Tests components from Module 3
  ├─ Tests state from Module 6
  └─ Uses TypeScript from Module 7
```

---

## Implementation Roadmap

### Week 1: Core Skill (Phases 1-2)
```
Day 1: Extract Phase 1 fundamentals & runes
Day 2: Extract Phase 2 templates & state
Day 3: Create main SKILL.md navigation + resource index
Day 4-5: Test activation, document usage
```

### Week 2: Reference (Phase 3)
```
Day 6-7: Extract API reference section
Day 8-9: Extract error & warning reference
Day 10: Verify and link all references
```

### Optional Week 3: Migration & Legacy (Phase 4)
```
Day 11-12: Extract migration guides
Day 13-14: Extract Svelte 4 legacy API
Day 15: Complete documentation
```

---

## Quick Statistics

```
EXTRACTION EFFORT ESTIMATE:
├─ Phase 1 (Core): 5-6 files    ~3,800 lines    1-2 hours
├─ Phase 2 (Extended): 9 files  ~2,100 lines    1-2 hours
├─ Phase 3 (Reference): 19 files ~7,500 lines   2-3 hours
└─ Phase 4 (Legacy): 8 files    ~2,005 lines    1 hour
└─ TOTAL: 41-42 files            ~15,405 lines   5-8 hours

SKILL DENSITY:
├─ Learning material (P1-3): ~8,600 lines (57%)
├─ Reference material (P4-5): ~6,800 lines (43%)
└─ Total: 16,249 lines

SKILL STRUCTURE:
├─ Main SKILL.md: ~500 lines (navigation + fundamentals)
├─ Resource files: 40-45 files (~15,700 lines)
└─ Total: ~16,200 lines of skill content
```

---

## Troubleshooting Common Issues

**Q: Some rune sections feel too small**
A: Combine with related sections:
- `$bindable` with `$props` (related concept)
- `$inspect` + `$host` into utilities file

**Q: Template syntax file would be huge if complete**
A: Split strategically:
- Separate control flow (each, if, etc.)
- Separate directives (bind:, transition:, etc.)
- Separate special tags ({@render}, {@html})

**Q: API reference is overwhelming**
A: Create by category, not alphabetical:
- Easing functions together
- Animation types together
- Store utilities together
- Keep compiler errors/warnings separate from runtime

**Q: Should I extract the AST compiler section?**
A: Probably not for a user-facing skill.
- It's highly specialized
- Users doing AST work are likely aware of compiler docs
- Could create separate "Compiler Internals" skill if needed


# Svelte 5 Documentation Content Map

**File:** llms.txt
**Total Lines:** 16,249
**Analysis Date:** 2025-11-07

## Executive Summary

This document provides a complete content map of the Svelte documentation with exact line ranges for all major topic areas. The file naturally segments into **15 primary content modules** that can be further divided into **400-500 line chunks** for skill resources.

**Recommendation:** Create a modular skill structure with:
- Main SKILL.md (~500 lines): Navigation guide + foundational concepts
- 15-20 resource files (~400-500 lines each): Topic-specific deep dives

---

## Primary Content Modules

### Module 1: Introduction & Setup (Lines 1-143)
**Total Lines:** 142
**Purpose:** Getting started, project setup, tooling

| Section | Lines | Type | Size |
|---------|-------|------|------|
| Overview | 4-32 | Intro | 29 |
| Getting started | 33-63 | Setup | 31 |
| Alternatives to SvelteKit | 46-52 | Guide | 7 |
| Editor tooling | 54-58 | Guide | 5 |
| Getting help | 60-63 | Guide | 4 |
| .svelte files overview | 64-142 | Concepts | 79 |
| `<script>` tag | 89-116 | Reference | 28 |
| `<script module>` | 97-118 | Reference | 22 |
| `<style>` tag | 119-133 | Reference | 15 |

**Modularity:** Can remain as single ~140 line foundation resource

---

### Module 2: Component Files (.svelte.js/.svelte.ts) (Lines 134-142)
**Total Lines:** 9
**Purpose:** TypeScript and JavaScript module components

| Section | Lines | Type | Size |
|---------|-------|------|------|
| .svelte.js and .svelte.ts files | 134-142 | Reference | 9 |

**Modularity:** Merge with Module 1 (minimal content)

---

### Module 3: Runes - Core Reactivity (Lines 143-1350)
**Total Lines:** 1,207
**Purpose:** State management, derived state, effects, props, lifecycle
**Critical:** This is largest section - requires 3-4 resource files

#### Breakdown by Rune Type:

| Rune | Lines | Size | Notes |
|------|-------|------|-------|
| What are runes? (intro) | 143-165 | 23 | Essential intro |
| `$state` | 166-513 | 348 | **LARGE** - split into 2 chunks |
| → Core `$state` | 166-288 | 123 | Main rune |
| → `$state.raw`, `.snapshot`, `.eager` | 289-347 | 59 | Variants |
| → Passing state (functions & modules) | 347-513 | 167 | Advanced patterns |
| `$derived` | 514-646 | 133 | Standalone |
| → Core `$derived` | 514-536 | 23 | Main concept |
| → `$derived.by` | 537-559 | 23 | Variant |
| → Understanding dependencies | 560-609 | 50 | Advanced |
| `$effect` | 647-984 | 338 | **LARGE** - split into 2 chunks |
| → Core `$effect` | 647-812 | 166 | Main rune |
| → Effect variants | 813-902 | 90 | `.pre`, `.tracking`, `.pending`, `.root` |
| → When not to use `$effect` | 902-1020 | 119 | Critical patterns |
| `$props` | 985-1205 | 221 | Medium |
| → Core `$props` | 985-1029 | 45 | Main concept |
| → Patterns (fallback, renaming, rest, updating) | 1020-1150 | 131 | Advanced |
| → Type safety & `$props.id()` | 1150-1205 | 56 | TypeScript |
| `$bindable` | 1206-1258 | 53 | Standalone |
| `$inspect` | 1259-1314 | 56 | Standalone |
| `$host` | 1315-1350 | 36 | Standalone |

**Modularity Recommendation:**

**Resource files for Runes module:**
- `runes-intro-and-state.md` (280 lines) - What are runes + `$state` core
- `runes-state-advanced.md` (340 lines) - `$state` variants + passing state
- `runes-derived.md` (390 lines) - `$derived` all variants + understanding dependencies
- `runes-effect-core.md` (430 lines) - `$effect` core + variants + when not to use
- `runes-props.md` (420 lines) - `$props` + `$bindable` + type safety
- `runes-inspect-host.md` (180 lines) - `$inspect` + `$host`

---

### Module 4: Template Syntax - Markup & Control Flow (Lines 1351-3518)
**Total Lines:** 2,168
**Purpose:** HTML markup, event handling, conditional/loop rendering, directives
**Critical:** Second largest section - requires 4-5 resource files

#### Breakdown by Topic:

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Basic markup | 1351-1566 | 216 | Tags, attributes, spread |
| → Tags | 1355-1368 | 14 | Reference |
| → Element attributes | 1369-1422 | 54 | Reference |
| → Component props | 1423-1432 | 10 | Reference |
| → Spread attributes | 1433-1442 | 10 | Reference |
| → Events | 1443-1498 | 56 | Medium |
| → Text expressions | 1499-1528 | 30 | Reference |
| → Comments | 1529-1566 | 38 | Reference |
| Control Flow Blocks | 1567-2099 | 533 | **LARGE** |
| → `{#if}` | 1567-1605 | 39 | Reference |
| → `{#each}` | 1606-1720 | 115 | Medium (includes keyed/else blocks) |
| → `{#key}` | 1721-1743 | 23 | Reference |
| → `{#await}` | 1744-1820 | 77 | Medium |
| → `{#snippet}` | 1821-2099 | 279 | **LARGE** - includes scope, typing, exports |
| Special Tags | 2100-2365 | 266 | Standalone tags |
| → `{@render}` | 2100-2137 | 38 | Reference |
| → `{@html}` | 2138-2187 | 50 | Reference |
| → `{@attach}` | 2188-2352 | 165 | Medium |
| → `{@const}` | 2353-2365 | 13 | Reference |
| → `{@debug}` | 2366-2398 | 33 | Reference |
| Directives - Binding | 2399-2880 | 482 | **LARGE** |
| → bind: overview & function bindings | 2399-2438 | 40 | Reference |
| → Input bindings (value, checked, group, files) | 2439-2589 | 151 | Medium |
| → Select, audio, video, img bindings | 2590-2680 | 91 | Medium |
| → Binding special cases (contenteditable, dimensions, this, components) | 2681-2844 | 164 | Medium |
| → Binding typing | 2845-2880 | 36 | Reference |
| Directives - Transitions | 2881-3051 | 171 | Medium |
| → transition: overview & parameters | 2881-2932 | 52 | Reference |
| → Custom transitions & events | 2933-3051 | 119 | Medium |
| Directives - Animations & Styling | 3052-3329 | 278 | Medium |
| → in: and out: | 3051-3071 | 21 | Reference |
| → animate: | 3072-3184 | 113 | Medium |
| → style: | 3185-3226 | 42 | Reference |
| → class directive | 3227-3329 | 103 | Medium |
| Async Data Handling | 3328-3518 | 191 | Medium |
| → await overview | 3328-3406 | 79 | Medium |
| → Loading states, error handling, server-side rendering | 3407-3510 | 104 | Medium |
| → Forking and caveats | 3511-3518 | 8 | Reference |

**Modularity Recommendation:**

**Resource files for Template Syntax module:**
- `template-basics.md` (350 lines) - Tags, attributes, events, text, comments
- `template-control-flow.md` (480 lines) - if/each/key/await blocks
- `template-snippets.md` (320 lines) - `{#snippet}`, `{@render}`, scope, typing
- `template-special-tags.md` (220 lines) - `{@html}`, `{@attach}`, `{@const}`, `{@debug}`
- `template-binding-directive.md` (420 lines) - All bind: variants and patterns
- `template-transitions-animations.md` (400 lines) - transition:, in:, out:, animate:
- `template-styling-directives.md` (380 lines) - style:, class, and async data handling

---

### Module 5: Styling (Lines 3519-3696)
**Total Lines:** 178
**Purpose:** CSS scoping, global styles, custom properties

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Scoped styles | 3519-3556 | 38 | Reference |
| → Specificity | 3534-3539 | 6 | Reference |
| → Scoped keyframes | 3540-3556 | 17 | Reference |
| Global styles | 3557-3620 | 64 | Reference |
| → :global(...) | 3559-3596 | 38 | Reference |
| → :global (selector) | 3597-3620 | 24 | Reference |
| Custom properties | 3621-3676 | 56 | Reference |
| Nested `<style>` elements | 3677-3696 | 20 | Reference |

**Modularity:** Single ~178 line resource file (can combine if needed)

---

### Module 6: Special Elements (Lines 3697-3958)
**Total Lines:** 262
**Purpose:** Svelte-specific HTML elements

| Element | Lines | Size | Notes |
|---------|-------|------|-------|
| `<svelte:boundary>` | 3697-3799 | 103 | Reference |
| `<svelte:window>` | 3800-3842 | 43 | Reference |
| `<svelte:document>` | 3843-3869 | 27 | Reference |
| `<svelte:body>` | 3870-3883 | 14 | Reference |
| `<svelte:head>` | 3884-3900 | 17 | Reference |
| `<svelte:element>` | 3901-3932 | 32 | Reference |
| `<svelte:options>` | 3933-3958 | 26 | Reference |

**Modularity:** Single ~262 line resource file

---

### Module 7: State & Lifecycle (Lines 3959-4638)
**Total Lines:** 680
**Purpose:** Stores, context, lifecycle hooks, imperative API

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Stores | 3959-4264 | 306 | **LARGE** |
| → Stores intro & when to use | 3959-4020 | 62 | Reference |
| → svelte/store functions | 4021-4249 | 229 | Medium |
| → Store contract | 4250-4264 | 15 | Reference |
| Context | 4265-4392 | 128 | Medium |
| → Context overview | 4265-4302 | 38 | Reference |
| → Using context with state | 4303-4345 | 43 | Reference |
| → Type-safe context | 4346-4361 | 16 | Reference |
| → Replacing global state | 4362-4392 | 31 | Reference |
| Lifecycle hooks | 4393-4556 | 164 | Medium |
| → Hook overview | 4393-4401 | 9 | Reference |
| → onMount | 4402-4435 | 34 | Reference |
| → onDestroy | 4436-4451 | 16 | Reference |
| → tick | 4452-4468 | 17 | Reference |
| → Deprecated hooks | 4469-4556 | 88 | Reference |
| Imperative component API | 4557-4638 | 82 | Medium |
| → mount | 4569-4587 | 19 | Reference |
| → unmount | 4588-4605 | 18 | Reference |
| → render | 4606-4621 | 16 | Reference |
| → hydrate | 4622-4638 | 17 | Reference |

**Modularity Recommendation:**

**Resource files for State & Lifecycle module:**
- `state-stores.md` (380 lines) - Stores intro, svelte/store, contract, when to use
- `state-context.md` (280 lines) - Context, type-safe context, patterns
- `state-lifecycle.md` (280 lines) - Lifecycle hooks, imperative API

---

### Module 8: Testing & TypeScript (Lines 4639-5243)
**Total Lines:** 605
**Purpose:** Testing strategies and TypeScript integration

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Testing | 4639-4971 | 333 | Medium |
| → Unit/component tests with Vitest | 4643-4884 | 242 | Medium |
| → Component tests with Storybook | 4885-4926 | 42 | Reference |
| → E2E tests with Playwright | 4927-4971 | 45 | Reference |
| TypeScript | 4972-5243 | 272 | Medium |
| → TypeScript intro | 4972-4997 | 26 | Reference |
| → Preprocessor setup | 4998-5040 | 43 | Reference |
| → tsconfig.json settings | 5041-5048 | 8 | Reference |
| → Typing `$props` | 5049-5078 | 30 | Reference |
| → Generic `$props` | 5079-5101 | 23 | Reference |
| → Typing wrapper components | 5102-5131 | 30 | Reference |
| → Typing `$state` | 5132-5158 | 27 | Reference |
| → The `Component` type | 5159-5211 | 53 | Reference |
| → Enhancing built-in DOM types | 5212-5243 | 32 | Reference |

**Modularity Recommendation:**

**Resource files for Testing & TypeScript module:**
- `testing-strategies.md` (380 lines) - All testing approaches
- `typescript-integration.md` (380 lines) - TypeScript setup, typing, types

---

### Module 9: Custom Elements (Lines 5244-5372)
**Total Lines:** 129
**Purpose:** Building custom elements with Svelte

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Custom Elements with Svelte | 5244-5372 | 129 | Medium |
| → Custom elements overview | 5453-5463 | 11 | Reference |
| → SvelteComponentTyped deprecation | 5464-5496 | 33 | Reference |
| → Component lifecycle | 5295-5304 | 10 | Reference |
| → Component options | 5305-5359 | 55 | Reference |
| → Caveats and limitations | 5360-5372 | 13 | Reference |

**Modularity:** Single ~129 line resource file

---

### Module 10: Migration & Breaking Changes (Lines 5373-6593)
**Total Lines:** 1,221
**Purpose:** Svelte 4 to 5 migration guide
**Critical:** Requires 2-3 resource files

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Svelte 4 Migration Guide | 5373-6135 | 763 | **LARGE** |
| → Minimum version requirements | 5379-5387 | 9 | Reference |
| → Browser conditions | 5388-5395 | 8 | Reference |
| → CJS removal | 5396-5399 | 4 | Reference |
| → Stricter types | 5400-5452 | 53 | Reference |
| → Custom elements section | 5453-5497 | 45 | Reference |
| → Transitions local by default | 5497-5511 | 15 | Reference |
| → Default slot bindings | 5512-5532 | 21 | Reference |
| → Preprocessors | 5533-5602 | 70 | Reference |
| → ESLint package | 5603-5606 | 4 | Reference |
| → Other breaking changes (early) | 5607-5623 | 17 | Reference |
| → Reactivity syntax changes | 5624-5735 | 112 | Medium |
| → Event changes | 5736-5955 | 220 | **LARGE** |
| → Snippets instead of slots | 5956-6077 | 122 | Medium |
| → Migration script | 6078-6133 | 56 | Reference |
| Components no longer classes | 6134-6258 | 125 | Medium |
| `<svelte:component>` changes | 6259-6293 | 35 | Reference |
| Whitespace handling | 6294-6303 | 10 | Reference |
| Modern browser requirement | 6304-6313 | 10 | Reference |
| Compiler options changes | 6314-6322 | 9 | Reference |
| Reserved props | 6323-6326 | 4 | Reference |
| Runes mode breaking changes | 6327-6444 | 118 | Medium |
| Other breaking changes (late) | 6445-6593 | 149 | Medium |

**Modularity Recommendation:**

**Resource files for Migration module:**
- `migration-overview-breaking-changes.md` (480 lines) - Overview, minimum requirements, stricter types, breaking changes intro
- `migration-reactivity-events.md` (420 lines) - Reactivity syntax, event changes, snippets vs slots
- `migration-components-advanced.md` (380 lines) - Components no longer classes, component changes, runes mode, caveats

---

### Module 11: FAQs (Lines 6594-6710)
**Total Lines:** 117
**Purpose:** Frequently asked questions

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Getting started | 6596-6601 | 6 | FAQ |
| Support | 6602-6609 | 8 | FAQ |
| Third-party resources | 6610-6613 | 4 | FAQ |
| VS Code syntax highlighting | 6614-6617 | 4 | FAQ |
| Code formatting | 6618-6621 | 4 | FAQ |
| Component documentation | 6622-6652 | 31 | FAQ |
| Scalability | 6653-6656 | 4 | FAQ |
| UI component libraries | 6657-6660 | 4 | FAQ |
| Testing | 6661-6682 | 22 | FAQ |
| Routing | 6683-6688 | 6 | FAQ |
| Mobile apps | 6689-6694 | 6 | FAQ |
| Unused styles | 6695-6705 | 11 | FAQ |
| Svelte v2 | 6706-6711 | 6 | FAQ |
| Hot module reloading | 6712-6745 | 34 | FAQ |

**Modularity:** Single ~117 line resource file

---

### Module 12: API Reference - Components & General (Lines 6746-7341)
**Total Lines:** 596
**Purpose:** Component API documentation

| Item | Lines | Type | Size |
|------|-------|------|------|
| SvelteComponent | 6746-6865 | Reference | 120 |
| SvelteComponentTyped | 6866-6887 | Reference | 22 |
| Component lifecycle & methods | 6888-7305 | Reference | 418 |
| → afterUpdate | 6888-6911 | Reference | 24 |
| → beforeUpdate | 6912-6935 | Reference | 24 |
| → createContext | 6936-6957 | Reference | 22 |
| → createEventDispatcher | 6958-6995 | Reference | 38 |
| → createRawSnippet | 6996-7014 | Reference | 19 |
| → flushSync | 7015-7029 | Reference | 15 |
| → fork | 7030-7061 | Reference | 32 |
| → getAbortSignal | 7062-7095 | Reference | 34 |
| → getAllContexts | 7096-7113 | Reference | 18 |
| → getContext | 7114-7130 | Reference | 17 |
| → hasContext | 7131-7145 | Reference | 15 |
| → hydrate | 7146-7183 | Reference | 38 |
| → mount | 7184-7206 | Reference | 23 |
| → onDestroy | 7207-7223 | Reference | 17 |
| → onMount | 7224-7249 | Reference | 26 |
| → setContext | 7250-7269 | Reference | 20 |
| → settled | 7270-7290 | Reference | 21 |
| → tick | 7291-7304 | Reference | 14 |
| → unmount | 7305-7340 | Reference | 36 |
| untrack | 7341-7365 | Reference | 25 |

**Modularity Recommendation:**

**Resource files for API Reference - Components:**
- `api-components-overview.md` (320 lines) - SvelteComponent, SvelteComponentTyped, basic API
- `api-context-lifecycle.md` (380 lines) - Context, lifecycle, component manipulation
- `api-utilities.md` (300 lines) - Utilities (flushSync, fork, etc.) and untrack

---

### Module 13: API Reference - Types (Lines 7365-8318)
**Total Lines:** 954
**Purpose:** Type definitions and interfaces

| Item | Lines | Type | Size |
|------|-------|------|------|
| Component | 7365-7445 | Type | 81 |
| ComponentConstructorOptions | 7446-7554 | Type | 109 |
| ComponentEvents | 7555-7573 | Type | 19 |
| ComponentInternals | 7574-7585 | Type | 12 |
| ComponentProps | 7586-7631 | Type | 46 |
| ComponentType | 7632-7658 | Type | 27 |
| EventDispatcher | 7659-7683 | Type | 25 |
| Fork | 7684-7726 | Type | 43 |
| MountOptions | 7727-7775 | Type | 49 |
| Snippet | 7776-7812 | Type | 37 |
| Action | 7813-7854 | Type | 42 |
| ActionReturn | 7855-7918 | Type | 64 |
| Compiler functions | 7919-8039 | Reference | 121 |
| → flip | 7919-7943 | Reference | 25 |
| → AnimationConfig | 7944-7996 | Reference | 53 |
| → FlipParams | 7997-8038 | Reference | 42 |
| → createAttachmentKey | 8039-8076 | Reference | 38 |
| → fromAction | 8077-8122 | Reference | 46 |
| → Attachment | 8123-8160 | Reference | 38 |
| VERSION | 8161-8174 | Reference | 14 |
| Compiler functions (compile, compileModule, migrate, parse) | 8175-8278 | Reference | 104 |
| → compile | 8175-8191 | Reference | 17 |
| → compileModule | 8192-8208 | Reference | 17 |
| → migrate | 8209-8236 | Reference | 28 |
| → parse | 8237-8277 | Reference | 41 |
| Preprocessor functions | 8278-8318 | Reference | 41 |
| → preprocess | 8278-8300 | Reference | 23 |
| → walk | 8301-8318 | Reference | 18 |

**Modularity Recommendation:**

**Resource files for API Reference - Types:**
- `api-types-component.md` (420 lines) - Component, ComponentConstructorOptions, ComponentProps, ComponentEvents
- `api-types-extensions.md` (300 lines) - ComponentInternals, ComponentType, EventDispatcher, Fork, MountOptions
- `api-types-snippets-actions.md` (280 lines) - Snippet, Action, ActionReturn
- `api-compiler-functions.md` (380 lines) - Compiler functions (compile, parse, migrate, preprocess, walk)

---

### Module 14: API Reference - Built-ins (Lines 8319-12398)
**Total Lines:** 4,080
**Purpose:** Built-in easing, animations, stores, DOM utilities
**Critical:** Very large section - requires 6-8 resource files

#### Breakdown:

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| AST & Compiler | 8319-9653 | 1335 | **VERY LARGE** |
| Easing functions | 9654-10030 | 377 | Large |
| Event handlers | 10031-10401 | 371 | Large |
| Legacy components | 10401-10569 | 169 | Medium |
| Animation (Spring & Tween) | 10433-10893 | 461 | Large |
| DOM media queries | 10713-11114 | 402 | Large |
| Svelte utilities (SvelteDate, SvelteMap, etc.) | 11114-11430 | 317 | Large |
| Store utilities | 11430-11841 | 412 | Large |
| Transitions (built-in) | 11826-12398 | 573 | Large |

**Modularity Recommendation:**

**Resource files for Built-ins module:**
- `api-builtin-easing.md` (420 lines) - All easing functions
- `api-builtin-animations.md` (480 lines) - Spring, Tween, tweened, spring functions
- `api-builtin-events.md` (420 lines) - Event handlers, on directive, modifiers
- `api-builtin-dom-utilities.md` (420 lines) - DOM media queries, device utilities, media query objects
- `api-builtin-svelte-utilities.md` (420 lines) - SvelteDate, SvelteMap, SvelteSet, SvelteURL, SvelteURLSearchParams
- `api-builtin-stores.md` (460 lines) - Store utilities, readable, writable, derived, get, etc.
- `api-builtin-transitions.md` (480 lines) - All transition functions and types

**Note:** AST documentation (8319-9653) is highly specialized compiler documentation that may warrant its own section or be deferred.

---

### Module 15: Errors & Warnings (Lines 12399-15052)
**Total Lines:** 2,654
**Purpose:** Error and warning reference
**Critical:** Large reference section - requires 3-4 resource files

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Compiler errors | 12399-13607 | 1209 | **VERY LARGE** |
| Compiler warnings | 13608-14619 | 1012 | **LARGE** |
| Runtime errors | 14620-15051 | 432 | Large |
| Runtime warnings | 15052-15460 | 409 | Large |

**Modularity Recommendation:**

**Resource files for Errors & Warnings module:**
- `errors-compiler.md` (800 lines) - Compiler errors
- `errors-compiler-warnings.md` (700 lines) - Compiler warnings
- `errors-runtime.md` (500 lines) - Runtime errors and warnings

---

### Module 16: Svelte 4 Legacy API (Lines 15461-16249)
**Total Lines:** 789
**Purpose:** Svelte 4 reactive declarations, legacy component API
**Note:** Legacy documentation - lower priority for extraction

| Topic | Lines | Size | Notes |
|-------|-------|------|-------|
| Overview | 15461-15473 | 13 | Reference |
| Reactive let/var declarations | 15474-15506 | 33 | Reference |
| Reactive $: statements | 15507-15592 | 86 | Reference |
| export let | 15593-15663 | 71 | Reference |
| $$props and $$restProps | 15664-15692 | 29 | Reference |
| on: directive | 15693-15827 | 135 | Medium |
| `<slot>` element | 15828-15946 | 119 | Medium |
| $$slots | 15947-15972 | 26 | Reference |
| `<svelte:fragment>` | 15973-16003 | 31 | Reference |
| `<svelte:component>` (legacy) | 16004-16015 | 12 | Reference |
| `<svelte:self>` | 16016-16051 | 36 | Reference |
| Imperative component API | 16052-16249 | 198 | Medium |

**Modularity Recommendation:**

**Resource files for Legacy API module:**
- `legacy-reactivity.md` (280 lines) - Reactive declarations and $: statements
- `legacy-component-api.md` (330 lines) - export let, $$props, imperative API
- `legacy-slots-directives.md` (380 lines) - on:, <slot>, $$slots, fragments, svelte:component, svelte:self

---

## Summary Statistics

| Category | Count | Total Lines | Avg Lines/Section |
|----------|-------|-------------|-------------------|
| Level 1 Headers (Major Sections) | 74 | 16,249 | 220 |
| Level 2 Headers (Subsections) | 335 | 16,249 | 48 |
| Recommended Resource Files | 45-50 | 16,249 | 330-360 |
| Optimal Chunk Size | - | - | 400-500 |

---

## Recommended File Structure for Svelte Skill

```
svelte-5-development/
├── SKILL.md                          (~500 lines)
│   ├── Navigation guide
│   ├── Quick reference index
│   └── Tech stack requirements
│
├── resources/
│   ├── fundamentals/
│   │   ├── intro-setup.md            (142 lines)
│   │   ├── component-structure.md    (178 lines)
│   │   └── styling-essentials.md     (178 lines)
│   │
│   ├── runes/
│   │   ├── runes-intro-state.md      (280 lines)
│   │   ├── runes-state-advanced.md   (340 lines)
│   │   ├── runes-derived.md          (390 lines)
│   │   ├── runes-effect.md           (430 lines)
│   │   ├── runes-props.md            (420 lines)
│   │   └── runes-inspect-host.md     (180 lines)
│   │
│   ├── templates/
│   │   ├── template-basics.md        (350 lines)
│   │   ├── template-control-flow.md  (480 lines)
│   │   ├── template-snippets.md      (320 lines)
│   │   ├── template-special-tags.md  (220 lines)
│   │   ├── template-bindings.md      (420 lines)
│   │   ├── template-transitions.md   (400 lines)
│   │   └── template-styling.md       (380 lines)
│   │
│   ├── state-advanced/
│   │   ├── state-stores.md           (380 lines)
│   │   ├── state-context.md          (280 lines)
│   │   └── state-lifecycle.md        (280 lines)
│   │
│   ├── development/
│   │   ├── testing-strategies.md     (380 lines)
│   │   └── typescript-integration.md (380 lines)
│   │
│   ├── special/
│   │   ├── special-elements.md       (262 lines)
│   │   ├── custom-elements.md        (129 lines)
│   │   └── faq.md                    (117 lines)
│   │
│   ├── migration/
│   │   ├── migration-overview.md     (480 lines)
│   │   ├── migration-reactivity.md   (420 lines)
│   │   └── migration-components.md   (380 lines)
│   │
│   ├── api-reference/
│   │   ├── api-components.md         (~960 lines, split into 3 files)
│   │   ├── api-types.md              (~960 lines, split into 4 files)
│   │   ├── api-builtin.md            (~2000 lines, split into 7 files)
│   │   └── api-errors.md             (~2654 lines, split into 3 files)
│   │
│   └── legacy-svelte4/
│       ├── legacy-reactivity.md      (280 lines)
│       ├── legacy-api.md             (330 lines)
│       └── legacy-slots.md           (380 lines)
```

---

## Key Extraction Tips

### For Large Sections (1000+ lines):
1. **Runes module** - Extract intro separately, then group by rune type
2. **Template syntax** - Group by functionality (control flow, directives, bindings)
3. **Built-ins** - Group by domain (easing, animations, stores, transitions)
4. **Errors** - Create separate files for compiler vs runtime, errors vs warnings

### For Medium Sections (300-700 lines):
- Can often stay as single 400-500 line files
- Consider subsection importance when deciding to split

### For Small Sections (<200 lines):
- Combine related sections to reach ~400 line minimum
- Ensures files aren't too fragmented

### Extraction Commands:
```bash
# Extract lines 166-513 for $state rune
sed -n '166,513p' llms.txt > runes-state-core.md

# Extract lines 143-165 for runes intro
sed -n '143,165p' llms.txt > runes-intro.md

# Extract multiple sections combined
(sed -n '3519,3556p' llms.txt; echo ""; sed -n '3677,3696p' llms.txt) > styling-combined.md
```

---

## Prioritization for Skill Creation

**Phase 1 (Essential - Extract First):**
- Intro & Setup (Module 1)
- Runes (Module 3) - Most important for modern Svelte
- Template Syntax (Module 4) - Core markup knowledge

**Phase 2 (High Value):**
- State & Lifecycle (Module 7)
- Styling (Module 5)
- Special Elements (Module 6)

**Phase 3 (Development):**
- Testing & TypeScript (Module 8)
- Custom Elements (Module 9)

**Phase 4 (Reference):**
- API References (Modules 12-14)
- Errors & Warnings (Module 15)

**Phase 5 (Maintenance):**
- Migration Guide (Module 10)
- FAQs (Module 11)
- Legacy API (Module 16)

---

## Notes for Implementation

1. **Consistent metadata:** Each resource file should include:
   - Topic title and description
   - Primary use cases
   - Cross-references to related topics
   - Code examples extracted from original

2. **Progressive disclosure:** Main SKILL.md should:
   - Provide navigation to all resource files
   - Explain when to use each resource
   - Highlight most common use cases

3. **Search optimization:** Consider creating an index in main SKILL.md with:
   - Full topic list with line numbers
   - Quick lookup by concept name
   - FAQ cross-references

4. **Keep line numbers:** When extracting, preserve section identifiers for traceability to original


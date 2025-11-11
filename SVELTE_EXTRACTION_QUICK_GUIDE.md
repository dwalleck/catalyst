# Svelte Documentation Extraction Quick Guide

Quick reference for extracting content from `llms.txt` into modular skill resources.

---

## One-Line Extraction Commands

Use these `sed` commands to extract exact line ranges from llms.txt:

### Fundamentals (Start Here)
```bash
# Introduction & Setup (142 lines)
sed -n '1,143p' llms.txt > intro-setup.md

# Styling (178 lines)
sed -n '3519,3696p' llms.txt > styling-essentials.md

# Special Elements (262 lines)
sed -n '3697,3958p' llms.txt > special-elements.md

# FAQs (117 lines)
sed -n '6594,6710p' llms.txt > faq.md
```

### Runes (Most Important Module)
```bash
# Runes Intro & $state core (280 lines recommended)
sed -n '143,288p' llms.txt > runes-intro-state.md

# $state advanced patterns (340 lines)
sed -n '289,513p' llms.txt > runes-state-advanced.md

# $derived with examples (390 lines)
sed -n '514,646p' llms.txt > runes-derived.md

# $effect core to advanced (430 lines)
sed -n '647,984p' llms.txt > runes-effect.md

# $props and related (420 lines)
sed -n '985,1205p' llms.txt > runes-props.md

# $bindable, $inspect, $host (180 lines)
sed -n '1206,1350p' llms.txt > runes-inspect-host.md
```

### Template Syntax (Second Most Important)
```bash
# Basic markup & events (216 lines)
sed -n '1351,1566p' llms.txt > template-basics.md

# Control flow blocks (533 lines) - **LARGE**
sed -n '1567,2099p' llms.txt > template-control-flow.md

# Snippets deep dive (279 lines)
sed -n '1821,2099p' llms.txt > template-snippets-deep.md

# Binding directive (482 lines) - **LARGE**
sed -n '2399,2880p' llms.txt > template-bindings.md

# Transitions, animations, styling (471 lines)
sed -n '2881,3329p' llms.txt > template-transitions-styling.md
```

### State & Lifecycle
```bash
# Stores (306 lines)
sed -n '3959,4264p' llms.txt > state-stores.md

# Context (128 lines)
sed -n '4265,4392p' llms.txt > state-context.md

# Lifecycle hooks (164 lines)
sed -n '4393,4556p' llms.txt > state-lifecycle.md

# Imperative API (82 lines)
sed -n '4557,4638p' llms.txt > state-imperative-api.md
```

### Development
```bash
# Testing strategies (333 lines)
sed -n '4639,4971p' llms.txt > testing-strategies.md

# TypeScript integration (272 lines)
sed -n '4972,5243p' llms.txt > typescript-integration.md

# Custom elements (129 lines)
sed -n '5244,5372p' llms.txt > custom-elements.md
```

### Migration Guide (Svelte 4 → 5)
```bash
# Breaking changes overview (390 lines)
sed -n '5373,5500p' llms.txt > migration-overview.md

# Reactivity & event changes (332 lines)
sed -n '5501,6077p' llms.txt > migration-reactivity-events.md

# Component changes & caveats (516 lines)
sed -n '6078,6593p' llms.txt > migration-components-advanced.md
```

### API Reference
```bash
# Component API functions (596 lines)
sed -n '6746,7341p' llms.txt > api-component-functions.md

# Component types (954 lines) - **LARGE**
sed -n '7365,8318p' llms.txt > api-types.md

# Easing functions (377 lines)
sed -n '9654,10030p' llms.txt > api-easing.md

# Animation functions (461 lines)
sed -n '10433,10893p' llms.txt > api-animations.md

# DOM utilities (402 lines)
sed -n '10713,11114p' llms.txt > api-dom-utilities.md

# Store utilities (412 lines)
sed -n '11430,11841p' llms.txt > api-stores.md

# Transitions (573 lines)
sed -n '11826,12398p' llms.txt > api-transitions.md
```

### Errors & Warnings
```bash
# Compiler errors (1209 lines) - **VERY LARGE**
# Split into subsections or keep as reference
sed -n '12399,13607p' llms.txt > errors-compiler.md

# Compiler warnings (1012 lines) - **LARGE**
sed -n '13608,14619p' llms.txt > warnings-compiler.md

# Runtime errors & warnings (841 lines) - **LARGE**
sed -n '14620,15460p' llms.txt > errors-runtime.md
```

### Legacy Svelte 4 API (Lower Priority)
```bash
# Reactive declarations (119 lines)
sed -n '15461,15592p' llms.txt > legacy-reactive-declarations.md

# Component exports & props (106 lines)
sed -n '15593,15663p' llms.txt > legacy-component-exports.md

# Event & slot handling (281 lines)
sed -n '15664,15946p' llms.txt > legacy-events-slots.md

# Fragment & component elements (97 lines)
sed -n '15947,16051p' llms.txt > legacy-component-elements.md

# Imperative API (198 lines)
sed -n '16052,16249p' llms.txt > legacy-imperative-api.md
```

---

## Batch Extraction Script

Extract all priority 1 & 2 resources at once:

```bash
#!/bin/bash

# Create resources directory
mkdir -p resources/{fundamentals,runes,templates,state,development,migration,api,errors,legacy}

# Fundamentals
sed -n '1,143p' llms.txt > resources/fundamentals/intro-setup.md
sed -n '3519,3696p' llms.txt > resources/fundamentals/styling-essentials.md
sed -n '3697,3958p' llms.txt > resources/fundamentals/special-elements.md
sed -n '6594,6710p' llms.txt > resources/fundamentals/faq.md

# Runes (Priority 1)
sed -n '143,288p' llms.txt > resources/runes/intro-state.md
sed -n '289,513p' llms.txt > resources/runes/state-advanced.md
sed -n '514,646p' llms.txt > resources/runes/derived.md
sed -n '647,984p' llms.txt > resources/runes/effect.md
sed -n '985,1205p' llms.txt > resources/runes/props.md
sed -n '1206,1350p' llms.txt > resources/runes/inspect-host.md

# Templates (Priority 1)
sed -n '1351,1566p' llms.txt > resources/templates/basics.md
sed -n '1567,2099p' llms.txt > resources/templates/control-flow.md
sed -n '2399,2880p' llms.txt > resources/templates/bindings.md
sed -n '2881,3329p' llms.txt > resources/templates/transitions-styling.md

# State & Lifecycle (Priority 2)
sed -n '3959,4264p' llms.txt > resources/state/stores.md
sed -n '4265,4392p' llms.txt > resources/state/context.md
sed -n '4393,4556p' llms.txt > resources/state/lifecycle.md

# Development (Priority 2)
sed -n '4639,4971p' llms.txt > resources/development/testing.md
sed -n '4972,5243p' llms.txt > resources/development/typescript.md
sed -n '5244,5372p' llms.txt > resources/development/custom-elements.md

# Migration (Priority 3)
sed -n '5373,5500p' llms.txt > resources/migration/overview.md
sed -n '5501,6077p' llms.txt > resources/migration/reactivity-events.md
sed -n '6078,6593p' llms.txt > resources/migration/components.md

# API Reference (Priority 4)
sed -n '6746,7341p' llms.txt > resources/api/component-functions.md
sed -n '7365,8318p' llms.txt > resources/api/types.md
sed -n '9654,10030p' llms.txt > resources/api/easing.md
sed -n '10433,10893p' llms.txt > resources/api/animations.md

# Errors (Priority 4)
sed -n '12399,13607p' llms.txt > resources/errors/compiler.md
sed -n '14620,15460p' llms.txt > resources/errors/runtime.md

echo "Extraction complete!"
ls -la resources/**/*.md | wc -l
```

---

## Line Range Reference Table

Quick lookup table for all major sections:

| Topic | Start | End | Lines | Type |
|-------|-------|-----|-------|------|
| Overview & Getting Started | 1 | 143 | 142 | Foundation |
| Runes - Intro & $state | 143 | 288 | 145 | Critical |
| Runes - $state advanced | 289 | 513 | 224 | Critical |
| Runes - $derived | 514 | 646 | 132 | Critical |
| Runes - $effect | 647 | 984 | 337 | Critical |
| Runes - $props | 985 | 1205 | 220 | Critical |
| Runes - $bindable/$inspect/$host | 1206 | 1350 | 144 | Important |
| Template - Basics | 1351 | 1566 | 215 | Critical |
| Template - Control Flow | 1567 | 2099 | 532 | Critical |
| Template - Special Tags | 2100 | 2365 | 265 | Important |
| Template - Bindings | 2399 | 2880 | 481 | Critical |
| Template - Transitions/Animations | 2881 | 3329 | 448 | Important |
| Styling | 3519 | 3696 | 177 | Important |
| Special Elements | 3697 | 3958 | 261 | Important |
| Stores | 3959 | 4264 | 305 | Important |
| Context | 4265 | 4392 | 127 | Important |
| Lifecycle | 4393 | 4556 | 163 | Important |
| Testing | 4639 | 4971 | 332 | Important |
| TypeScript | 4972 | 5243 | 271 | Important |
| Custom Elements | 5244 | 5372 | 128 | Moderate |
| Migration - Overview | 5373 | 5500 | 127 | Reference |
| Migration - Reactivity | 5501 | 6077 | 576 | Reference |
| Migration - Components | 6078 | 6593 | 515 | Reference |
| FAQs | 6594 | 6710 | 116 | Reference |
| Component API | 6746 | 7341 | 595 | Reference |
| Component Types | 7365 | 8318 | 953 | Reference |
| Easing Functions | 9654 | 10030 | 376 | Reference |
| Animations | 10433 | 10893 | 460 | Reference |
| DOM Utilities | 10713 | 11114 | 401 | Reference |
| Store Utilities | 11430 | 11841 | 411 | Reference |
| Transitions | 11826 | 12398 | 572 | Reference |
| Compiler Errors | 12399 | 13607 | 1208 | Reference |
| Compiler Warnings | 13608 | 14619 | 1011 | Reference |
| Runtime Errors | 14620 | 15051 | 431 | Reference |
| Runtime Warnings | 15052 | 15460 | 408 | Reference |
| Legacy - Reactive Declarations | 15461 | 15592 | 131 | Legacy |
| Legacy - Component Exports | 15593 | 15663 | 70 | Legacy |
| Legacy - Events & Slots | 15664 | 15946 | 282 | Legacy |
| Legacy - Component Elements | 15947 | 16051 | 104 | Legacy |
| Legacy - Imperative API | 16052 | 16249 | 197 | Legacy |

---

## Tips for Clean Extraction

1. **Remove system tags** (if present):
   ```bash
   sed -n 'START,ENDp' llms.txt | grep -v "^<SYSTEM>" > output.md
   ```

2. **Add file headers** to each extracted file:
   ```bash
   (echo "# Topic Name"; echo ""; echo "Lines: START-END from llms.txt"; echo ""; cat file.md) > file-with-header.md
   ```

3. **Verify extraction count**:
   ```bash
   sed -n '1,143p' llms.txt | wc -l  # Should output 143
   ```

4. **Create index document**:
   ```bash
   ls -1 resources/**/*.md | while read f; do
     wc -l "$f"
   done | sort -rn | head -20
   ```

---

## Validation Checklist

After extraction, verify:

- [ ] All files have clear headings
- [ ] Line counts match expected ranges (±2 lines for blank lines)
- [ ] No duplicate sections across files
- [ ] Cross-references updated (if present)
- [ ] Code examples complete and properly formatted
- [ ] No truncated content at boundaries
- [ ] File hierarchy matches resource organization

---

## File Size Summary

Total extractable content by priority:

| Priority | Topic Count | Total Lines | Avg per File |
|----------|-------------|-------------|--------------|
| P1 (Essential) | 12 | ~3,800 | ~320 |
| P2 (High Value) | 9 | ~2,100 | ~230 |
| P3 (Development) | 3 | ~730 | ~240 |
| P4 (Reference) | 15 | ~7,500 | ~500 |
| P5 (Legacy) | 5 | ~784 | ~157 |
| **TOTAL** | **44** | **~14,900** | **~340** |

---

## Recommended Extraction Phases

**Phase 1 (1-2 hours):** Extract P1 + P2 (21 files, ~5,900 lines)
- Focus on Runes, Templates, Styling, Lifecycle, State

**Phase 2 (1 hour):** Extract P3 (3 files, ~730 lines)
- Testing, TypeScript, Custom Elements

**Phase 3 (2-3 hours):** Extract P4 Reference (15 files, ~7,500 lines)
- API documentation and error reference

**Phase 4 (1 hour):** Extract P5 Legacy (5 files, ~784 lines)
- Svelte 4 compatibility docs

**Total Time: 5-7 hours** for complete skill resource library


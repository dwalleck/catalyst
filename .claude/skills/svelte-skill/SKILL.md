---
tags: [svelte, frontend, reactive, components, web-development]
version: 5.x
framework: svelte
description: Comprehensive Svelte 5 development guidelines covering runes, components, reactivity, and modern web development patterns
---

# Svelte Development Skill

**Expert guidance for building modern web applications with Svelte 5**

This skill provides comprehensive documentation for Svelte 5, the revolutionary web framework that compiles declarative components into highly optimized JavaScript. Use this skill when working with `.svelte` files, implementing reactive state management, or building component-based user interfaces.

## What is Svelte?

Svelte is a **compiler-based framework** for building user interfaces. Unlike traditional frameworks that do their work in the browser, Svelte shifts that work into a compile step that happens at build time. This means:

- **No virtual DOM** - Svelte compiles components to highly efficient imperative code
- **True reactivity** - Reactive values update automatically without manual subscriptions
- **Small bundle sizes** - Only the code you use is included
- **Simple syntax** - Familiar HTML, CSS, and JavaScript with minimal boilerplate

## Quick Start

**Creating a new Svelte project with SvelteKit:**

```bash
npx sv create myapp
cd myapp
npm install
npm run dev
```

**Basic component structure:**

```svelte
<script>
  let count = $state(0);

  function increment() {
    count++;
  }
</script>

<button onclick={increment}>
  Clicks: {count}
</button>

<style>
  button {
    font-size: 1.5em;
    padding: 0.5em 1em;
  }
</style>
```

## When to Use This Skill

**Activate this skill when:**
- Creating or editing `.svelte` component files
- Implementing reactive state with runes (`$state`, `$derived`, `$effect`)
- Working with component props and two-way bindings
- Using template syntax (conditionals, loops, await blocks)
- Styling components with scoped CSS
- Integrating with SvelteKit for full-stack applications
- Migrating from Svelte 4 to Svelte 5
- Debugging reactivity or component lifecycle issues

## Core Concepts Overview

### 1. **Components** → [See resources/components.md](resources/components.md)
   - `.svelte` file structure (`<script>`, markup, `<style>`)
   - Component lifecycle and instantiation
   - `.svelte.js` and `.svelte.ts` modules for shared reactive logic

### 2. **Runes (Svelte 5's Reactivity System)**
   → [runes-state.md](resources/runes-state.md) | [runes-derived-effect.md](resources/runes-derived-effect.md) | [runes-props-utilities.md](resources/runes-props-utilities.md)
   - **`$state`** - Create reactive state
   - **`$derived`** - Computed values that update automatically
   - **`$effect`** - Side effects that run when dependencies change
   - **`$props`** - Component inputs with type safety
   - **`$bindable`** - Two-way binding between parent and child
   - **`$inspect`** - Development-time debugging tool

### 3. **Template Syntax**
   → [template-control-flow.md](resources/template-control-flow.md) | [template-snippets.md](resources/template-snippets.md)
   - **`{#if}` / `{:else}`** - Conditional rendering
   - **`{#each}`** - List rendering with keying
   - **`{#await}`** - Async data handling
   - **`{#snippet}`** - Reusable template fragments
   - **`{@render}`** - Render snippets
   - **`{@html}`** - Inject raw HTML (use carefully!)
   - **`{@const}`** - Local constants in templates
   - **`{@debug}`** - Breakpoints in templates

### 4. **Directives**
   → [directives-bind-use.md](resources/directives-bind-use.md) | [directives-animations.md](resources/directives-animations.md)
   - **`bind:`** - Two-way data binding
   - **`use:`** - Actions for DOM element lifecycle
   - **`transition:`** - Smooth element transitions
   - **`in:` / `out:`** - Separate enter/exit animations
   - **`animate:`** - List reordering animations
   - **`class:`** - Conditional class application
   - **`style:`** - Dynamic inline styles

### 5. **Styling** → [See resources/styling.md](resources/styling.md)
   - Scoped styles (default behavior)
   - Global styles with `:global()`
   - CSS custom properties
   - Dynamic class and style bindings

### 6. **Advanced Patterns**
   → [advanced-concepts-1.md](resources/advanced-concepts-1.md) | [advanced-concepts-2.md](resources/advanced-concepts-2.md)
   - Context API for dependency injection
   - Stores for cross-component state
   - Lifecycle hooks (`onMount`, `onDestroy`, etc.)
   - Custom elements / Web Components
   - Server-side rendering considerations

## Resource Files Quick Reference

### Core Concepts
| Resource File | Topics Covered | Use When |
|--------------|----------------|----------|
| **[components.md](resources/components.md)** | File structure, script blocks, module context | Creating new components |
| **[runes-state.md](resources/runes-state.md)** | $state rune, deep reactivity, classes | Managing reactive state |
| **[runes-derived-effect.md](resources/runes-derived-effect.md)** | $derived, $effect, side effects | Computed values & effects |
| **[runes-props-utilities.md](resources/runes-props-utilities.md)** | $props, $bindable, $inspect, $host | Component props & utilities |
| **[template-control-flow.md](resources/template-control-flow.md)** | {#if}, {#each}, {#await}, {#key} | Conditional rendering & loops |
| **[template-snippets.md](resources/template-snippets.md)** | {#snippet}, {@render}, {@html}, {@const} | Reusable template fragments |
| **[directives-bind-use.md](resources/directives-bind-use.md)** | bind:, use: directives | Two-way binding & actions |
| **[directives-animations.md](resources/directives-animations.md)** | transition:, in:, out:, animate:, class:, style: | Animations & transitions |
| **[styling.md](resources/styling.md)** | Scoped CSS, global styles, dynamic styling | Styling components |
| **[special-elements.md](resources/special-elements.md)** | svelte:window, svelte:head, svelte:boundary | Built-in special elements |

### Advanced Topics
| Resource File | Topics Covered | Use When |
|--------------|----------------|----------|
| **[advanced-concepts-1.md](resources/advanced-concepts-1.md)** | Stores, context API | Cross-component state |
| **[advanced-concepts-2.md](resources/advanced-concepts-2.md)** | Lifecycle hooks, testing, imperative API | Advanced patterns |
| **[typescript.md](resources/typescript.md)** | Type safety, generics, proper typing | Adding TypeScript |

### Migration & Upgrades
| Resource File | Topics Covered | Use When |
|--------------|----------------|----------|
| **[migration-svelte4.md](resources/migration-svelte4.md)** | Svelte 4 migration guide | Migrating from Svelte 3 |
| **[migration-svelte5-1.md](resources/migration-svelte5-1.md)** | Svelte 5 migration: Runes & reactivity | Upgrading to Svelte 5 (part 1) |
| **[migration-svelte5-2.md](resources/migration-svelte5-2.md)** | Svelte 5 migration: Components & events | Upgrading to Svelte 5 (part 2) |

### Reference & Troubleshooting
| Resource File | Topics Covered | Use When |
|--------------|----------------|----------|
| **[faqs-1.md](resources/faqs-1.md)** | Frequently asked questions (part 1) | General questions |
| **[faqs-2.md](resources/faqs-2.md)** | Frequently asked questions (part 2) | More common questions |
| **[compiler-errors-1.md](resources/compiler-errors-1.md)** | Compiler error reference (A-M) | Debugging compile errors |
| **[compiler-errors-2.md](resources/compiler-errors-2.md)** | Compiler error reference (N-Z) | Debugging compile errors |
| **[compiler-warnings-1.md](resources/compiler-warnings-1.md)** | Compiler warnings (part 1) | Understanding warnings |
| **[compiler-warnings-2.md](resources/compiler-warnings-2.md)** | Compiler warnings (part 2) | Understanding warnings |
| **[runtime-errors-1.md](resources/runtime-errors-1.md)** | Runtime errors (part 1) | Debugging runtime issues |
| **[runtime-errors-2.md](resources/runtime-errors-2.md)** | Runtime errors & warnings (part 2) | Debugging runtime issues |

## Navigation Guide

### For Beginners
1. Start with **[components.md](resources/components.md)** - Learn file structure
2. Read **[runes-state.md](resources/runes-state.md)** - Understand reactive state (critical!)
3. Study **[runes-derived-effect.md](resources/runes-derived-effect.md)** - Computed values & effects
4. Learn **[template-control-flow.md](resources/template-control-flow.md)** - Build templates
5. Review **[directives-bind-use.md](resources/directives-bind-use.md)** - Add interactivity

### For Svelte 4 Users
1. Read **[migration-svelte5-1.md](resources/migration-svelte5-1.md)** - Understand runes migration
2. Read **[migration-svelte5-2.md](resources/migration-svelte5-2.md)** - Component changes
3. Focus on all three **runes-*.md** files - New reactivity paradigm
4. Check **[faqs-1.md](resources/faqs-1.md)** & **[faqs-2.md](resources/faqs-2.md)** - Common questions

### For Specific Tasks
- **State Management** → [runes-state.md](resources/runes-state.md) ($state)
- **Computed Values** → [runes-derived-effect.md](resources/runes-derived-effect.md) ($derived)
- **Component Props** → [runes-props-utilities.md](resources/runes-props-utilities.md) ($props, $bindable)
- **Side Effects** → [runes-derived-effect.md](resources/runes-derived-effect.md) ($effect)
- **Forms & Inputs** → [directives-bind-use.md](resources/directives-bind-use.md) (bind:)
- **Animations** → [directives-animations.md](resources/directives-animations.md) (transition:, animate:)
- **SEO & Metadata** → [special-elements.md](resources/special-elements.md) (svelte:head)
- **Global State** → [advanced-concepts-1.md](resources/advanced-concepts-1.md) (stores, context)
- **Lifecycle** → [advanced-concepts-2.md](resources/advanced-concepts-2.md) (onMount, etc.)
- **Type Safety** → [typescript.md](resources/typescript.md)
- **Troubleshooting** → [faqs-1.md](resources/faqs-1.md), [compiler-errors-1.md](resources/compiler-errors-1.md)

## Svelte 5 Key Differences from Svelte 4

**If you're coming from Svelte 4, these are the critical changes:**

| Svelte 4 | Svelte 5 | Resource |
|----------|----------|----------|
| `let count = 0` | `let count = $state(0)` | [runes.md](resources/runes.md) |
| `$: doubled = count * 2` | `let doubled = $derived(count * 2)` | [runes.md](resources/runes.md) |
| `$: { console.log(count) }` | `$effect(() => { console.log(count) })` | [runes.md](resources/runes.md) |
| `export let value` | `let { value } = $props()` | [runes.md](resources/runes.md) |
| `export let value` (two-way) | `let { value = $bindable() } = $props()` | [runes.md](resources/runes.md) |
| `<script context="module">` | `<script module>` | [components.md](resources/components.md) |
| `createEventDispatcher()` | Regular callbacks in props | [runes.md](resources/runes.md) |
| `$$props` / `$$restProps` | Rest destructuring: `let { ...rest } = $props()` | [runes.md](resources/runes.md) |

**Critical:** Svelte 5 uses **runes** (`$state`, `$derived`, `$effect`) for all reactivity. Reactive statements (`$:`) are legacy syntax.

## Common Development Workflows

### Creating a New Component

1. Create `.svelte` file with three sections:
   ```svelte
   <script>
     // Component logic
   </script>

   <!-- Template markup -->

   <style>
     /* Scoped styles */
   </style>
   ```

2. Add reactive state with `$state()`
3. Create computed values with `$derived()`
4. Handle side effects with `$effect()`
5. Accept props with `$props()`

See **[components.md](resources/components.md)** for details.

### Implementing Reactivity

**Simple state:**
```svelte
<script>
  let count = $state(0);
</script>
```

**Derived values:**
```svelte
<script>
  let count = $state(0);
  let doubled = $derived(count * 2);
</script>
```

**Effects:**
```svelte
<script>
  let count = $state(0);

  $effect(() => {
    console.log(`Count is now ${count}`);
  });
</script>
```

See **[runes.md](resources/runes.md)** for comprehensive reactivity patterns.

### Building Forms

```svelte
<script>
  let name = $state('');
  let email = $state('');

  function handleSubmit() {
    console.log({ name, email });
  }
</script>

<form onsubmit={handleSubmit}>
  <input bind:value={name} placeholder="Name" />
  <input bind:value={email} type="email" placeholder="Email" />
  <button>Submit</button>
</form>
```

See **[directives.md](resources/directives.md)** for all binding options.

### Conditional Rendering

```svelte
{#if condition}
  <p>Condition is true</p>
{:else if otherCondition}
  <p>Other condition is true</p>
{:else}
  <p>All conditions are false</p>
{/if}
```

See **[template-syntax.md](resources/template-syntax.md)** for all control flow patterns.

### List Rendering

```svelte
<script>
  let items = $state([
    { id: 1, name: 'Item 1' },
    { id: 2, name: 'Item 2' }
  ]);
</script>

{#each items as item (item.id)}
  <div>{item.name}</div>
{/each}
```

See **[template-syntax.md](resources/template-syntax.md)** for iteration patterns.

## Best Practices

### State Management
- ✅ Use `$state()` for mutable reactive values
- ✅ Use `$derived()` for computed values (not `$effect`)
- ✅ Keep effects minimal and focused
- ❌ Don't update state inside `$effect()` (use `$derived()` instead)
- ❌ Don't destructure reactive state (it breaks reactivity)

### Component Design
- ✅ Keep components small and focused
- ✅ Use props for parent → child communication
- ✅ Use callbacks or events for child → parent communication
- ✅ Use `$bindable()` sparingly for two-way binding
- ❌ Don't mutate props (unless marked `$bindable`)
- ❌ Don't over-use context API (prefer explicit props)

### Performance
- ✅ Use keyed `{#each}` blocks for dynamic lists
- ✅ Memoize expensive computations with `$derived()`
- ✅ Use `$state.raw()` for large non-reactive data
- ✅ Lazy-load components when appropriate
- ❌ Don't create unnecessary derived values
- ❌ Don't use effects where derived values would work

### Styling
- ✅ Leverage scoped styles by default
- ✅ Use CSS custom properties for theming
- ✅ Use `class:` directive for conditional classes
- ❌ Don't use global styles unless absolutely necessary
- ❌ Don't inline complex styles (use classes)

See **[common-patterns.md](resources/common-patterns.md)** for detailed best practices.

## Error Handling & Debugging

### Common Issues

**"State is not updating"**
- Check that you're using `$state()`, not plain variables
- Ensure you're not destructuring reactive state
- See [common-patterns.md](resources/common-patterns.md)

**"Effect running infinitely"**
- Check for state updates inside the effect
- Use `$derived()` instead for computed values
- See [runes.md](resources/runes.md)

**"Props not working"**
- Ensure you're using `$props()` destructuring
- Check prop names match in parent and child
- See [runes.md](resources/runes.md)

### Debugging Tools
- `$inspect()` - Log reactive state changes
- `$inspect.trace()` - Trace effect dependencies
- `{@debug}` - Breakpoints in templates
- Browser DevTools - Standard debugging

See **[common-patterns.md](resources/common-patterns.md)** for troubleshooting guide.

## TypeScript Support

Svelte has excellent TypeScript support. Add type safety to your components:

```svelte
<script lang="ts">
  interface Props {
    count: number;
    onIncrement: () => void;
  }

  let { count, onIncrement }: Props = $props();
</script>
```

See **[typescript.md](resources/typescript.md)** for comprehensive typing patterns.

## SvelteKit Integration

This skill focuses on Svelte components. For SvelteKit-specific features (routing, server-side rendering, API routes, etc.), refer to the SvelteKit documentation or use a dedicated SvelteKit skill.

**Svelte vs SvelteKit:**
- **Svelte** = Component framework (this skill)
- **SvelteKit** = Application framework (routing, SSR, API, deployment)

## Getting Help

**Official Resources:**
- [Interactive Tutorial](https://svelte.dev/tutorial)
- [Playground](https://svelte.dev/playground)
- [Discord Community](https://svelte.dev/chat)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/svelte)

**Within This Skill:**
- Start with the appropriate resource file from the table above
- Check [common-patterns.md](resources/common-patterns.md) for FAQs
- Review [migration-guide.md](resources/migration-guide.md) if upgrading

## Skill Maintenance

**Version:** Svelte 5.x (latest stable)
**Last Updated:** Extracted from official Svelte documentation
**Compatibility:** This skill covers Svelte 5. For Svelte 4 projects, see [migration-guide.md](resources/migration-guide.md)

---

## Progressive Disclosure Strategy

This skill follows the **500-line rule** for optimal context management:

1. **SKILL.md** (this file) - High-level overview and navigation (~490 lines)
2. **Resource files** - Deep dives into specific topics (<500 lines each)

**Load resource files only when needed:**
- General guidance → Stay in SKILL.md
- Specific implementation → Load the relevant resource file
- Deep troubleshooting → Load common-patterns.md

This keeps your context clean and focused on the task at hand.

---

**Ready to start?** Pick a resource file from the Quick Reference table above, or ask specific questions about Svelte development!

<div tabindex="1"></div>
```

### a11y_role_has_required_aria_props

```
Elements with the ARIA role "%role%" must have the following attributes defined: %props%
```

Elements with ARIA roles must have all required attributes for that role.

```svelte
<!-- A11y: A11y: Elements with the ARIA role "checkbox" must have the following attributes defined: "aria-checked" -->
<span role="checkbox" aria-labelledby="foo" tabindex="0"></span>
```

### a11y_role_supports_aria_props

```
The attribute '%attribute%' is not supported by the role '%role%'
```

Elements with explicit or implicit roles defined contain only `aria-*` properties supported by that role.

```svelte
<!-- A11y: The attribute 'aria-multiline' is not supported by the role 'link'. -->
<div role="link" aria-multiline></div>

<!-- A11y: The attribute 'aria-required' is not supported by the role 'listitem'. This role is implicit on the element <li>. -->
<li aria-required></li>
```

### a11y_role_supports_aria_props_implicit

```
The attribute '%attribute%' is not supported by the role '%role%'. This role is implicit on the element `<%name%>`
```

Elements with explicit or implicit roles defined contain only `aria-*` properties supported by that role.

```svelte
<!-- A11y: The attribute 'aria-multiline' is not supported by the role 'link'. -->
<div role="link" aria-multiline></div>

<!-- A11y: The attribute 'aria-required' is not supported by the role 'listitem'. This role is implicit on the element <li>. -->
<li aria-required></li>
```

### a11y_unknown_aria_attribute

```
Unknown aria attribute 'aria-%attribute%'
```

```
Unknown aria attribute 'aria-%attribute%'. Did you mean '%suggestion%'?
```

Enforce that only known ARIA attributes are used. This is based on the [WAI-ARIA States and Properties spec](https://www.w3.org/WAI/PF/aria-1.1/states_and_properties).

```svelte
<!-- A11y: Unknown aria attribute 'aria-labeledby' (did you mean 'labelledby'?) -->
<input type="image" aria-labeledby="foo" />
```

### a11y_unknown_role

```
Unknown role '%role%'
```

```
Unknown role '%role%'. Did you mean '%suggestion%'?
```

Elements with ARIA roles must use a valid, non-abstract ARIA role. A reference to role definitions can be found at [WAI-ARIA](https://www.w3.org/TR/wai-aria/#role_definitions) site.

<!-- prettier-ignore -->
```svelte
<!-- A11y: Unknown role 'toooltip' (did you mean 'tooltip'?) -->
<div role="toooltip"></div>
```

### attribute_avoid_is

```
The "is" attribute is not supported cross-browser and should be avoided
```

### attribute_global_event_reference

```
You are referencing `globalThis.%name%`. Did you forget to declare a variable with that name?
```

### attribute_illegal_colon

```
Attributes should not contain ':' characters to prevent ambiguity with Svelte directives
```

### attribute_invalid_property_name

```
'%wrong%' is not a valid HTML attribute. Did you mean '%right%'?
```

### attribute_quoted

```
Quoted attributes on components and custom elements will be stringified in a future version of Svelte. If this isn't what you want, remove the quotes
```

### bidirectional_control_characters

```
A bidirectional control character was detected in your code. These characters can be used to alter the visual direction of your code and could have unintended consequences
```

Bidirectional control characters can alter the direction in which text appears to be in. For example, via control characters, you can make `defabc` look like `abcdef`. As a result, if you were to unknowingly copy and paste some code that has these control characters, they may alter the behavior of your code in ways you did not intend. See [trojansource.codes](https://trojansource.codes/) for more information.

### bind_invalid_each_rest

```
The rest operator (...) will create a new object and binding '%name%' with the original object will not work
```

### block_empty

```
Empty block
```

### component_name_lowercase

```
`<%name%>` will be treated as an HTML element unless it begins with a capital letter
```

### css_unused_selector

```
Unused CSS selector "%name%"
```

Svelte traverses both the template and the `<style>` tag to find out which of the CSS selectors are not used within the template, so it can remove them.

In some situations a selector may target an element that is not 'visible' to the compiler, for example because it is part of an `{@html ...}` tag or you're overriding styles in a child component. In these cases, use [`:global`](/docs/svelte/global-styles) to preserve the selector as-is:

```svelte
<div class="post">{@html content}</div>

<style>
  .post :global {
    p {...}
  }
</style>
```

### custom_element_props_identifier

```
Using a rest element or a non-destructured declaration with `$props()` means that Svelte can't infer what properties to expose when creating a custom element. Consider destructuring all the props or explicitly specifying the `customElement.props` option.
```

### element_implicitly_closed

```
This element is implicitly closed by the following `%tag%`, which can cause an unexpected DOM structure. Add an explicit `%closing%` to avoid surprises.
```

In HTML, some elements are implicitly closed by another element. For example, you cannot nest a `<p>` inside another `<p>`:

```html
<!-- this HTML... -->
<p><p>hello</p>

<!-- results in this DOM structure -->
<p></p>
<p>hello</p>
```

Similarly, a parent element's closing tag will implicitly close all child elements, even if the `</` was a typo and you meant to create a _new_ element. To avoid ambiguity, it's always a good idea to have an explicit closing tag.

### element_invalid_self_closing_tag

```
Self-closing HTML tags for non-void elements are ambiguous — use `<%name% ...></%name%>` rather than `<%name% ... />`
```

In HTML, there's [no such thing as a self-closing tag](https://jakearchibald.com/2023/against-self-closing-tags-in-html/). While this _looks_ like a self-contained element with some text next to it...

```html
<div>
	<span class="icon" /> some text!
</div>
```

...a spec-compliant HTML parser (such as a browser) will in fact parse it like this, with the text _inside_ the icon:

```html
<div>
	<span class="icon"> some text! </span>
</div>
```

Some templating languages (including Svelte) will 'fix' HTML by turning `<span />` into `<span></span>`. Others adhere to the spec. Both result in ambiguity and confusion when copy-pasting code between different contexts, so Svelte prompts you to resolve the ambiguity directly by having an explicit closing tag.

To automate this, run the dedicated migration:

```sh
npx sv migrate self-closing-tags
```

In a future version of Svelte, self-closing tags may be upgraded from a warning to an error.

### event_directive_deprecated

```
Using `on:%name%` to listen to the %name% event is deprecated. Use the event attribute `on%name%` instead
```

See [the migration guide](v5-migration-guide#Event-changes) for more info.

### export_let_unused

```
Component has unused export property '%name%'. If it is for external reference only, please consider using `export const %name%`
```

### legacy_code

```
`%code%` is no longer valid — please use `%suggestion%` instead
```

### legacy_component_creation

```
Svelte 5 components are no longer classes. Instantiate them using `mount` or `hydrate` (imported from 'svelte') instead.
```

See the [migration guide](v5-migration-guide#Components-are-no-longer-classes) for more info.

### node_invalid_placement_ssr

```
%message%. When rendering this component on the server, the resulting HTML will be modified by the browser (by moving, removing, or inserting elements), likely resulting in a `hydration_mismatch` warning
```

HTML restricts where certain elements can appear. In case of a violation the browser will 'repair' the HTML in a way that breaks Svelte's assumptions about the structure of your components. Some examples:

- `<p>hello <div>world</div></p>` will result in `<p>hello </p><div>world</div><p></p>` (the `<div>` autoclosed the `<p>` because `<p>` cannot contain block-level elements)
- `<option><div>option a</div></option>` will result in `<option>option a</option>` (the `<div>` is removed)
- `<table><tr><td>cell</td></tr></table>` will result in `<table><tbody><tr><td>cell</td></tr></tbody></table>` (a `<tbody>` is auto-inserted)

This code will work when the component is rendered on the client (which is why this is a warning rather than an error), but if you use server rendering it will cause hydration to fail.

### non_reactive_update

```
`%name%` is updated, but is not declared with `$state(...)`. Changing its value will not correctly trigger updates
```

This warning is thrown when the compiler detects the following:
- a variable was declared without `$state` or `$state.raw`
- the variable is reassigned
- the variable is read in a reactive context

In this case, changing the value will not correctly trigger updates. Example:

```svelte
<script>
	let reactive = $state('reactive');
	let stale = 'stale';
</script>

<p>This value updates: {reactive}</p>
<p>This value does not update: {stale}</p>

<button onclick={() => {
	stale = 'updated';
	reactive = 'updated';
}}>update</button>
```

To fix this, wrap your variable declaration with `$state`.

### options_deprecated_accessors

```
The `accessors` option has been deprecated. It will have no effect in runes mode
```

### options_deprecated_immutable

```
The `immutable` option has been deprecated. It will have no effect in runes mode
```

### options_missing_custom_element

```
The `customElement` option is used when generating a custom element. Did you forget the `customElement: true` compile option?
```

### options_removed_enable_sourcemap

```
The `enableSourcemap` option has been removed. Source maps are always generated now, and tooling can choose to ignore them
```

### options_removed_hydratable

```
The `hydratable` option has been removed. Svelte components are always hydratable now
```

### options_removed_loop_guard_timeout

```
The `loopGuardTimeout` option has been removed
```

### options_renamed_ssr_dom

```
`generate: "dom"` and `generate: "ssr"` options have been renamed to "client" and "server" respectively
```

### perf_avoid_inline_class

```
Avoid 'new class' — instead, declare the class at the top level scope
```

### perf_avoid_nested_class

```
Avoid declaring classes below the top level scope
```

### reactive_declaration_invalid_placement

```
Reactive declarations only exist at the top level of the instance script
```

### reactive_declaration_module_script_dependency

```
Reassignments of module-level declarations will not cause reactive statements to update
```

### script_context_deprecated

```
`context="module"` is deprecated, use the `module` attribute instead
```

```svelte
<script ---context="module"--- +++module+++>
	let foo = 'bar';
</script>
```

### script_unknown_attribute

```
Unrecognized attribute — should be one of `generics`, `lang` or `module`. If this exists for a preprocessor, ensure that the preprocessor removes it
```

### slot_element_deprecated

```
Using `<slot>` to render parent content is deprecated. Use `{@render ...}` tags instead
```

See [the migration guide](v5-migration-guide#Snippets-instead-of-slots) for more info.

### state_referenced_locally

```
This reference only captures the initial value of `%name%`. Did you mean to reference it inside a %type% instead?
```

This warning is thrown when the compiler detects the following:

- A reactive variable is declared
- ...and later reassigned...
- ...and referenced in the same scope

This 'breaks the link' to the original state declaration. For example, if you pass the state to a function, the function loses access to the state once it is reassigned:

```svelte
<!--- file: Parent.svelte --->
<script>
	import { setContext } from 'svelte';

	let count = $state(0);

	// warning: state_referenced_locally
	setContext('count', count);
</script>

<button onclick={() => count++}>
	increment
</button>
```

```svelte
<!--- file: Child.svelte --->
<script>
	import { getContext } from 'svelte';

	const count = getContext('count');
</script>

<!-- This will never update -->
<p>The count is {count}</p>
```

To fix this, reference the variable such that it is lazily evaluated. For the above example, this can be achieved by wrapping `count` in a function:

```svelte
<!--- file: Parent.svelte --->
<script>
	import { setContext } from 'svelte';

	let count = $state(0);
	setContext('count', +++() => count+++);
</script>

<button onclick={() => count++}>
	increment
</button>
```

```svelte
<!--- file: Child.svelte --->
<script>
	import { getContext } from 'svelte';

	const count = getContext('count');
</script>

<!-- This will update -->
<p>The count is {+++count()+++}</p>
```

For more info, see [Passing state into functions]($state#Passing-state-into-functions).

### store_rune_conflict

```
It looks like you're using the `$%name%` rune, but there is a local binding called `%name%`. Referencing a local variable with a `$` prefix will create a store subscription. Please rename `%name%` to avoid the ambiguity
```

### svelte_component_deprecated

```
`<svelte:component>` is deprecated in runes mode — components are dynamic by default
```

In previous versions of Svelte, the component constructor was fixed when the component was rendered. In other words, if you wanted `<X>` to re-render when `X` changed, you would either have to use `<svelte:component this={X}>` or put the component inside a `{#key X}...{/key}` block.

In Svelte 5 this is no longer true — if `X` changes, `<X>` re-renders.

In some cases `<object.property>` syntax can be used as a replacement; a lowercased variable with property access is recognized as a component in Svelte 5.

For complex component resolution logic, an intermediary, capitalized variable may be necessary. E.g. in places where `@const` can be used:

<!-- prettier-ignore -->
```svelte
{#each items as item}
	---<svelte:component this={item.condition ? Y : Z} />---
	+++{@const Component = item.condition ? Y : Z}+++
	+++<Component />+++
{/each}
```

A derived value may be used in other contexts:

<!-- prettier-ignore -->
```svelte
<script>
	// ...
	let condition = $state(false);
	+++const Component = $derived(condition ? Y : Z);+++
</script>

---<svelte:component this={condition ? Y : Z} />---
+++<Component />+++
```

### svelte_element_invalid_this

```
`this` should be an `{expression}`. Using a string attribute value will cause an error in future versions of Svelte
```

### svelte_self_deprecated

```
`<svelte:self>` is deprecated — use self-imports (e.g. `import %name% from './%basename%'`) instead
```

See [the note in the docs](legacy-svelte-self) for more info.

### unknown_code

```
`%code%` is not a recognised code
```

```
`%code%` is not a recognised code (did you mean `%suggestion%`?)
```


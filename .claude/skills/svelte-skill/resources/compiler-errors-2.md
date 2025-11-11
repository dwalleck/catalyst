The '%modifier1%' and '%modifier2%' modifiers cannot be used together
```

### expected_attribute_value

```
Expected attribute value
```

### expected_block_type

```
Expected 'if', 'each', 'await', 'key' or 'snippet'
```

### expected_identifier

```
Expected an identifier
```

### expected_pattern

```
Expected identifier or destructure pattern
```

### expected_token

```
Expected token %token%
```

### expected_whitespace

```
Expected whitespace
```

### experimental_async

```
Cannot use `await` in deriveds and template expressions, or at the top level of a component, unless the `experimental.async` compiler option is `true`
```

### export_undefined

```
`%name%` is not defined
```

### global_reference_invalid

```
`%name%` is an illegal variable name. To reference a global variable called `%name%`, use `globalThis.%name%`
```

### host_invalid_placement

```
`$host()` can only be used inside custom element component instances
```

### illegal_element_attribute

```
`<%name%>` does not support non-event attributes or spread attributes
```

### import_svelte_internal_forbidden

```
Imports of `svelte/internal/*` are forbidden. It contains private runtime code which is subject to change without notice. If you're importing from `svelte/internal/*` to work around a limitation of Svelte, please open an issue at https://github.com/sveltejs/svelte and explain your use case
```

### inspect_trace_generator

```
`$inspect.trace(...)` cannot be used inside a generator function
```

### inspect_trace_invalid_placement

```
`$inspect.trace(...)` must be the first statement of a function body
```

### invalid_arguments_usage

```
The arguments keyword cannot be used within the template or at the top level of a component
```

### js_parse_error

```
%message%
```

### legacy_await_invalid

```
Cannot use `await` in deriveds and template expressions, or at the top level of a component, unless in runes mode
```

### legacy_export_invalid

```
Cannot use `export let` in runes mode — use `$props()` instead
```

### legacy_props_invalid

```
Cannot use `$$props` in runes mode
```

### legacy_reactive_statement_invalid

```
`$:` is not allowed in runes mode, use `$derived` or `$effect` instead
```

### legacy_rest_props_invalid

```
Cannot use `$$restProps` in runes mode
```

### let_directive_invalid_placement

```
`let:` directive at invalid position
```

### mixed_event_handler_syntaxes

```
Mixing old (on:%name%) and new syntaxes for event handling is not allowed. Use only the on%name% syntax
```

### module_illegal_default_export

```
A component cannot have a default export
```

### node_invalid_placement

```
%message%. The browser will 'repair' the HTML (by moving, removing, or inserting elements) which breaks Svelte's assumptions about the structure of your components.
```

HTML restricts where certain elements can appear. In case of a violation the browser will 'repair' the HTML in a way that breaks Svelte's assumptions about the structure of your components. Some examples:

- `<p>hello <div>world</div></p>` will result in `<p>hello </p><div>world</div><p></p>` (the `<div>` autoclosed the `<p>` because `<p>` cannot contain block-level elements)
- `<option><div>option a</div></option>` will result in `<option>option a</option>` (the `<div>` is removed)
- `<table><tr><td>cell</td></tr></table>` will result in `<table><tbody><tr><td>cell</td></tr></tbody></table>` (a `<tbody>` is auto-inserted)

### options_invalid_value

```
Invalid compiler option: %details%
```

### options_removed

```
Invalid compiler option: %details%
```

### options_unrecognised

```
Unrecognised compiler option %keypath%
```

### props_duplicate

```
Cannot use `%rune%()` more than once
```

### props_id_invalid_placement

```
`$props.id()` can only be used at the top level of components as a variable declaration initializer
```

### props_illegal_name

```
Declaring or accessing a prop starting with `$$` is illegal (they are reserved for Svelte internals)
```

### props_invalid_identifier

```
`$props()` can only be used with an object destructuring pattern
```

### props_invalid_pattern

```
`$props()` assignment must not contain nested properties or computed keys
```

### props_invalid_placement

```
`$props()` can only be used at the top level of components as a variable declaration initializer
```

### reactive_declaration_cycle

```
Cyclical dependency detected: %cycle%
```

### render_tag_invalid_call_expression

```
Calling a snippet function using apply, bind or call is not allowed
```

### render_tag_invalid_expression

```
`{@render ...}` tags can only contain call expressions
```

### render_tag_invalid_spread_argument

```
cannot use spread arguments in `{@render ...}` tags
```

### rune_invalid_arguments

```
`%rune%` cannot be called with arguments
```

### rune_invalid_arguments_length

```
`%rune%` must be called with %args%
```

### rune_invalid_computed_property

```
Cannot access a computed property of a rune
```

### rune_invalid_name

```
`%name%` is not a valid rune
```

### rune_invalid_spread

```
`%rune%` cannot be called with a spread argument
```

### rune_invalid_usage

```
Cannot use `%rune%` rune in non-runes mode
```

### rune_missing_parentheses

```
Cannot use rune without parentheses
```

### rune_removed

```
The `%name%` rune has been removed
```

### rune_renamed

```
`%name%` is now `%replacement%`
```

### runes_mode_invalid_import

```
%name% cannot be used in runes mode
```

### script_duplicate

```
A component can have a single top-level `<script>` element and/or a single top-level `<script module>` element
```

### script_invalid_attribute_value

```
If the `%name%` attribute is supplied, it must be a boolean attribute
```

### script_invalid_context

```
If the context attribute is supplied, its value must be "module"
```

### script_reserved_attribute

```
The `%name%` attribute is reserved and cannot be used
```

### slot_attribute_duplicate

```
Duplicate slot name '%name%' in <%component%>
```

### slot_attribute_invalid

```
slot attribute must be a static value
```

### slot_attribute_invalid_placement

```
Element with a slot='...' attribute must be a child of a component or a descendant of a custom element
```

### slot_default_duplicate

```
Found default slot content alongside an explicit slot="default"
```

### slot_element_invalid_attribute

```
`<slot>` can only receive attributes and (optionally) let directives
```

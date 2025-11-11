# Basic markup

Markup inside a Svelte component can be thought of as HTML++.

## Tags

A lowercase tag, like `<div>`, denotes a regular HTML element. A capitalised tag or a tag that uses dot notation, such as `<Widget>` or `<my.stuff>`, indicates a _component_.

```svelte
<script>
	import Widget from './Widget.svelte';
</script>

<div>
	<Widget />
</div>
```

## Element attributes

By default, attributes work exactly like their HTML counterparts.

```svelte
<div class="foo">
	<button disabled>can't touch this</button>
</div>
```

As in HTML, values may be unquoted.

<!-- prettier-ignore -->
```svelte
<input type=checkbox />
```

Attribute values can contain JavaScript expressions.

```svelte
<a href="page/{p}">page {p}</a>
```

Or they can _be_ JavaScript expressions.

```svelte
<button disabled={!clickable}>...</button>
```

Boolean attributes are included on the element if their value is [truthy](https://developer.mozilla.org/en-US/docs/Glossary/Truthy) and excluded if it's [falsy](https://developer.mozilla.org/en-US/docs/Glossary/Falsy).

All other attributes are included unless their value is [nullish](https://developer.mozilla.org/en-US/docs/Glossary/Nullish) (`null` or `undefined`).

```svelte
<input required={false} placeholder="This input field is not required" />
<div title={null}>This div has no title attribute</div>
```

> [!NOTE] Quoting a singular expression does not affect how the value is parsed, but in Svelte 6 it will cause the value to be coerced to a string:
>
> <!-- prettier-ignore -->
> ```svelte
> <button disabled="{number !== 42}">...</button>
> ```

When the attribute name and value match (`name={name}`), they can be replaced with `{name}`.

```svelte
<button {disabled}>...</button>
<!-- equivalent to
<button disabled={disabled}>...</button>
-->
```

## Component props

By convention, values passed to components are referred to as _properties_ or _props_ rather than _attributes_, which are a feature of the DOM.

As with elements, `name={name}` can be replaced with the `{name}` shorthand.

```svelte
<Widget foo={bar} answer={42} text="hello" />
```

## Spread attributes

_Spread attributes_ allow many attributes or properties to be passed to an element or component at once.

An element or component can have multiple spread attributes, interspersed with regular ones. Order matters — if `things.a` exists it will take precedence over `a="b"`, while `c="d"` would take precedence over `things.c`:

```svelte
<Widget a="b" {...things} c="d" />
```

## Events

Listening to DOM events is possible by adding attributes to the element that start with `on`. For example, to listen to the `click` event, add the `onclick` attribute to a button:

```svelte
<button onclick={() => console.log('clicked')}>click me</button>
```

Event attributes are case sensitive. `onclick` listens to the `click` event, `onClick` listens to the `Click` event, which is different. This ensures you can listen to custom events that have uppercase characters in them.

Because events are just attributes, the same rules as for attributes apply:

- you can use the shorthand form: `<button {onclick}>click me</button>`
- you can spread them: `<button {...thisSpreadContainsEventAttributes}>click me</button>`

Timing-wise, event attributes always fire after events from bindings (e.g. `oninput` always fires after an update to `bind:value`). Under the hood, some event handlers are attached directly with `addEventListener`, while others are _delegated_.

When using `ontouchstart` and `ontouchmove` event attributes, the handlers are [passive](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#using_passive_listeners) for better performance. This greatly improves responsiveness by allowing the browser to scroll the document immediately, rather than waiting to see if the event handler calls `event.preventDefault()`.

In the very rare cases that you need to prevent these event defaults, you should use [`on`](svelte-events#on) instead (for example inside an action).

### Event delegation

To reduce memory footprint and increase performance, Svelte uses a technique called event delegation. This means that for certain events — see the list below — a single event listener at the application root takes responsibility for running any handlers on the event's path.

There are a few gotchas to be aware of:

- when you manually dispatch an event with a delegated listener, make sure to set the `{ bubbles: true }` option or it won't reach the application root
- when using `addEventListener` directly, avoid calling `stopPropagation` or the event won't reach the application root and handlers won't be invoked. Similarly, handlers added manually inside the application root will run _before_ handlers added declaratively deeper in the DOM (with e.g. `onclick={...}`), in both capturing and bubbling phases. For these reasons it's better to use the `on` function imported from `svelte/events` rather than `addEventListener`, as it will ensure that order is preserved and `stopPropagation` is handled correctly.

The following event handlers are delegated:

- `beforeinput`
- `click`
- `change`
- `dblclick`
- `contextmenu`
- `focusin`
- `focusout`
- `input`
- `keydown`
- `keyup`
- `mousedown`
- `mousemove`
- `mouseout`
- `mouseover`
- `mouseup`
- `pointerdown`
- `pointermove`
- `pointerout`
- `pointerover`
- `pointerup`
- `touchend`
- `touchmove`
- `touchstart`

## Text expressions

A JavaScript expression can be included as text by surrounding it with curly braces.

```svelte
{expression}
```

Expressions that are `null` or `undefined` will be omitted; all others are [coerced to strings](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String#string_coercion).

Curly braces can be included in a Svelte template by using their [HTML entity](https://developer.mozilla.org/docs/Glossary/Entity) strings: `&lbrace;`, `&lcub;`, or `&#123;` for `{` and `&rbrace;`, `&rcub;`, or `&#125;` for `}`.

If you're using a regular expression (`RegExp`) [literal notation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp#literal_notation_and_constructor), you'll need to wrap it in parentheses.

<!-- prettier-ignore -->
```svelte
<h1>Hello {name}!</h1>
<p>{a} + {b} = {a + b}.</p>

<div>{(/^[A-Za-z ]+$/).test(value) ? x : y}</div>
```

The expression will be stringified and escaped to prevent code injections. If you want to render HTML, use the `{@html}` tag instead.

```svelte
{@html potentiallyUnsafeHtmlString}
```

> [!NOTE] Make sure that you either escape the passed string or only populate it with values that are under your control in order to prevent [XSS attacks](https://owasp.org/www-community/attacks/xss/)

## Comments

You can use HTML comments inside components.

```svelte
<!-- this is a comment! --><h1>Hello world</h1>
```

Comments beginning with `svelte-ignore` disable warnings for the next block of markup. Usually, these are accessibility warnings; make sure that you're disabling them for a good reason.

```svelte
<!-- svelte-ignore a11y_autofocus -->
<input bind:value={name} autofocus />
```

You can add a special comment starting with `@component` that will show up when hovering over the component name in other files.

````svelte
<!--
@component
- You can use markdown here.
- You can also use code blocks here.
- Usage:
  ```html
  <Main name="Arethra">
  ```
-->
<script>
	let { name } = $props();
</script>

<main>
	<h1>
		Hello, {name}
	</h1>
</main>
````

# {#if ...}

```svelte
<!--- copy: false  --->
{#if expression}...{/if}
```

```svelte
<!--- copy: false  --->
{#if expression}...{:else if expression}...{/if}
```

```svelte
<!--- copy: false  --->
{#if expression}...{:else}...{/if}
```

Content that is conditionally rendered can be wrapped in an if block.

```svelte
{#if answer === 42}
	<p>what was the question?</p>
{/if}
```

Additional conditions can be added with `{:else if expression}`, optionally ending in an `{:else}` clause.

```svelte
{#if porridge.temperature > 100}
	<p>too hot!</p>
{:else if 80 > porridge.temperature}
	<p>too cold!</p>
{:else}
	<p>just right!</p>
{/if}
```

(Blocks don't have to wrap elements, they can also wrap text within elements.)

# {#each ...}

```svelte
<!--- copy: false  --->
{#each expression as name}...{/each}
```

```svelte
<!--- copy: false  --->
{#each expression as name, index}...{/each}
```

Iterating over values can be done with an each block. The values in question can be arrays, array-like objects (i.e. anything with a `length` property), or iterables like `Map` and `Set` — in other words, anything that can be used with `Array.from`.

```svelte
<h1>Shopping list</h1>
<ul>
	{#each items as item}
		<li>{item.name} x {item.qty}</li>
	{/each}
</ul>
```

An each block can also specify an _index_, equivalent to the second argument in an `array.map(...)` callback:

```svelte
{#each items as item, i}
	<li>{i + 1}: {item.name} x {item.qty}</li>
{/each}
```

## Keyed each blocks

```svelte
<!--- copy: false  --->
{#each expression as name (key)}...{/each}
```

```svelte
<!--- copy: false  --->
{#each expression as name, index (key)}...{/each}
```

If a _key_ expression is provided — which must uniquely identify each list item — Svelte will use it to intelligently update the list when data changes by inserting, moving and deleting items, rather than adding or removing items at the end and updating the state in the middle.

The key can be any object, but strings and numbers are recommended since they allow identity to persist when the objects themselves change.

```svelte
{#each items as item (item.id)}
	<li>{item.name} x {item.qty}</li>
{/each}

<!-- or with additional index value -->
{#each items as item, i (item.id)}
	<li>{i + 1}: {item.name} x {item.qty}</li>
{/each}
```

You can freely use destructuring and rest patterns in each blocks.

```svelte
{#each items as { id, name, qty }, i (id)}
	<li>{i + 1}: {name} x {qty}</li>
{/each}

{#each objects as { id, ...rest }}
	<li><span>{id}</span><MyComponent {...rest} /></li>
{/each}

{#each items as [id, ...rest]}
	<li><span>{id}</span><MyComponent values={rest} /></li>
{/each}
```

## Each blocks without an item

```svelte
<!--- copy: false  --->
{#each expression}...{/each}
```

```svelte
<!--- copy: false  --->
{#each expression, index}...{/each}
```

In case you just want to render something `n` times, you can omit the `as` part ([demo](/playground/untitled#H4sIAAAAAAAAE3WR0W7CMAxFf8XKNAk0WsSeUEaRpn3Guoc0MbQiJFHiMlDVf18SOrZJ48259_jaVgZmxBEZZ28thgCNFV6xBdt1GgPj7wOji0t2EqI-wa_OleGEmpLWiID_6dIaQkMxhm1UdwKpRQhVzWSaVORJNdvWpqbhAYVsYQCNZk8thzWMC_DCHMZk3wPSThNQ088I3mghD9UwSwHwlLE5PMIzVFUFq3G7WUZ2OyUvU3JOuZU332wCXTRmtPy1NgzXZtUFp8WFw9536uWqpbIgPEaDsJBW90cTOHh0KGi2XsBq5-cT6-3nPauxXqHnsHJnCFZ3CvJVkyuCQ0mFF9TZyCQ162WGvteLKfG197Y3iv_pz_fmS68Hxt8iPBPj5HscP8YvCNX7uhYCAAA=)):

```svelte
<div class="chess-board">
	{#each { length: 8 }, rank}
		{#each { length: 8 }, file}
			<div class:black={(rank + file) % 2 === 1}></div>
		{/each}
	{/each}
</div>
```

## Else blocks

```svelte
<!--- copy: false  --->
{#each expression as name}...{:else}...{/each}
```

An each block can also have an `{:else}` clause, which is rendered if the list is empty.

```svelte
{#each todos as todo}
	<p>{todo.text}</p>
{:else}
	<p>No tasks today!</p>
{/each}
```

# {#key ...}

```svelte
<!--- copy: false  --->
{#key expression}...{/key}
```

Key blocks destroy and recreate their contents when the value of an expression changes. When used around components, this will cause them to be reinstantiated and reinitialised:

```svelte
{#key value}
	<Component />
{/key}
```

It's also useful if you want a transition to play whenever a value changes:

```svelte
{#key value}
	<div transition:fade>{value}</div>
{/key}
```

# {#await ...}

```svelte
<!--- copy: false  --->
{#await expression}...{:then name}...{:catch name}...{/await}
```

```svelte
<!--- copy: false  --->
{#await expression}...{:then name}...{/await}
```

```svelte
<!--- copy: false  --->
{#await expression then name}...{/await}
```

```svelte
<!--- copy: false  --->
{#await expression catch name}...{/await}
```

Await blocks allow you to branch on the three possible states of a [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise) — pending, fulfilled or rejected.

```svelte
{#await promise}
	<!-- promise is pending -->
	<p>waiting for the promise to resolve...</p>
{:then value}
	<!-- promise was fulfilled or not a Promise -->
	<p>The value is {value}</p>
{:catch error}
	<!-- promise was rejected -->
	<p>Something went wrong: {error.message}</p>
{/await}
```

> [!NOTE] During server-side rendering, only the pending branch will be rendered.
>
> If the provided expression is not a `Promise`, only the `:then` branch will be rendered, including during server-side rendering.

The `catch` block can be omitted if you don't need to render anything when the promise rejects (or no error is possible).

```svelte
{#await promise}
	<!-- promise is pending -->
	<p>waiting for the promise to resolve...</p>
{:then value}
	<!-- promise was fulfilled -->
	<p>The value is {value}</p>
{/await}
```

If you don't care about the pending state, you can also omit the initial block.

```svelte
{#await promise then value}
	<p>The value is {value}</p>
{/await}
```

Similarly, if you only want to show the error state, you can omit the `then` block.

```svelte
{#await promise catch error}
	<p>The error is {error}</p>
{/await}
```

> [!NOTE] You can use `#await` with [`import(...)`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/import) to render components lazily:
>
> ```svelte
> {#await import('./Component.svelte') then { default: Component }}
> 	<Component />
> {/await}
> ```


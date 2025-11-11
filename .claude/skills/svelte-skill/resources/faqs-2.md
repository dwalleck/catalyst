
<div class="ts-block">

```dts
function fork(fn: () => void): Fork;
```

</div>



## getAbortSignal

Returns an [`AbortSignal`](https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal) that aborts when the current [derived](/docs/svelte/$derived) or [effect](/docs/svelte/$effect) re-runs or is destroyed.

Must be called while a derived or effect is running.

```svelte
<script>
	import { getAbortSignal } from 'svelte';

	let { id } = $props();

	async function getData(id) {
		const response = await fetch(`/items/${id}`, {
			signal: getAbortSignal()
		});

		return await response.json();
	}

	const data = $derived(await getData(id));
</script>
```

<div class="ts-block">

```dts
function getAbortSignal(): AbortSignal;
```

</div>



## getAllContexts

Retrieves the whole context map that belongs to the closest parent component.
Must be called during component initialisation. Useful, for example, if you
programmatically create a component and want to pass the existing context to it.

<div class="ts-block">

```dts
function getAllContexts<
	T extends Map<any, any> = Map<any, any>
>(): T;
```

</div>



## getContext

Retrieves the context that belongs to the closest parent component with the specified `key`.
Must be called during component initialisation.

[`createContext`](/docs/svelte/svelte#createContext) is a type-safe alternative.

<div class="ts-block">

```dts
function getContext<T>(key: any): T;
```

</div>



## hasContext

Checks whether a given `key` has been set in the context of a parent component.
Must be called during component initialisation.

<div class="ts-block">

```dts
function hasContext(key: any): boolean;
```

</div>



## hydrate

Hydrates a component on the given target and returns the exports and potentially the props (if compiled with `accessors: true`) of the component

<div class="ts-block">

```dts
function hydrate<
	Props extends Record<string, any>,
	Exports extends Record<string, any>
>(
	component:
		| ComponentType<SvelteComponent<Props>>
		| Component<Props, Exports, any>,
	options: {} extends Props
		? {
				target: Document | Element | ShadowRoot;
				props?: Props;
				events?: Record<string, (e: any) => any>;
				context?: Map<any, any>;
				intro?: boolean;
				recover?: boolean;
			}
		: {
				target: Document | Element | ShadowRoot;
				props: Props;
				events?: Record<string, (e: any) => any>;
				context?: Map<any, any>;
				intro?: boolean;
				recover?: boolean;
			}
): Exports;
```

</div>



## mount

Mounts a component to the given target and returns the exports and potentially the props (if compiled with `accessors: true`) of the component.
Transitions will play during the initial render unless the `intro` option is set to `false`.

<div class="ts-block">

```dts
function mount<
	Props extends Record<string, any>,
	Exports extends Record<string, any>
>(
	component:
		| ComponentType<SvelteComponent<Props>>
		| Component<Props, Exports, any>,
	options: MountOptions<Props>
): Exports;
```

</div>



## onDestroy

Schedules a callback to run immediately before the component is unmounted.

Out of `onMount`, `beforeUpdate`, `afterUpdate` and `onDestroy`, this is the
only one that runs inside a server-side component.

<div class="ts-block">

```dts
function onDestroy(fn: () => any): void;
```

</div>



## onMount

`onMount`, like [`$effect`](/docs/svelte/$effect), schedules a function to run as soon as the component has been mounted to the DOM.
Unlike `$effect`, the provided function only runs once.

It must be called during the component's initialisation (but doesn't need to live _inside_ the component;
it can be called from an external module). If a function is returned _synchronously_ from `onMount`,
it will be called when the component is unmounted.

`onMount` functions do not run during [server-side rendering](/docs/svelte/svelte-server#render).

<div class="ts-block">

```dts
function onMount<T>(
	fn: () =>
		| NotFunction<T>
		| Promise<NotFunction<T>>
		| (() => any)
): void;
```

</div>



## setContext

Associates an arbitrary `context` object with the current component and the specified `key`
and returns that object. The context is then available to children of the component
(including slotted content) with `getContext`.

Like lifecycle functions, this must be called during component initialisation.

[`createContext`](/docs/svelte/svelte#createContext) is a type-safe alternative.

<div class="ts-block">

```dts
function setContext<T>(key: any, context: T): T;
```

</div>



## settled

<blockquote class="since note">

Available since 5.36

</blockquote>

Returns a promise that resolves once any state changes, and asynchronous work resulting from them,
have resolved and the DOM has been updated

<div class="ts-block">

```dts
function settled(): Promise<void>;
```

</div>



## tick

Returns a promise that resolves once any pending state changes have been applied.

<div class="ts-block">

```dts
function tick(): Promise<void>;
```

</div>



## unmount

Unmounts a component that was previously mounted using `mount` or `hydrate`.

Since 5.13.0, if `options.outro` is `true`, [transitions](/docs/svelte/transition) will play before the component is removed from the DOM.

Returns a `Promise` that resolves after transitions have completed if `options.outro` is true, or immediately otherwise (prior to 5.13.0, returns `void`).

```js
// @errors: 7031
import { mount, unmount } from 'svelte';
import App from './App.svelte';

const app = mount(App, { target: document.body });

// later...
unmount(app, { outro: true });
```

<div class="ts-block">

```dts
function unmount(
	component: Record<string, any>,
	options?:
		| {
				outro?: boolean;
		  }
		| undefined
): Promise<void>;
```

</div>



## untrack

When used inside a [`$derived`](/docs/svelte/$derived) or [`$effect`](/docs/svelte/$effect),
any state read inside `fn` will not be treated as a dependency.

```ts
$effect(() => {
	// this will run when `data` changes, but not when `time` changes
	save(data, {
		timestamp: untrack(() => time)
	});
});
```

<div class="ts-block">

```dts
function untrack<T>(fn: () => T): T;
```

</div>



## Component

Can be used to create strongly typed Svelte components.

#### Example:

You have component library on npm called `component-library`, from which
you export a component called `MyComponent`. For Svelte+TypeScript users,
you want to provide typings. Therefore you create a `index.d.ts`:
```ts
import type { Component } from 'svelte';
export declare const MyComponent: Component<{ foo: string }> {}
```
Typing this makes it possible for IDEs like VS Code with the Svelte extension
to provide intellisense and to use the component like this in a Svelte file
with TypeScript:
```svelte
<script lang="ts">
	import { MyComponent } from "component-library";
</script>
<MyComponent foo={'bar'} />
```

<div class="ts-block">

```dts
interface Component<
	Props extends Record<string, any> = {},
	Exports extends Record<string, any> = {},
	Bindings extends keyof Props | '' = string
> {/*…*/}
```

<div class="ts-block-property">

```dts
(
	this: void,
	internals: ComponentInternals,
	props: Props
): {
	/**
	 * @deprecated This method only exists when using one of the legacy compatibility helpers, which
	 * is a stop-gap solution. See [migration guide](https://svelte.dev/docs/svelte/v5-migration-guide#Components-are-no-longer-classes)
	 * for more info.
	 */
	$on?(type: string, callback: (e: any) => void): () => void;
	/**
	 * @deprecated This method only exists when using one of the legacy compatibility helpers, which
	 * is a stop-gap solution. See [migration guide](https://svelte.dev/docs/svelte/v5-migration-guide#Components-are-no-longer-classes)
	 * for more info.
	 */
	$set?(props: Partial<Props>): void;
} & Exports;
```

<div class="ts-block-property-details">

<div class="ts-block-property-bullets">

- `internal` An internal object used by Svelte. Do not use or modify.
- `props` The props passed to the component.

</div>

</div>
</div>

<div class="ts-block-property">

```dts
element?: typeof HTMLElement;
```

<div class="ts-block-property-details">

The custom element version of the component. Only present if compiled with the `customElement` compiler option

</div>
</div></div>

## ComponentConstructorOptions

<blockquote class="tag deprecated note">

In Svelte 4, components are classes. In Svelte 5, they are functions.
Use `mount` instead to instantiate components.
See [migration guide](/docs/svelte/v5-migration-guide#Components-are-no-longer-classes)
for more info.

</blockquote>

<div class="ts-block">

```dts
interface ComponentConstructorOptions<
	Props extends Record<string, any> = Record<string, any>
> {/*…*/}
```

<div class="ts-block-property">

```dts
target: Element | Document | ShadowRoot;
```

<div class="ts-block-property-details"></div>
</div>

<div class="ts-block-property">

```dts
anchor?: Element;
```

<div class="ts-block-property-details"></div>
</div>

<div class="ts-block-property">

```dts
props?: Props;
```

<div class="ts-block-property-details"></div>
</div>

<div class="ts-block-property">

```dts
context?: Map<any, any>;
```

<div class="ts-block-property-details"></div>
</div>


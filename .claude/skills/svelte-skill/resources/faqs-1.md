# Frequently asked questions

## I'm new to Svelte. Where should I start?

We think the best way to get started is playing through the interactive [tutorial](/tutorial). Each step there is mainly focused on one specific aspect and is easy to follow. You'll be editing and running real Svelte components right in your browser.

Five to ten minutes should be enough to get you up and running. An hour and a half should get you through the entire tutorial.

## Where can I get support?

If your question is about certain syntax, the [reference docs](/docs/svelte) are a good place to start.

Stack Overflow is a popular forum to ask code-level questions or if you’re stuck with a specific error. Read through the existing questions tagged with [Svelte](https://stackoverflow.com/questions/tagged/svelte+or+svelte-3) or [ask your own](https://stackoverflow.com/questions/ask?tags=svelte)!

There are online forums and chats which are a great place for discussion about best practices, application architecture or just to get to know fellow Svelte users. [Our Discord](/chat) or [the Reddit channel](https://www.reddit.com/r/sveltejs/) are examples of that. If you have an answerable code-level question, Stack Overflow is usually a better fit.

## Are there any third-party resources?

Svelte Society maintains a [list of books and videos](https://sveltesociety.dev/resources).

## How can I get VS Code to syntax-highlight my .svelte files?

There is an [official VS Code extension for Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode).

## Is there a tool to automatically format my .svelte files?

You can use prettier with the [prettier-plugin-svelte](https://www.npmjs.com/package/prettier-plugin-svelte) plugin.

## How do I document my components?

In editors which use the Svelte Language Server you can document Components, functions and exports using specially formatted comments.

````svelte
<script>
	/** What should we call the user? */
	export let name = 'world';
</script>

<!--
@component
Here's some documentation for this component.
It will show up on hover.

- You can use markdown here.
- You can also use code blocks here.
- Usage:
  ```svelte
  <main name="Arethra">
  ```
-->
<main>
	<h1>
		Hello, {name}
	</h1>
</main>
````

Note: The `@component` is necessary in the HTML comment which describes your component.

## Does Svelte scale?

There will be a blog post about this eventually, but in the meantime, check out [this issue](https://github.com/sveltejs/svelte/issues/2546).

## Is there a UI component library?

There are several [UI component libraries](/packages#component-libraries) as well as standalone components listed on [the packages page](/packages).

## How do I test Svelte apps?

How your application is structured and where logic is defined will determine the best way to ensure it is properly tested. It is important to note that not all logic belongs within a component - this includes concerns such as data transformation, cross-component state management, and logging, among others. Remember that the Svelte library has its own test suite, so you do not need to write tests to validate implementation details provided by Svelte.

A Svelte application will typically have three different types of tests: Unit, Component, and End-to-End (E2E).

_Unit Tests_: Focus on testing business logic in isolation. Often this is validating individual functions and edge cases. By minimizing the surface area of these tests they can be kept lean and fast, and by extracting as much logic as possible from your Svelte components more of your application can be covered using them. When creating a new SvelteKit project, you will be asked whether you would like to setup [Vitest](https://vitest.dev/) for unit testing. There are a number of other test runners that could be used as well.

_Component Tests_: Validating that a Svelte component mounts and interacts as expected throughout its lifecycle requires a tool that provides a Document Object Model (DOM). Components can be compiled (since Svelte is a compiler and not a normal library) and mounted to allow asserting against element structure, listeners, state, and all the other capabilities provided by a Svelte component. Tools for component testing range from an in-memory implementation like jsdom paired with a test runner like [Vitest](https://vitest.dev/) to solutions that leverage an actual browser to provide a visual testing capability such as [Playwright](https://playwright.dev/docs/test-components) or [Cypress](https://www.cypress.io/).

_End-to-End Tests_: To ensure your users are able to interact with your application it is necessary to test it as a whole in a manner as close to production as possible. This is done by writing end-to-end (E2E) tests which load and interact with a deployed version of your application in order to simulate how the user will interact with your application. When creating a new SvelteKit project, you will be asked whether you would like to setup [Playwright](https://playwright.dev/) for end-to-end testing. There are many other E2E test libraries available for use as well.

Some resources for getting started with testing:

- [Svelte docs on testing](/docs/svelte/testing)
- [Setup Vitest using the Svelte CLI](/docs/cli/vitest)
- [Svelte Testing Library](https://testing-library.com/docs/svelte-testing-library/example/)
- [Svelte Component Testing in Cypress](https://docs.cypress.io/guides/component-testing/svelte/overview)
- [Example using uvu test runner with JSDOM](https://github.com/lukeed/uvu/tree/master/examples/svelte)
- [Test Svelte components using Vitest & Playwright](https://davipon.hashnode.dev/test-svelte-component-using-vitest-playwright)
- [Component testing with WebdriverIO](https://webdriver.io/docs/component-testing/svelte)

## Is there a router?

The official routing library is [SvelteKit](/docs/kit). SvelteKit provides a filesystem router, server-side rendering (SSR), and hot module reloading (HMR) in one easy-to-use package. It shares similarities with Next.js for React and Nuxt.js for Vue.

However, you can use any router library. A sampling of available routers are highlighted [on the packages page](/packages#routing).

## How do I write a mobile app with Svelte?

While most mobile apps are written without using JavaScript, if you'd like to leverage your existing Svelte components and knowledge of Svelte when building mobile apps, you can turn a [SvelteKit SPA](https://kit.svelte.dev/docs/single-page-apps) into a mobile app with [Tauri](https://v2.tauri.app/start/frontend/sveltekit/) or [Capacitor](https://capacitorjs.com/solution/svelte). Mobile features like the camera, geolocation, and push notifications are available via plugins for both platforms.

Some work has been completed towards [custom renderer support in Svelte 5](https://github.com/sveltejs/svelte/issues/15470), but this feature is not yet available. The custom rendering API would support additional mobile frameworks like Lynx JS and Svelte Native. Svelte Native was an option available for Svelte 4, but Svelte 5 does not currently support it. Svelte Native lets you write NativeScript apps using Svelte components that contain [NativeScript UI components](https://docs.nativescript.org/ui/) rather than DOM elements, which may be familiar for users coming from React Native.

## Can I tell Svelte not to remove my unused styles?

No. Svelte removes the styles from the component and warns you about them in order to prevent issues that would otherwise arise.

Svelte's component style scoping works by generating a class unique to the given component, adding it to the relevant elements in the component that are under Svelte's control, and then adding it to each of the selectors in that component's styles. When the compiler can't see what elements a style selector applies to, there would be two bad options for keeping it:

- If it keeps the selector and adds the scoping class to it, the selector will likely not match the expected elements in the component, and they definitely won't if they were created by a child component or `{@html ...}`.
- If it keeps the selector without adding the scoping class to it, the given style will become a global style, affecting your entire page.

If you need to style something that Svelte can't identify at compile time, you will need to explicitly opt into global styles by using `:global(...)`. But also keep in mind that you can wrap `:global(...)` around only part of a selector. `.foo :global(.bar) { ... }` will style any `.bar` elements that appear within the component's `.foo` elements. As long as there's some parent element in the current component to start from, partially global selectors like this will almost always be able to get you what you want.

## Is Svelte v2 still available?

New features aren't being added to it, and bugs will probably only be fixed if they are extremely nasty or present some sort of security vulnerability.

The documentation is still available [here](https://v2.svelte.dev/guide).

## How do I do hot module reloading?

We recommend using [SvelteKit](/docs/kit), which supports HMR out of the box and is built on top of [Vite](https://vitejs.dev/) and [svelte-hmr](https://github.com/sveltejs/svelte-hmr). There are also community plugins for [rollup](https://github.com/rixo/rollup-plugin-svelte-hot) and [webpack](https://github.com/sveltejs/svelte-loader).

# svelte

```js
// @noErrors
import {
	SvelteComponent,
	SvelteComponentTyped,
	afterUpdate,
	beforeUpdate,
	createContext,
	createEventDispatcher,
	createRawSnippet,
	flushSync,
	fork,
	getAbortSignal,
	getAllContexts,
	getContext,
	hasContext,
	hydrate,
	mount,
	onDestroy,
	onMount,
	setContext,
	settled,
	tick,
	unmount,
	untrack
} from 'svelte';
```

## SvelteComponent

This was the base class for Svelte components in Svelte 4. Svelte 5+ components
are completely different under the hood. For typing, use `Component` instead.
To instantiate components, use `mount` instead.
See [migration guide](/docs/svelte/v5-migration-guide#Components-are-no-longer-classes) for more info.

<div class="ts-block">

```dts
class SvelteComponent<
	Props extends Record<string, any> = Record<string, any>,
	Events extends Record<string, any> = any,
	Slots extends Record<string, any> = any
> {/*…*/}
```

<div class="ts-block-property">

```dts
static element?: typeof HTMLElement;
```

<div class="ts-block-property-details">

The custom element version of the component. Only present if compiled with the `customElement` compiler option

</div>
</div>

<div class="ts-block-property">

```dts
[prop: string]: any;
```

<div class="ts-block-property-details"></div>
</div>

<div class="ts-block-property">

```dts
constructor(options: ComponentConstructorOptions<Properties<Props, Slots>>);
```

<div class="ts-block-property-details">

<div class="ts-block-property-bullets">

- <span class="tag deprecated">deprecated</span> This constructor only exists when using the `asClassComponent` compatibility helper, which
is a stop-gap solution. Migrate towards using `mount` instead. See
[migration guide](https://svelte.dev/docs/svelte/v5-migration-guide#Components-are-no-longer-classes) for more info.

</div>

</div>
</div>

<div class="ts-block-property">

```dts
$destroy(): void;
```

<div class="ts-block-property-details">

<div class="ts-block-property-bullets">

- <span class="tag deprecated">deprecated</span> This method only exists when using one of the legacy compatibility helpers, which
is a stop-gap solution. See [migration guide](https://svelte.dev/docs/svelte/v5-migration-guide#Components-are-no-longer-classes)
for more info.

</div>

</div>
</div>

<div class="ts-block-property">

```dts
$on<K extends Extract<keyof Events, string>>(
	type: K,
	callback: (e: Events[K]) => void
): () => void;
```

<div class="ts-block-property-details">

<div class="ts-block-property-bullets">

- <span class="tag deprecated">deprecated</span> This method only exists when using one of the legacy compatibility helpers, which
is a stop-gap solution. See [migration guide](https://svelte.dev/docs/svelte/v5-migration-guide#Components-are-no-longer-classes)
for more info.

</div>

</div>
</div>

<div class="ts-block-property">

```dts
$set(props: Partial<Props>): void;
```

<div class="ts-block-property-details">

<div class="ts-block-property-bullets">

- <span class="tag deprecated">deprecated</span> This method only exists when using one of the legacy compatibility helpers, which
is a stop-gap solution. See [migration guide](https://svelte.dev/docs/svelte/v5-migration-guide#Components-are-no-longer-classes)
for more info.

</div>

</div>
</div></div>



## SvelteComponentTyped

<blockquote class="tag deprecated note">

Use `Component` instead. See [migration guide](/docs/svelte/v5-migration-guide#Components-are-no-longer-classes) for more information.

</blockquote>

<div class="ts-block">

```dts
class SvelteComponentTyped<
	Props extends Record<string, any> = Record<string, any>,
	Events extends Record<string, any> = any,
	Slots extends Record<string, any> = any
> extends SvelteComponent<Props, Events, Slots> {}
```

</div>



## afterUpdate

<blockquote class="tag deprecated note">

Use [`$effect`](/docs/svelte/$effect) instead

</blockquote>

Schedules a callback to run immediately after the component has been updated.

The first time the callback runs will be after the initial `onMount`.

In runes mode use `$effect` instead.

<div class="ts-block">

```dts
function afterUpdate(fn: () => void): void;
```

</div>



## beforeUpdate

<blockquote class="tag deprecated note">

Use [`$effect.pre`](/docs/svelte/$effect#$effect.pre) instead

</blockquote>

Schedules a callback to run immediately before the component is updated after any state change.

The first time the callback runs will be before the initial `onMount`.

In runes mode use `$effect.pre` instead.

<div class="ts-block">

```dts
function beforeUpdate(fn: () => void): void;
```

</div>



## createContext

<blockquote class="since note">

Available since 5.40.0

</blockquote>

Returns a `[get, set]` pair of functions for working with context in a type-safe way.

`get` will throw an error if no parent component called `set`.

<div class="ts-block">

```dts
function createContext<T>(): [() => T, (context: T) => T];
```

</div>



## createEventDispatcher

<blockquote class="tag deprecated note">

Use callback props and/or the `$host()` rune instead — see [migration guide](/docs/svelte/v5-migration-guide#Event-changes-Component-events)

</blockquote>

Creates an event dispatcher that can be used to dispatch [component events](/docs/svelte/legacy-on#Component-events).
Event dispatchers are functions that can take two arguments: `name` and `detail`.

Component events created with `createEventDispatcher` create a
[CustomEvent](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent).
These events do not [bubble](https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Building_blocks/Events#Event_bubbling_and_capture).
The `detail` argument corresponds to the [CustomEvent.detail](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent/detail)
property and can contain any type of data.

The event dispatcher can be typed to narrow the allowed event names and the type of the `detail` argument:
```ts
const dispatch = createEventDispatcher<{
 loaded: null; // does not take a detail argument
 change: string; // takes a detail argument of type string, which is required
 optional: number | null; // takes an optional detail argument of type number
}>();
```

<div class="ts-block">

```dts
function createEventDispatcher<
	EventMap extends Record<string, any> = any
>(): EventDispatcher<EventMap>;
```

</div>



## createRawSnippet

Create a snippet programmatically

<div class="ts-block">

```dts
function createRawSnippet<Params extends unknown[]>(
	fn: (...params: Getters<Params>) => {
		render: () => string;
		setup?: (element: Element) => void | (() => void);
	}
): Snippet<Params>;
```

</div>



## flushSync

Synchronously flush any pending updates.
Returns void if no callback is provided, otherwise returns the result of calling the callback.

<div class="ts-block">

```dts
function flushSync<T = void>(fn?: (() => T) | undefined): T;
```

</div>



## fork

<blockquote class="since note">

Available since 5.42

</blockquote>

Creates a 'fork', in which state changes are evaluated but not applied to the DOM.
This is useful for speculatively loading data (for example) when you suspect that
the user is about to take some action.

Frameworks like SvelteKit can use this to preload data when the user touches or
hovers over a link, making any subsequent navigation feel instantaneous.

The `fn` parameter is a synchronous function that modifies some state. The
state changes will be reverted after the fork is initialised, then reapplied
if and when the fork is eventually committed.

When it becomes clear that a fork will _not_ be committed (e.g. because the
user navigated elsewhere), it must be discarded to avoid leaking memory.

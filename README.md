# leptos-classes

`leptos-classes` is a small utility crate for passing CSS classes through Leptos component layers without flattening them into a plain `String` too early.

It is designed for component props like `#[prop(into, optional)] classes: Classes`, where intermediate components can keep extending the class list and the final element can still render it reactively with `class=classes`.

## Features

- Static and conditional class entries in one container
- Reactive updates through Leptos signals
- Builder and composable mutation APIs
- `IntoClass` integration for `class=classes`
- Small inline storage for common class counts

## Example

```rust
use leptos::prelude::*;
use leptos_classes::Classes;

#[component]
fn Button(
    #[prop(into, optional)] classes: Classes,
) -> impl IntoView {
    view! {
        <button class=classes.add("btn")>
            "Save"
        </button>
    }
}

#[component]
fn Demo() -> impl IntoView {
    let (is_primary, _) = signal(true);

    view! {
        <Button classes=Classes::builder()
            .with("btn-base")
            .with(("btn-primary", is_primary))
            .with(("btn-secondary", Signal::derive(move || !is_primary.get())))
            .build()
        />
    }
}
```

## API Overview

- `Classes::new()` creates an empty container.
- `Classes::add(...)` appends one class entry and returns the updated value.
- `Classes::add_all(...)` appends multiple class entries from any `IntoIterator` in place.
- `Classes::add_toggle(...)` appends a mutually exclusive pair of classes.
- `Classes::builder()` offers the same operations with `with(...)`, `with_all(...)`, and `with_toggle(...)`.
- `Classes::to_class_string()` materializes the currently active classes into a `String`.
- `Classes::write_class_string(...)` appends the currently active classes into an existing buffer.

## Supported Inputs

`ClassEntry` and `Classes` support common conversions:

- `"foo"`
- `String::from("foo")`
- `("foo", true)`
- `("foo", Signal<bool>)`
- `("foo", ReadSignal<bool>)`
- `(String::from("foo"), ReadSignal<bool>)`
- arrays of all tuple forms above for `Classes`

Empty or whitespace-only class strings are ignored. Adding one emits a `tracing::warn!` once when
the entry is inserted.

## Rendering Semantics

`Classes` implements Leptos' `IntoClass` trait as a full `class="..."` attribute value.

- On SSR, it serializes to a space-separated class string.
- On hydration, it reconciles against the element's current `class` attribute.
- On client-side rendering, it updates the element reactively when tracked signals change.
- Fully static `Classes` values avoid installing a reactive effect and behave like a direct `class="..."` value.
- `Classes` owns the entire `class` attribute of the target element. Reactive updates replace the
  full attribute value rather than reconciling token-by-token.

This means `class=classes` should not be mixed on the same element with:

- `class:foo=...` directives
- Imperative `classList` mutations
- Third-party code that mutates the element's classes after render

Any such unmanaged class changes will be overwritten on the next managed update pass or rebuild.

Calling `IntoClass::reset()` on a built `Classes` value removes the managed `class` attribute and
tears down reactive subscriptions, but the state remains rebuildable with a subsequent
`IntoClass::rebuild()` call.

If you need the final value outside the renderer path, use `to_class_string()`. For actual elements, prefer `class=classes`.

## Testing

The DOM lifecycle coverage for `leptos-classes` is exercised through a real browser integration test.
From the repository root, run:

```bash
cargo test --test browser_test -- --nocapture
```

This starts a dedicated crate-local Leptos test frontend with `cargo leptos serve`, then drives it through Chrome via
`chrome-for-testing-manager` and `thirtyfour`.

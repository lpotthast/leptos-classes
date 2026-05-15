# leptos-classes

[![crates.io](https://img.shields.io/crates/v/leptos-classes.svg)](https://crates.io/crates/leptos-classes)
[![docs.rs](https://docs.rs/leptos-classes/badge.svg)](https://docs.rs/leptos-classes)
[![CI](https://github.com/lpotthast/leptos-classes/actions/workflows/ci.yml/badge.svg)](https://github.com/lpotthast/leptos-classes/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/rust-1.89%2B-blue.svg)](https://www.rust-lang.org/)

`leptos-classes` is a small utility crate for passing class names through Leptos component layers without flattening
them into a plain `String` too early.

It is designed for component props like `#[prop(into, optional)] classes: Classes`, where intermediate components can
keep extending the class list and the final element can still render it reactively with `class=classes`.

## Why

Plain `String` props collapse reactivity and force every intermediate layer to re-concatenate. A typed container that
preserves conditional entries and signal bindings until the final render lets components compose without that tax.

## Installation

```toml
[dependencies]
leptos-classes = "0.1.2"
```

Compatible with Leptos 0.8. Requires Rust 1.89 or newer.

## Example

```rust
use leptos::prelude::*;
use leptos_classes::Classes;

/// The lowest-level component renders the accumulated class list
/// onto the real element.
#[component]
fn Button(
    #[prop(into, optional)] classes: Classes,
    children: Children,
) -> impl IntoView {
    view! {
        <button class=classes.add("btn")>
            {children()}
        </button>
    }
}

/// Components sitting in the middle can add their own classes.
#[component]
fn FormActionButton(
    #[prop(into, optional)] classes: Classes,
    children: Children,
) -> impl IntoView {
    view! {
        <Button classes=classes.add("form-action-btn")>
            {children()}
        </Button>
    }
}

/// Root component defines the initial classes using a builder pattern
/// or can rely on `Into` conversions (see docs).
#[component]
fn FormActions() -> impl IntoView {
    let (is_dirty, _) = signal(true);

    view! {
        // Plain &str conversion.
        <FormActionButton classes="cancel-btn">
            "Cancel"
        </FormActionButton>

        // Builder chain.
        <FormActionButton
            classes=Classes::builder()
                .with("save-btn")
                .with_reactive("is-dirty", is_dirty)
                .build()
        >
            "Save changes"
        </FormActionButton>

        // Method chain.
        <FormActionButton
            classes=Classes::from("help-btn")
                .add("mobile-hidden")
        >
            "Open help"
        </FormActionButton>
    }
}
```

While `is_dirty` is `true`, the save button renders with `class="save-button is-dirty form-action-button button"`.

## Concepts

A `Classes` value is built either by chained mutation (`Classes::from("foo").add(...).add_reactive(...)`) or via
`Classes::builder()` using the `with_*` API. Both shapes accept static names, reactive conditions (signals, memos,
closures), mutually exclusive toggle pairs, and merging of two independently-built values via `Classes::merge`. A
`Classes` can also be constructed through one of several `From` impls covering string types, slices, arrays, and
`(name, condition)` tuples. See the [API docs](https://docs.rs/leptos-classes) for the full surface.

Each entry is a single class token. Empty, whitespace-only, or whitespace-containing names (Unicode definition, so
non-breaking spaces and other non-ASCII whitespace are rejected too) panic at construction. For unknown runtime input,
use `ClassName::try_new` to validate without panicking. Each token may appear in at most one entry within a `Classes`
value. Compose conditions instead of registering the same token twice.

## Rendering Semantics

`Classes` implements Leptos' `IntoClass` trait and owns the full `class="..."` attribute on its target element. SSR
produces a space-separated string. The SSR-then-hydrate path reuses the server-rendered attribute without a redundant
write. Reactive updates replace the whole attribute value rather than reconciling token-by-token. Mixing `class=classes`
on the same element with `class:foo=...` directives or other third-party class mutations is not supported. External
mutations are reconciled only by a reactive update that produces a *different* rendered class string; re-setting a
signal to its current value is a no-op and will not flush a stomped attribute.

Fully static `Classes` values install no reactive effect and behave like a direct `class="..."` attribute.

## Cargo Features

- `nightly` (off by default): forwards to `leptos/nightly`. Enable this only if your project already depends on Leptos
  with the `nightly` feature.

## Testing

`just verify` runs the pre-PR bundle: format check, native check, `wasm32-unknown-unknown` check, clippy, lib tests,
doc tests, browser test, and docs with rustdoc warnings denied. Run `just` for the full list of recipes.

## Related Crates

- [`leptos-styles`](https://github.com/lpotthast/leptos-styles) (work in progress) is the `style`-attribute counterpart
  to this crate, with the same prop-drilling shape and reactive `IntoStyle` integration.

## Contributing

Issues and pull requests are welcome on the [GitHub repository](https://github.com/lpotthast/leptos-classes).

## License

Licensed under either of [MIT License](LICENSE-MIT) or [Apache License, Version 2.0](LICENSE-APACHE) at your option.

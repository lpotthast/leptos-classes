# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-07-17

### Fixed

- Fixed the `nightly` feature's signal conversions so they no longer conflict with Leptos' nightly callable signal
  implementations.

## [0.1.3] - 2026-07-17

### Changed

- Updated the browser integration test harness to `browser-test` 0.4.0 and `leptos-browser-test` 0.3.0.
- Refreshed the test-app's `wasm-bindgen` dependency family to match current `wasm-bindgen-cli` releases.

## [0.1.2] - 2026-05-15

### Changed

- Bumped `leptos-browser-test` dev-dependency to `0.2.0`, which unifies its `tokio-process-tools` version with
  `chrome-for-testing-manager` and brings improved graceful termination support on Windows. This fixes a hang at the
  end of `just browser-test` on Windows where `cargo leptos serve` and its server child were not torn down reliably 
  before.

## [0.1.1] - 2026-05-14

### Changed

- Updated the `README.md` example to a more representative class flow.
- CI now properly runs the browser integration tests.
- Fallible CI jobs now use a custom script block for handling non-successful outcomes (dropped `continue-on-error`).

## [0.1.0] - 2026-05-12

### Added

- Public surface: `Classes`, `ClassesBuilder`, `ClassName`, `InvalidClassName`, `MergeStrategy`.
- Blanket `From` impls on `Classes`: `&'static str`, `String`, `Cow<'static, str>`, `ClassName`,
  `&[N]` and `[N; M]` for `N: Into<ClassName>`, `(N, C)` reactive tuples and their slice/array variants.
- Chaining API on `Classes` and mirroring builder API on `ClassesBuilder` (via `Classes::builder()`):
  `add` / `with`, `add_reactive` / `with_reactive`, `add_all` / `with_all` (any `IntoIterator` of
  `Into<ClassName>`), `add_toggle` / `with_toggle`, `add_parsed` / `with_parsed`, plus `Classes::parse`,
  `Classes::merge`, and `ClassesBuilder::with_merged`. Chaining methods consume `self`; the builder finalizes
  with `build()`.
- Reactive condition shapes for `add_reactive` / `add_toggle`: `bool`, `Signal<bool>`, `ReadSignal<bool>`,
  `RwSignal<bool>`, `Memo<bool>`, and `Fn() -> bool + Send + Sync + 'static` closures.
- `MergeStrategy` with `UnionConditions` (default), `KeepSelf`, `PanicOnConflict`. Non-panic strategies do not
  preserve toggle-pair structure across a merge.
- `ClassName`: validated single-token wrapper around `Cow<'static, str>` with `Deref<Target = str>` and
  `AsRef<str>`. `ClassName::try_new` returns `Result<ClassName, InvalidClassName>`; the `From<&'static str>` /
  `From<String>` / `From<Cow<'static, str>>` impls panic on invalid input.
- `IntoClass` integration so `class=classes` works in Leptos views: fully static class lists skip
  `RenderEffect` setup, hydration reconciles against the element's current `class` attribute, reactive
  updates diff against the live DOM attribute, and `IntoClass::reset` parks the state so a subsequent
  `rebuild` can re-establish reactivity.
- Unconditional token validation: empty, whitespace-only, or whitespace-containing input panics at the
  `ClassName` conversion (use `ClassName::try_new` for fail-soft handling). Token names are structurally unique
  within a `Classes`; duplicates panic at insertion.
- `Classes::to_class_string` for materializing the currently active classes outside the renderer path.
- `nightly` cargo feature forwarding to `leptos/nightly`.
- MSRV: Rust 1.89.0.

[Unreleased]: https://github.com/lpotthast/leptos-classes/compare/v0.1.3...HEAD

[0.1.3]: https://github.com/lpotthast/leptos-classes/compare/v0.1.2...v0.1.3

[0.1.2]: https://github.com/lpotthast/leptos-classes/compare/v0.1.1...v0.1.2

[0.1.1]: https://github.com/lpotthast/leptos-classes/compare/v0.1.0...v0.1.1

[0.1.0]: https://github.com/lpotthast/leptos-classes/releases/tag/v0.1.0

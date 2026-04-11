# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial standalone `leptos-classes` crate release for prop-drillable Leptos class handling.
- `Classes` builder API with `with`, `with_all`, and `with_toggle`.
- Chaining API with `add` and `add_toggle`, plus mutating bulk-add support via `add_all`.
- Conditional class handling for booleans, `Signal<bool>`, and `ReadSignal<bool>`.
- Conversions from common static, owned, conditional, and array-based class inputs.
- `IntoClass` integration for rendering `class=classes` directly in Leptos views.
- Debug-build duplicate class detection with `tracing` warnings.
- Crate README with usage guidance and rendering semantics.
- WASM DOM lifecycle tests covering client-side attribute updates and reset behavior.

### Changed

- `Classes` now skips `RenderEffect` setup for fully static class lists and only installs reactive bookkeeping when
  signal-backed entries are present.
- The recommended DOM test workflow now uses a real browser integration test driven by
  `chrome-for-testing-manager` and `thirtyfour`.
- `add_all` and `with_all` now accept any `IntoIterator`, which makes them work with vectors, arrays, slices, and
  iterator adapters without additional `.into_iter()` plumbing.
- Hydration now reconciles against the element's current `class` attribute on the first render pass instead of waiting
  for a later reactive change.
- Packaging metadata now includes a crate README and docs.rs target configuration for native and wasm builds.

### Fixed

- `write_class_string` now preserves correct spacing when appending into a non-empty buffer.
- Managed class updates now reconcile against the live DOM attribute, which ensures external `class` mutations are
  overwritten on the next managed update pass even when the computed class list itself is unchanged.
- Added missing `ReadSignal<bool>` conversions for owned-string and array-based `Classes` inputs.
- Expanded unit and browser regression coverage for append semantics, state transitions between static and reactive
  rendering, and unmanaged DOM mutations.
- Fixed an initial hydration bug where `Classes::hydrate` could leave the DOM out of sync when the first computed class
  string differed from the element's existing `class` attribute.
- Added lifecycle coverage for `build`, `hydrate`, `rebuild`, `reset`, and reactive updates in wasm-targeted tests.
- Empty and whitespace-only class names are now ignored during rendering, with a `tracing::warn!` emitted when such
  entries are added.

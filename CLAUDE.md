# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common commands

Day-to-day work goes through `just` (recipes are in `Justfile`). One-time setup:
`just once` (runs `enable-wasm` + `install-tools`).

| Task                                                       | Command                                               |
|------------------------------------------------------------|-------------------------------------------------------|
| Format                                                     | `just fmt`                                            |
| Type-check native                                          | `just check`                                          |
| Type-check `wasm32-unknown-unknown` (the published target) | `just check-wasm`                                     |
| Clippy (denies warnings, with `clippy::pedantic`)          | `just clippy`                                         |
| Unit tests + doctests                                      | `just test-lib`                                       |
| Full native test (incl. browser)                           | `just test`                                           |
| Browser integration test (headless)                        | `just browser-test`                                   |
| Same, with a visible Chrome window                         | `just browser-test-visible`                           |
| Same, paused after app start for manual poking             | `just browser-test-pause`                             |
| Serve the test app for manual inspection                   | `just serve-test-app` (binds `127.0.0.1:4300`)        |
| Pre-PR verification bundle                                 | `just verify` (lib tests + wasm check + browser test) |

Running a single unit test: `cargo test --lib <name>` (substring filter, e.g.
`cargo test --lib chained_with_add_keeps_order` or `cargo test --lib toggle::` for a whole
sub-module). Unit tests are grouped into `mod construction / toggle / parsing / reactivity /
validation / rendering / merge` sub-modules inside `src/classes.rs`, plus `mod from_unconditional /
from_conditional` inside `src/convert.rs`.
Running a single browser scenario: there is one `#[tokio::test]`, `browser_tests`, that internally
runs every assertion in `tests/ui_tests/test_classes.rs`. To narrow it down, comment out unwanted
`assert_*_target` calls in that file or add new `BrowserTest` impls under `tests/ui_tests/`.

CI (`.github/workflows/ci.yml`) runs fmt, `cargo check --all-targets --locked`, `check-wasm`,
clippy, lib + doc tests, the browser test, `cargo build --release`, `cargo doc` (with
`RUSTDOCFLAGS=-D warnings`), and an MSRV job pinned to Rust 1.89.0. Keep these green;
`RUSTFLAGS=-D warnings` is set globally in CI, so any new warning fails the build.

## Architecture

This is a single-crate library (`leptos-classes`) plus a separate test-app workspace under
`testing/test-app/`. The published crate excludes `/.cargo/**`, `/.github/**`, `/.idea/**`,
`/AGENTS.md`, `/CLAUDE.md`, `/Justfile`, `/testing/**`, and `/tests/**` (see `Cargo.toml`
`exclude`). Note that `tests/` is the integration-test crate that drives the browser test; the
library itself is `src/` only.

### Module-purity

A "pure" module owns exactly one concept. Code, docs, and tests respect that ownership;
downstream callers reason against the module's contract, not its implementation. There are two
complementary slices.

#### Structural purity (where code lives)

1. **One concept per module, named by what the concept is.** Names are nouns - what the module
   owns, not what it does. `utils`, `helpers`, `common`, `shared`, `misc` are anti-patterns:
   they signal an absence of a concept, not a concept.
2. **Invariants live at the boundary that enforces them.** Validation rules live in the
   constructor of the validated type. Structural invariants live in the type that maintains
   them. Once a value crosses a module boundary, downstream code trusts the invariant and never
   re-validates. If you find yourself re-checking, the wrong module is enforcing the rule.
3. **Default to the narrowest visibility that works; the public surface is a versioned
   contract.** In Rust: `pub(crate)` by default, `pub` only for the published surface. The
   public surface stays small enough to enumerate in the crate root.
4. **Cross-module data flow is explicit through method signatures.** Modules don't reach into
   each other's fields when a method would do. An operation lives on the type it operates on,
   not as a free function in another module.
5. **Each module's tests exercise that module's contract, not its consumers'.** A storage
   module's tests build storage values directly; a user-facing module's tests use the
   user-facing API. A storage-module test that constructs the user-facing wrapper to assert via
   its public surface is a smell.
6. **Sub-test modules inherit from their immediate parent only.** Child test modules use
   `super::*` for the parent's items and helpers. They must not reach across to
   `super::super::*` for items in a sibling module - that pierces two boundaries.
7. **No duplication across modules; one home per piece of logic.** If two modules need the same
   operation, it lives in the module that owns the type it operates on. Resist `*_helpers`
   modules that host logic which should sit on a type.
8. **A module's name and shape should survive a refactor.** When internal representation
   changes, the module's identity stays put; internals swap; callers remain untouched.
9. **Resist adding modules to organize lines.** A new module exists when there's a new concept.
   Splitting `foo.rs` into `foo_part_1.rs` / `foo_part_2.rs` because the file got long violates
   the rule.

#### Documentary purity (what doc comments may say)

A module's doc comments describe what its types and functions are and do, in the module's own
vocabulary. They do not name up-layer callers, do not expose down-layer implementation details,
and do not explain how the module's outputs get consumed elsewhere. The goal is docs that
survive refactors on either side of the module boundary.

1. **Module-level doc states the contract on the first line.** A reader who opens the file
   sees what the module promises before any implementation. Invariants the module enforces (and
   the ones it relies on from below) are stated explicitly.
2. **Reference DOWN or sideways, not UP.** A module may name the primitives and types it
   imports (its direct collaborators, one layer below). It must not name modules that import
   it.
3. **Don't leak callee internals upward.** When a higher-layer doc describes behavior produced
   by a lower layer, describe it in vocabulary at-or-above the higher layer, not in the lower
   layer's variant names or representations.
4. **Peer references are fine.** Items in the same module/file or the same impl block can name
   each other freely - that's the same layer.
5. **Public top-level types are shared vocabulary.** The crate's user-facing public types may
   be cited from any layer when it genuinely helps the reader, because they *are* the
   vocabulary every reader brings to the docs. Don't strip these reflexively.
6. **Diagnostic messages may speak the user-facing layer.** Panic strings or error messages
   tripped at the public API may reference public API methods in their text. Same principle
   inverted: the message surfaces at a layer, so it speaks that layer's vocabulary.
7. **Scope: doc comments only.** In Rust, `///` and `//!`. The principle does not constrain
   implementation comments (`//`) that justify tricky internals, test code, or the body of
   panic/error strings under test.

#### Heuristics

- **Docs:** would this doc still read correctly if every up-layer caller were renamed, deleted,
  or replaced?
- **API:** if I tightened this visibility one level, who breaks? If nobody outside the module's
  intended consumers, the visibility was too loose.
- **Invariants:** am I re-validating something the source type already guaranteed? If yes, the
  rule lives in the wrong module.
- **Tests:** does this test reach across more than one module boundary to set up its fixture?
  If yes, it's testing two layers at once.

#### Why bother

Layer-pure docs are usually *shorter* docs. Stripping "Used by X to do Y" sentences from a leaf
module removes content that didn't earn its keep - the contract was already complete without it.
The discipline doubles as a concision filter.

Structurally pure modules survive refactors. Internal representation can change without
rippling through the rest of the codebase; callers only ever depended on the contract.

### Module layout (`src/`)

The public surface (`lib.rs`) is intentionally tiny: `Classes`, `ClassesBuilder`, `ClassName`,
`InvalidClassName`. Everything else is `pub(crate)`. Internally:

- `class_name.rs` defines `ClassName` (a `Cow<'static, str>` newtype) and the `InvalidClassName`
  error enum (`Empty`, `ContainsWhitespace`). `ClassName::try_new` is the fallible constructor for
  runtime input; the `From<&'static str>` / `From<String>` / `From<Cow<'static, str>>` impls call
  `try_new` and panic via `unwrap_or_else(|err| panic!("{err}"))` -- unconditionally, in both
  debug and release. All token-validity rules live here. The error messages (`"Class name is
  empty or whitespace-only"` and `"Class names must not be whitespace-separated"`) are what
  `#[should_panic(expected = ...)]` validation tests assert on, so keep them stable.
- `condition.rs` defines `ClassCondition` and its internal `Always | Never | When(Signal<bool>)
  | Not(Signal<bool>)` representation. `Always`/`Never` are the static constants. `When(s)` is
  the standard reactive arm produced from any signal/closure source. `Not(s)` is the structural
  negation produced exclusively by `ClassCondition::negate`, which is called by
  `Toggle::into_singles` during the merge slow path to dissolve a structural toggle into two
  flat entries with opposing conditions without allocating a fresh `Signal::derive` arena node;
  `negate` swaps `When(s) <-> Not(s)` and `Always <-> Never` in place. Splitting these arms out
  lets `is_reactive()` be a cheap match without touching any signal. The blanket `From` impls
  cover `bool`, `Signal<bool>`, `ReadSignal<bool>`, `RwSignal<bool>`, `Memo<bool>`, and any
  `Fn() -> bool + Send + Sync + 'static` closure (wrapped via `Signal::derive`).
- `class_item.rs` defines the internal `ClassItem` sum type stored inside `ClassList`, plus its
  two variants in `class_item/single.rs` (`Single { name, when }`) and `class_item/toggle.rs`
  (`Toggle { when, when_true, when_false }`). Keeping the toggle structural (one slot, two
  names) instead of pre-decomposed lets the `estimated_class_len` cache stay tight
  (`max(when_true.len(), when_false.len())` for a toggle, not the sum) and preserves the
  user's "this is a toggle" intent through inserts and non-colliding merges.
  `Toggle::into_singles` is the only place that materializes the `(when_true, When(s)) +
  (when_false, Not(s))` flat pair, and it is called only from the merge slow path.
- `class_list.rs` is the storage: `ClassList` holds a `SmallVec<[ClassItem; 3]>` (chosen so the
  typical prop-drilled list of <=3 classes is heap-free). Name uniqueness is structural: each
  token appears in at most one stored item, where a `Toggle` "owns" both of its branch names.
  Insertion goes through two methods - `add_single(name, condition)` and `add_toggle(when,
  when_true, when_false)`. Both `panic!` (in debug AND release) if the incoming name collides
  with one already stored; `add_toggle` also panics if its two branches share a name (the
  branch-equality check fires first so that case is reported as such, not as a duplicate).
  `is_reactive` and `estimated_class_len` are caches maintained incrementally on insertion via
  `push_item`. `merge(other, strategy)` has a fast path that pushes a non-colliding incoming
  `Toggle` whole, preserving its structural identity; the slow path (collision under any
  strategy) dissolves the involved toggle into `(when_true, When(s)) + (when_false, Not(s))`
  via `Toggle::into_singles` and routes each half through the strategy individually. The merge
  paths all bypass `push_item`'s cache maintenance and rely on a single `recompute_caches()`
  call at the end of `merge` to repair both caches - do not call `push_item` from inside the
  merge loop. When `union_at` dissolves a self-side `Toggle` on collision, the two resulting
  `Single`s are always inserted in `when_true`-then-`when_false` order regardless of which
  half matched, so the user's insertion order survives the dissolution.
- `classes.rs` is the user-facing `Classes` and `ClassesBuilder`. `Classes` exposes consuming
  mutation (`add`, `add_reactive`, `add_all`, `add_toggle`, `add_parsed`, `parse`, each returning
  `Self`) and `ClassesBuilder` mirrors them as `with`, `with_reactive`, `with_all`, `with_toggle`,
  `with_parsed` + a final `build()`. All paths delegate to `ClassList::add_single` or
  `ClassList::add_toggle`. The `parse` / `add_parsed` / `with_parsed` family is the explicit
  opt-in for splitting a runtime whitespace-separated class string into one always-active entry
  per non-empty token (whereas a single-entry path like `Classes::from("foo bar")` deliberately
  panics on embedded whitespace).
- `convert.rs` provides every blanket `From` impl on `Classes` that lets callers write
  `Classes::from("foo")`, `Classes::from(String::from("foo"))`, `Classes::from(["foo", "bar"])`,
  `Classes::from(("foo", signal))`, `Classes::from([("foo", signal), ...])`, etc. Tuple impls are
  generic over `C: Into<ClassCondition>`, so any condition shape accepted by
  `add_reactive` works on the tuple side too. New input shapes belong here, not on `Classes`.
- `into_class.rs` is the Leptos integration. It implements `tachys::html::class::IntoClass`
  for `Classes` with `should_overwrite() = true` -- meaning a `Classes` value owns the entire
  `class="..."` attribute on its element. The render state is `ClassesState` (`#[doc(hidden)]
  pub` because `IntoClass::State` must be nameable), structured as a parent struct holding
  `el: Elem` plus a private `kind: ClassesKind` enum. Keeping the element at the parent level
  (rather than inside each variant) means `take_buffers` and the kind-rebuilder only touch
  `self.kind` -- `self.el` stays put and never has to be cloned for state transitions. The two
  kinds are:
    - `ClassesKind::Static { buffers }` for fully non-reactive class lists. No `RenderEffect`
      is installed; the attribute is written at `build`/`hydrate` and `rebuild` reuses the
      cached `scratch` to compare against `current` before deciding whether to `setAttribute`,
      so a `class=move || some_static_classes()` closure that re-fires often stays
      allocation-free across rebuilds.
    - `ClassesKind::Reactive { render_effect }` wraps a `RenderEffect<ClassBuffers>` carrying
      the buffer pair, threaded through each effect run via the `prev` parameter.
      `sync_class_attribute` writes into `scratch`, diffs against `current`, flushes only on
      change, then `std::mem::swap`s the two. The reactive path performs zero DOM reads in
      steady state; the only `getAttribute` call in the whole lifecycle is the one-shot seed
      inside `hydrate::<true>` that lets the SSR-then-hydrate path skip a redundant
      `setAttribute` when the rendered value matches what the server already wrote.
  `ClassesKind` derives a manual `Default` (empty `Static`) so `take_buffers` can use
  `std::mem::take` to extract the live kind without cloning `Elem`. The buffer pair itself is
  the private `ClassBuffers` struct (`current` = last value written to the DOM, treated as
  authoritative; `scratch` = working buffer for the next render).
  `rebuild` collapses to a single decision: `state.take_buffers()` recovers the cached buffer
  pair from whichever kind is current, then `ClassesKind::build(self, &state.el, buffers)`
  asks the *new* `Classes` whether it is reactive and picks the new kind accordingly. All four
  (cached, new) transitions (Static<->Static, Static->Reactive, Reactive->Reactive,
  Reactive->Static) go through that one constructor and never reallocate. The only `Elem`
  clone that happens during normal operation is the one captured by the reactive arm's effect
  closure (necessary because the closure is `'static` and must own its element handle).
  `dry_resolve` must touch every reactive dependency (via
  `ClassList::touch_reactive_dependencies`) so Leptos can record subscriptions before the first
  paint. `reset` removes the `class` attribute and parks the state in the `Static` arm so a
  later `rebuild` can re-establish reactivity. The browser-side `transition-target` test in the
  test app exercises every direction of these transitions, including the freeze-while-static /
  re-enable-reactive path.

### Invariants worth preserving

- A `ClassName` is **one CSS token only**: non-empty, no whitespace by the Unicode definition
  (`char::is_whitespace`, which rejects ASCII space/tab/newline plus characters like NBSP
  `U+00A0` and line/paragraph separators). The invariant is enforced at construction
  (`ClassName::try_new` returns `Err`; the panicking `From` impls call `unwrap_or_else(panic!)`).
  Downstream code (`ClassList`, `into_class`) relies on it and does not re-check. Multiple
  classes go through `add_all` / `with_all` / array `From` impls, or `parse` / `add_parsed` /
  `with_parsed` (which split on Unicode whitespace via `str::split_whitespace`) for runtime
  whitespace-separated strings.
- **Invalid tokens panic unconditionally**, in both debug and release, for both `ClassName::from`
  and every `Classes::add*` / `with*` / `Classes::from` path (they all delegate to
  `ClassName::from`). There is no warn-and-skip release fallback. Runtime user input that may be
  invalid must be filtered through `ClassName::try_new` first.
- **Name uniqueness is structural and panics unconditionally** on collision. Each class token may
  appear in at most one entry across a `Classes` value, including the case where a single
  entry's name matches one branch of a toggle. `ClassList::add_single` and `ClassList::add_toggle`
  both panic in debug and release at insertion time. There is no soft warning, no debug-only
  diagnostic; duplicates were always a programming error rather than something to deduplicate.
  Callers wanting "this token under multiple conditions" must compose the conditions instead
  (e.g. `add_reactive("foo", move || a.get() || b.get())`).
- The library has **no debug-only behavior** any more. Both invalid-token rejection and
  duplicate-name rejection are unconditional panics.
- `Classes` represents a **whole** `class` attribute. Mixing it on the same element with
  `class:foo=...` directives, `classList` mutations, or third-party DOM tweaks is unsupported.
  The reactive path treats its in-memory `current` (the last value we wrote) as authoritative
  for the DOM, so external mutations persist in the DOM until a *value-changing* signal update
  produces a different rendered string and triggers a write. Re-setting a signal to the same
  value renders the same string and is intentionally a no-op write -- it does not reconcile.
  The `managed-target` browser test asserts this contract.
- Static lists install no `RenderEffect`; the attribute is set once at `build`/`hydrate`.
  `rebuild` on a still-static value allocates a small scratch String to compare against the
  cached `current` and skips the DOM write when unchanged (the static rebuild path is rare, so
  one per-rebuild allocation is acceptable). When changing `into_class.rs`, preserve the
  reactive `(current, scratch)` buffer-swap pattern in `sync_class_attribute` -- it must stay
  free of DOM reads and free of per-tick allocations -- and the SSR `html_len` estimate
  (`Classes::estimated_class_len`, which overshoots for toggles whose shorter branch is active)
  which feeds the same buffer-sizing.
- `Classes::should_overwrite()` returning `true` is what tells Leptos to drop any pre-existing
  attribute value; do not flip it without auditing the hydrate path against the SSR-then-hydrate
  test (`hydrate-target`, which asserts the client-rebuilt value supersedes the server-rendered
  one).
- Public items must keep their doc comments: `missing_docs = "warn"` plus `RUSTDOCFLAGS=-D
  warnings` in CI means an undocumented public item fails the `doc` job.
- `clippy::pedantic` is denied crate-wide. The lint allow-list at the bottom of `Cargo.toml` is
  deliberate; reach for adding to it sparingly and only when the pedantic warning is genuinely
  noise.

### Test architecture

- **Unit tests** live next to the code as `#[cfg(test)] mod tests` blocks in `src/classes.rs`,
  `src/convert.rs`, and `src/into_class.rs`. They use `assertr` and exercise reactive behavior
  directly via `leptos::prelude::signal` (no DOM).
- **Doctests** in `src/classes.rs` are real integration with Leptos components and run in
  `cargo test --doc`.
- **Browser tests** live in `tests/`. The single `#[tokio::test] browser_tests` entry in
  `tests/browser_test.rs` boots `testing/test-app` via `leptos_browser_test::LeptosTestAppConfig`
  (which shells out to `cargo leptos serve`), then drives Chrome through `thirtyfour`.
  Page-objects are in `tests/pages/` and scenarios in `tests/ui_tests/`. Each rendered fixture
  in the test app (`testing/test-app/src/targets/*.rs` -- one file per scenario: `static`,
  `reactive`, `toggle`, `hydrate`, `transition`, `managed`, `drilling`, `empty`) corresponds to
  one `assert_*_target` block in `tests/ui_tests/test_classes.rs`. When adding a behavior to the
  library, mirror it as a new target component + a new `wait_for_class` assertion rather than
  relying on unit tests alone -- the browser test is what validates the real Leptos `IntoClass`
  lifecycle (SSR -> hydrate -> reactive update -> reset/rebuild).

### Compatibility constraints

- MSRV is **Rust 1.89.0** and is enforced by a CI job. No `let`-`else`-on-trait-fns,
  edition-2024-only patterns, or std APIs newer than 1.89 unless the MSRV is bumped
  in `Cargo.toml`, the README, and the CI matrix together.
- Library-side Leptos dep is `leptos = "0.8.19"` with `default-features = false`.
  Dev-deps re-enable `csr` + `hydrate` for tests. The `nightly` cargo feature only
  forwards to `leptos/nightly`.

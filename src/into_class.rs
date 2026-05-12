use crate::Classes;
use leptos::reactive::effect::RenderEffect;
use leptos::tachys::{
    html::class::IntoClass,
    renderer::{Rndr, dom::Element},
};
use leptos::web_sys;

const CLASS_ATTRIBUTE: &str = "class";

#[doc(hidden)]
#[derive(Clone)]
pub struct Elem(web_sys::Element);

impl Elem {
    /// Reads the live `class` attribute from the underlying DOM element. The `web_sys` API copies
    /// the JS string into a Rust `String`, requiring an allocation.
    fn read_class_attribute(&self) -> String {
        self.0.get_attribute(CLASS_ATTRIBUTE).unwrap_or_default()
    }

    /// Set `value` as the `class` attribute of `el`. Removes the attribute should `value` be empty.
    fn set_class_attribute(&self, value: &str) {
        if value.is_empty() {
            self.remove_class_attribute();
        } else {
            Rndr::set_attribute(&self.0, CLASS_ATTRIBUTE, value);
        }
    }

    fn remove_class_attribute(&self) {
        Rndr::remove_attribute(&self.0, CLASS_ATTRIBUTE);
    }
}

/// Reusable string buffers for rendering and diffing the `class` attribute.
///
/// `current` is the value we last wrote to the DOM and is treated as authoritative for what the
/// attribute currently contains. `scratch` is a working buffer to which classes can be written. It
/// only requires an in-memory comparison between (the new) `scratch` and (the old) `current` state
/// to determine whether the materialized class string changed, in which case we flush to the DOM
/// and `std::mem::swap` the two buffers. Both fields keep their capacity untouched, so this type
/// performs little allocations once stable.
#[doc(hidden)]
#[derive(Default)]
pub struct ClassBuffers {
    current: String,
    scratch: String,
}

impl ClassBuffers {
    /// Diffs the freshly rendered class string against the last value written to the DOM and
    /// flushes to the DOM only on change.
    ///
    /// `self.scratch` is cleared and re-rendered from `classes`; if it differs from `self.current`
    /// the new string is written to `el` via `set_class_attribute`; finally the two buffers are
    /// swapped so the freshly rendered string becomes `current` for the next call.
    ///
    /// Performance contract (relied on by the reactive `RenderEffect` path): this function must
    /// stay free of DOM reads (no `getAttribute`) and free of per-tick allocations - both buffers
    /// retain their capacity across calls via the swap, so `clear` + `write_active_classes` reuse
    /// the existing backing storage. Adding a DOM read here would defeat the whole point of
    /// caching `current` in memory; reallocating a fresh `String` per call would defeat the
    /// buffer reuse.
    fn sync_class_attribute(&mut self, classes: &Classes, el: &Elem) {
        self.scratch.clear();
        classes.write_active_classes(&mut self.scratch);

        if self.scratch != self.current {
            el.set_class_attribute(&self.scratch);
        }

        std::mem::swap(&mut self.current, &mut self.scratch);
    }
}

/// Per-element render state held by Leptos for a `Classes` attribute.
///
/// `el` lives at the parent-struct level (rather than inside each [`ClassesKind`] variant) so
/// that recovering the buffers on `rebuild` / `reset` does not have to clone or move `el` out of
/// a variant. The render path only ever borrows `&self.el` while writing to `self.kind`, which
/// the borrow checker permits because the two fields are disjoint. `rebuild` decides between
/// the two kinds by re-checking `Classes::is_reactive()` on the *new* value, so each rebuild
/// may flip kinds depending on the freshly produced `Classes`.
#[doc(hidden)]
pub struct ClassesState {
    el: Elem,
    kind: ClassesKind,
}

/// Variant-specific payload of [`ClassesState`].
///
/// `Static` holds the [`ClassBuffers`] directly. `Reactive` hides them inside the
/// `RenderEffect`'s value, which the effect threads through each run via its `prev` argument
/// so the allocations are reused across reactive ticks. The `Default` impl picks an empty
/// `Static` so `take_buffers` can use `std::mem::take` to extract the live kind without ever
/// having to clone `Elem`.
enum ClassesKind {
    Static {
        buffers: ClassBuffers,
    },
    Reactive {
        render_effect: RenderEffect<ClassBuffers>,
    },
}

impl Default for ClassesKind {
    fn default() -> Self {
        Self::Static {
            buffers: ClassBuffers::default(),
        }
    }
}

impl ClassesKind {
    /// Builds the kind that matches `classes`'s reactivity, performing the initial
    /// compare-and-flush against the supplied `buffers` (whose `current` is whatever the caller
    /// considers authoritative for the live DOM attribute - empty for a fresh build, last-written
    /// for a rebuild, DOM-seeded for an SSR hydrate). For the reactive arm, the closure clones
    /// `el` once (the only clone that survives this constructor); for the static arm it borrows
    /// `el` and writes through it directly.
    fn build(classes: Classes, el: &Elem, mut buffers: ClassBuffers) -> Self {
        if classes.is_reactive() {
            let closure_el = el.clone();
            Self::Reactive {
                render_effect: RenderEffect::new_with_value(
                    move |prev| {
                        let mut buffers = prev.unwrap_or_default();
                        buffers.sync_class_attribute(&classes, &closure_el);
                        buffers
                    },
                    Some(buffers),
                ),
            }
        } else {
            buffers.sync_class_attribute(&classes, el);
            Self::Static { buffers }
        }
    }
}

impl ClassesState {
    fn new(classes: Classes, el: Elem, buffers: ClassBuffers) -> Self {
        let kind = ClassesKind::build(classes, &el, buffers);
        Self { el, kind }
    }

    /// Recovers the cached buffer pair regardless of which kind `self` currently holds. `Static`
    /// hands the buffers over directly via `std::mem::take`; `Reactive` extracts them from the
    /// effect's value via `take_value`. Either path leaves `self.kind` as the `Default`
    /// (empty-Static) sentinel; the caller is expected to overwrite `self.kind` before any
    /// subsequent render. `self.el` is untouched, so no clone is required.
    fn take_buffers(&mut self) -> ClassBuffers {
        // This take constructs a default kind to be put into place. This will be of kind `Static`,
        // requiring no allocations, as `String::new` does not immediately allocate.
        match std::mem::take(&mut self.kind) {
            ClassesKind::Static { buffers } => buffers,
            ClassesKind::Reactive { render_effect } => {
                render_effect.take_value().unwrap_or_default()
            }
        }
    }
}

impl IntoClass for Classes {
    type AsyncOutput = Self;
    type State = ClassesState;
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn html_len(&self) -> usize {
        // Estimate is the sum of class names and required separator spaces.
        self.estimated_class_len()
    }

    fn to_html(self, class: &mut String) {
        // SSR path: build class string directly, avoiding intermediate allocations.
        self.write_active_classes(class);
    }

    fn should_overwrite(&self) -> bool {
        // `Classes` owns the whole `class` attribute!
        true
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        let el = Elem(el.clone());
        let mut buffers = ClassBuffers::default();
        if FROM_SERVER {
            // Seed `current` with what the server rendered, so a matching client-side render can
            // skip a redundant `set_class_attribute` call. This is the only DOM read in the
            // entire lifecycle.
            buffers.current = el.read_class_attribute();
        }
        ClassesState::new(self, el, buffers)
    }

    fn build(self, el: &Element) -> Self::State {
        let el = Elem(el.clone());
        ClassesState::new(self, el, ClassBuffers::default())
    }

    fn rebuild(self, state: &mut Self::State) {
        // Single decision point for the new kind: ask the *new* `Classes` whether it is
        // reactive, independent of which kind the cached state currently holds. The buffers are
        // recovered uniformly from either kind so every (cached, new) transition - including
        // Static<->Reactive flips when a `move || ...` closure swaps between reactive and
        // non-reactive `Classes` across re-renders - goes through `ClassesKind::build`.
        let buffers = state.take_buffers();
        state.kind = ClassesKind::build(self, &state.el, buffers);
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }

    fn dry_resolve(&mut self) {
        // Touch all reactive values to register dependencies.
        self.touch_reactive_dependencies();
    }

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }

    fn reset(state: &mut Self::State) {
        let mut buffers = state.take_buffers();
        buffers.current.clear();
        state.el.remove_class_attribute();
        state.kind = ClassesKind::Static { buffers };
    }
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;
    use leptos::tachys::html::class::IntoClass;

    use crate::Classes;

    #[test]
    fn to_html_writes_active_tokens() {
        let classes = Classes::builder().with("foo").with("bar").build();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("foo bar".to_string());
    }

    #[test]
    fn to_html_writes_nothing_when_empty() {
        let classes = Classes::new();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to(String::new());
    }

    #[test]
    fn to_html_skips_inactive_entries() {
        let classes = Classes::builder()
            .with_reactive("active", true)
            .with_reactive("disabled", false)
            .with_reactive("visible", true)
            .build();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("active visible".to_string());
    }

    #[test]
    fn to_html_appends_to_nonempty_buffer() {
        let classes = Classes::builder().with("new-class").build();
        let mut html = String::from("existing");
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("existing new-class".to_string());
    }

    #[test]
    fn should_overwrite_is_true() {
        let classes = Classes::new();
        assert_that!(classes.should_overwrite()).is_true();
    }

    #[test]
    fn html_len_is_exact_for_all_single_entries() {
        // The estimate is sum(name_len) + (n - 1) separators, which exactly matches the
        // rendered length when every entry is a single always-active token.
        let classes = Classes::builder().with("foo").with("bar").build();
        let rendered = classes.clone().to_class_string();

        assert_that!(classes.html_len()).is_equal_to(rendered.len());
    }

    #[test]
    fn html_len_overshoots_toggle_by_inactive_branch_diff() {
        // A toggle pair contributes max(when_true.len(), when_false.len()) to the estimate, so
        // when the shorter branch is active the estimate overshoots by the length difference.
        let classes = Classes::builder()
            .with("base")
            .with_toggle(false, "active-state", "off") // false branch "off" is active.
            .build();
        let rendered = classes.clone().to_class_string();

        let longer_branch = "active-state".len();
        let active_branch = "off".len();
        let expected_overshoot = longer_branch - active_branch;

        assert_that!(classes.html_len()).is_equal_to(rendered.len() + expected_overshoot);
    }
}

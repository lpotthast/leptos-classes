use crate::class_list::ClassList;
use crate::class_name::ClassName;
use crate::condition::ClassCondition;

/// Strategy for handling token collisions in [`Classes::merge`] and
/// [`ClassesBuilder::with_merged`].
///
/// The variants differ only in what happens on a token collision; non-overlapping entries are
/// appended in either case. [`UnionConditions`](Self::UnionConditions) is the [`Default`] and the
/// recommended choice when you do not have a specific reason to pick another: it never panics
/// (regardless of what the caller passes in) and never silently drops a caller-supplied
/// condition.
///
/// # Comparison with `leptos-styles`
///
/// `Styles::merge` in the sibling crate has a fixed override-with-fallback semantic
/// (theme-then-user-override layering). `Classes::merge` instead asks the caller to choose,
/// because classes have no values to "override" - the right behavior on collision depends entirely
/// on the caller's intent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MergeStrategy {
    /// On collision, replace `self`'s entry with one whose condition is the logical OR of both
    /// sides. The token renders when *either* condition is active.
    ///
    /// Does not preserve toggle-pair structure: if either side of the collision was a toggle
    /// half, the OR produces a well-defined condition for the merged entry, but the other half
    /// of any colliding toggle is left as an orphan flat entry.
    #[default]
    UnionConditions,
    /// On collision, drop the entry from `other` and leave `self`'s entry unchanged. Useful when
    /// `self` is a layered default that must win against override attempts. To get the opposite
    /// direction (`other` wins), call `other.merge(self, MergeStrategy::KeepSelf)` - this yields
    /// the same entry set with `other`'s conditions surviving collisions.
    ///
    /// If a dropped entry was half of a toggle pair in `other`, the surviving half lands as a
    /// regular flat entry under its own reactive condition - the toggle pair is not preserved as
    /// a structural unit.
    KeepSelf,
    /// Panic with the standard duplicate-class-token message on the first colliding entry from
    /// `other`. The strictest option; equivalent to manually re-adding each entry of `other` into
    /// `self` via `add` / `add_reactive` / `add_toggle`.
    ///
    /// Only safe when both sides of the merge are under your own control. Avoid this strategy
    /// for any merge that combines an arbitrary `classes: Classes` prop with internal classes:
    /// a caller passing a colliding token would crash the component. Use
    /// [`UnionConditions`](Self::UnionConditions) or [`KeepSelf`](Self::KeepSelf) there.
    PanicOnConflict,
}

/// Leptos component-prop-utility to drill down a list of classes.
///
/// # Duplicate Handling
///
/// Each class token may appear in at most one entry across a `Classes` value. Registering the
/// same token twice, including the case where an `add` / `add_reactive` entry's name matches one
/// branch of an `add_toggle`, panics in both debug and release builds at the point of insertion.
/// Compose conditions instead: if you want `"foo"` to render when either of two signals is true,
/// write `add_reactive("foo", move || a.get() || b.get())` rather than adding `"foo"` twice.
///
/// # Class Token Validation
///
/// Each entry must be one class token, not a whitespace-separated class string. Invalid
/// input (empty, whitespace-only, or containing any whitespace, by the Unicode definition)
/// panics in both debug and release builds at the [`ClassName`] conversion. For runtime input
/// you want to handle without a panic, validate via [`ClassName::try_new`] and only feed
/// successfully-validated tokens through the entry methods.
///
/// # Attribute Ownership
///
/// `Classes` represents a complete `class="..."` attribute value. When rendered onto an element,
/// it owns the full `class` attribute and will overwrite unmanaged class mutations on the next
/// managed update pass or rebuild.
///
/// # Example
/// ```rust
/// use leptos::prelude::*;
/// use leptos_classes::Classes;
///
/// /// The lowest-level component renders the class-list onto an actual HTML element.
/// #[component]
/// fn NeedingClasses(
///     #[prop(into, optional)] classes: Classes,
/// ) -> impl IntoView {
///     view! {
///         <div class=classes/>
///     }
/// }
///
/// /// Components sitting in the middle can add their own classes.
/// #[component]
/// fn ExtendingClasses(
///     #[prop(into, optional)] classes: Classes,
/// ) -> impl IntoView {
///     view! {
///         <NeedingClasses classes=classes.add("additional-class")/>
///     }
/// }
///
/// /// Root component defines the initial classes using a builder pattern or can rely on `Into`
/// /// conversions (see docs).
/// #[component]
/// fn ProvidingClasses() -> impl IntoView {
///     let (show_second, _) = signal(true);
///     view! {
///         <ExtendingClasses classes="single-class"/>
///         <ExtendingClasses classes=Classes::builder()
///             .with("first")
///             .with_reactive("second", show_second)
///             .build()/>
///     }
/// }
/// ```
#[derive(Clone, Debug, Default)]
pub struct Classes {
    pub(crate) classes: ClassList,
}

impl Classes {
    /// Creates a builder for a class list.
    #[must_use]
    pub fn builder() -> ClassesBuilder {
        ClassesBuilder::default()
    }

    /// Creates an empty class list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            classes: ClassList::empty(),
        }
    }

    /// Parses a whitespace-separated class string into a list of always-active entries.
    ///
    /// Splits `input` on Unicode whitespace ([`str::split_whitespace`]) and creates one
    /// always-active entry per non-empty token. Empty input or whitespace-only input produces an
    /// empty `Classes`. Non-breaking spaces (`U+00A0`) and other non-ASCII whitespace split
    /// tokens just like ASCII whitespace, so pasting `"foo\u{00A0}bar"` from a rich-text source
    /// yields two tokens rather than one whitespace-bearing token that would then fail
    /// validation.
    ///
    /// Unlike `Classes::from(&str)`, which treats its argument as a single class token (and
    /// panics on embedded whitespace), `parse` is the explicit opt-in for turning a runtime
    /// `"foo bar baz"` style string into multiple class entries.
    ///
    /// Tokens are inserted with the same uniqueness rule as [`Classes::add`]: if `input`
    /// contains the same token more than once (e.g. `"foo foo"`), the second insertion panics.
    /// Pre-deduplicate runtime input if you cannot guarantee distinct tokens.
    ///
    /// # Example
    /// ```rust
    /// use assertr::prelude::*;
    /// use leptos_classes::Classes;
    ///
    /// let classes = Classes::parse("btn btn-primary  btn-large");
    /// assert_that!(classes.to_class_string()).is_equal_to("btn btn-primary btn-large");
    /// ```
    #[must_use]
    pub fn parse(input: &str) -> Self {
        Self::new().add_parsed(input)
    }

    /// Adds one always-active class token.
    ///
    /// Panics if `name` is empty, whitespace-only, or contains any whitespace (Unicode
    /// definition: see [`char::is_whitespace`]), or if the token is already present in this
    /// `Classes` (see [Duplicate Handling](Classes#duplicate-handling)).
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, name: impl Into<ClassName>) -> Self {
        self.classes
            .add_single(name.into(), ClassCondition::always());
        self
    }

    /// Adds one reactive class token, controlled by `when`.
    ///
    /// Same validation policy for `name` as [`Classes::add`].
    ///
    /// # Accepted `when` shapes
    ///
    /// `when` accepts any value that converts into the internal condition type:
    ///
    /// - `bool` - treated as always-active when `true`, never-active when `false`; no
    ///   reactive subscription is installed.
    /// - `Signal<bool>` - reactive Leptos signal.
    /// - `ReadSignal<bool>` - read half of a `signal(...)` pair.
    /// - `RwSignal<bool>` - reactive read-write signal.
    /// - `Memo<bool>` - reactive memoized computation.
    /// - Any `Fn() -> bool + Send + Sync + 'static` closure, e.g.
    ///   `move || is_active.get() && !disabled.get()`.
    #[must_use]
    pub fn add_reactive(
        mut self,
        name: impl Into<ClassName>,
        when: impl Into<ClassCondition>,
    ) -> Self {
        self.classes.add_single(name.into(), when.into());
        self
    }

    /// Adds multiple always-active class tokens.
    ///
    /// Each `name` is validated independently per [`Classes::add`]'s policy. Iteration
    /// short-circuits on the first invalid or duplicate token: the panic fires from inside
    /// the loop, so items past the offending one are never inspected.
    #[must_use]
    pub fn add_all<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ClassName>,
    {
        for name in iter {
            self.classes
                .add_single(name.into(), ClassCondition::always());
        }
        self
    }

    /// Splits `input` on Unicode whitespace ([`str::split_whitespace`]) and appends each
    /// non-empty token as an always-active class entry.
    ///
    /// Use this when you have a runtime class string you cannot pre-tokenize. Empty input or
    /// whitespace-only input is a no-op. Non-breaking spaces (`U+00A0`) and other non-ASCII
    /// whitespace split tokens just like ASCII whitespace.
    ///
    /// Tokens land under the same uniqueness rule as [`Classes::add`]: a token from `input`
    /// that duplicates one already present on `self`, or that appears twice within `input`
    /// itself, panics at insertion time. Pre-deduplicate runtime input if you cannot guarantee
    /// distinct tokens.
    ///
    /// # Example
    /// ```rust
    /// use assertr::prelude::*;
    /// use leptos_classes::Classes;
    ///
    /// let classes = Classes::from("base").add_parsed("  primary  large ");
    /// assert_that!(classes.to_class_string()).is_equal_to("base primary large");
    /// ```
    #[must_use]
    pub fn add_parsed(self, input: &str) -> Self {
        self.add_all(input.split_whitespace().map(str::to_owned))
    }

    /// Adds a pair of mutually exclusive reactive classes.
    ///
    /// The `when_true` class is active when the condition is `true`, the `when_false` class
    /// when it is `false`. Panics if either branch is invalid (empty, whitespace-only, or
    /// containing any whitespace, by the Unicode definition: see [`char::is_whitespace`]), if
    /// `when_true` equals `when_false`, or if either branch collides with a class token
    /// already registered on this `Classes` (see [Duplicate Handling](Classes#duplicate-handling)).
    ///
    /// See [`Classes::add_reactive`] for the list of accepted `when` shapes.
    ///
    /// # Example
    /// ```rust
    /// use leptos::prelude::*;
    /// use leptos_classes::Classes;
    ///
    /// let (is_active, _) = signal(true);
    /// let classes = Classes::new()
    ///     .add_toggle(is_active, "active", "inactive");
    /// ```
    #[must_use]
    pub fn add_toggle(
        mut self,
        when: impl Into<ClassCondition>,
        when_true: impl Into<ClassName>,
        when_false: impl Into<ClassName>,
    ) -> Self {
        self.classes
            .add_toggle(when.into(), when_true.into(), when_false.into());
        self
    }

    /// Combines another `Classes` value into this one, appending every entry from `other` and
    /// applying `strategy` on token collisions.
    ///
    /// Use `merge` when you receive two independently-produced `Classes` values (a
    /// `classes: Classes` prop combined with a helper return, two hook return values, a
    /// third-party value combined with your own) that you cannot fold into one chained
    /// construction. When you control both producers, prefer chained `add_*` calls on a single
    /// `Classes`.
    ///
    /// Prefer [`MergeStrategy::default()`] (which is
    /// [`UnionConditions`](MergeStrategy::UnionConditions)) unless you have a specific reason to
    /// drop or reject collisions. It is the only strategy that never panics on caller input and
    /// never silently discards a caller-supplied condition. See [`MergeStrategy`] for per-variant
    /// semantics, including how each strategy treats collisions that involve a toggle half
    /// (toggle-pair structure is not preserved across any merge).
    ///
    /// # Example
    /// ```rust
    /// use leptos::prelude::*;
    /// use leptos_classes::{Classes, MergeStrategy};
    ///
    /// /// A helper that produces a self-contained `Classes`.
    /// fn primary_button_classes() -> Classes {
    ///     Classes::from("btn").add("bg-blue-600").add("text-white")
    /// }
    ///
    /// /// Component receives a `Classes` prop and merges in its own internal classes.
    /// /// `MergeStrategy::default()` (== `UnionConditions`) is the right pick here: a caller
    /// /// passing a colliding token must not crash the component.
    /// #[component]
    /// fn Button(#[prop(into, optional)] classes: Classes) -> impl IntoView {
    ///     let merged = classes.merge(primary_button_classes(), MergeStrategy::default());
    ///     view! { <button class=merged>"Click me"</button> }
    /// }
    /// ```
    #[must_use]
    pub fn merge(mut self, other: Classes, strategy: MergeStrategy) -> Self {
        self.classes.merge(other.classes, strategy);
        self
    }

    /// Returns the currently active classes as a space-separated `String`.
    ///
    /// If called within a reactive scope, signal reads register the surrounding scope as a
    /// subscriber. Prefer `class=classes` (via `IntoClass`) for rendering: it reuses the
    /// string buffer across reactive updates instead of allocating a fresh `String` each time.
    #[must_use]
    pub fn to_class_string(&self) -> String {
        let mut s = String::new();
        self.write_active_classes(&mut s);
        s
    }

    /// Appends all active classes to the given string buffer.
    ///
    /// If `buf` is non-empty, a single space is written before the first active token so it
    /// separates cleanly from existing content. If no entries are active, `buf` is left
    /// untouched. This method is zero-allocation when the buffer has sufficient capacity.
    pub(crate) fn write_active_classes(&self, buf: &mut String) {
        self.classes.write_active_classes(buf);
    }

    pub(crate) fn estimated_class_len(&self) -> usize {
        self.classes.estimated_class_len()
    }

    /// Whether there is any reactivity involved in this set of classes. When this returns `true`,
    /// rendering should take place in a reactivity-tracking context. When this returns `false`, one
    /// could say that these classes are "static" in the sense that a one-time rendering is enough.
    pub(crate) fn is_reactive(&self) -> bool {
        self.classes.is_reactive()
    }

    pub(crate) fn touch_reactive_dependencies(&self) {
        self.classes.touch_reactive_dependencies();
    }
}

/// Builder for [`Classes`].
#[derive(Clone, Debug, Default)]
pub struct ClassesBuilder {
    classes: ClassList,
}

impl ClassesBuilder {
    /// Adds one always-active class token to the builder.
    ///
    /// Validation policy matches [`Classes::add`].
    #[must_use]
    pub fn with(mut self, name: impl Into<ClassName>) -> Self {
        self.classes
            .add_single(name.into(), ClassCondition::always());
        self
    }

    /// Adds one reactive class token, controlled by `when`.
    ///
    /// Validation policy for `name` matches [`Classes::add`]. See
    /// [`Classes::add_reactive`] for the list of accepted `when` shapes.
    #[must_use]
    pub fn with_reactive(
        mut self,
        name: impl Into<ClassName>,
        when: impl Into<ClassCondition>,
    ) -> Self {
        self.classes.add_single(name.into(), when.into());
        self
    }

    /// Adds multiple always-active class tokens to the builder. Same short-circuit panic
    /// behavior as [`Classes::add_all`].
    #[must_use]
    pub fn with_all<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ClassName>,
    {
        for name in iter {
            self.classes
                .add_single(name.into(), ClassCondition::always());
        }
        self
    }

    /// Splits `input` on Unicode whitespace ([`str::split_whitespace`]) and adds each non-empty
    /// token as an always-active class entry. Empty or whitespace-only input is a no-op. Same
    /// duplicate-panic and whitespace-semantic behavior as [`Classes::add_parsed`].
    #[must_use]
    pub fn with_parsed(self, input: &str) -> Self {
        self.with_all(input.split_whitespace().map(str::to_owned))
    }

    /// Adds a pair of mutually exclusive reactive classes. Mirrors [`Classes::add_toggle`].
    ///
    /// See [`Classes::add_reactive`] for the list of accepted `when` shapes.
    ///
    /// # Example
    /// ```rust
    /// use leptos::prelude::*;
    /// use leptos_classes::Classes;
    ///
    /// let (is_active, _) = signal(true);
    /// let classes = Classes::builder()
    ///     .with_toggle(is_active, "active", "inactive")
    ///     .build();
    /// ```
    #[must_use]
    pub fn with_toggle(
        mut self,
        when: impl Into<ClassCondition>,
        when_true: impl Into<ClassName>,
        when_false: impl Into<ClassName>,
    ) -> Self {
        self.classes
            .add_toggle(when.into(), when_true.into(), when_false.into());
        self
    }

    /// Merges another `Classes` value into this builder. See [`Classes::merge`] for semantics
    /// and the [`MergeStrategy`] variants.
    ///
    /// Prefer [`MergeStrategy::default()`] (which is
    /// [`UnionConditions`](MergeStrategy::UnionConditions)) unless you specifically need
    /// [`KeepSelf`](MergeStrategy::KeepSelf) or
    /// [`PanicOnConflict`](MergeStrategy::PanicOnConflict). It is the only strategy that never
    /// panics on caller input and never silently discards a caller-supplied condition.
    #[must_use]
    pub fn with_merged(mut self, other: Classes, strategy: MergeStrategy) -> Self {
        self.classes.merge(other.classes, strategy);
        self
    }

    /// Builds the configured [`Classes`].
    #[must_use]
    pub fn build(self) -> Classes {
        Classes {
            classes: self.classes,
        }
    }
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;
    use leptos::prelude::{Get, Set, signal};

    use crate::condition::ClassCondition;
    use crate::{Classes, MergeStrategy};

    mod construction {
        use super::*;

        #[test]
        fn single_str_renders_token() {
            let classes: Classes = "foo".into();
            assert_that!(classes.to_class_string()).is_equal_to("foo");
        }

        #[test]
        fn new_renders_nothing() {
            let classes = Classes::new();
            assert_that!(classes.to_class_string()).is_equal_to(String::new());
        }

        #[test]
        fn add_chain_appends_tokens_in_order() {
            let classes = Classes::new().add("foo").add("bar");
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn builder_with_chain_accumulates() {
            let classes = Classes::builder().with("foo").with("bar").build();
            assert_that!(classes.to_class_string()).is_equal_to("foo bar");
        }

        #[test]
        fn extends_across_chained_layers() {
            let initial: Classes = "base".into();
            let extended = initial.add("extended");
            let final_classes = extended.add("final");
            assert_that!(final_classes.to_class_string())
                .is_equal_to("base extended final".to_string());
        }

        #[test]
        fn add_all_accepts_iterator() {
            let classes = Classes::new().add_all(vec!["foo", "bar"]);
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn with_all_accepts_iterator() {
            let classes = Classes::builder().with_all(vec!["foo", "bar"]).build();
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn from_tuple_with_bool_true_renders_token() {
            let classes: Classes = ("foo", true).into();
            assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
        }

        #[test]
        fn from_tuple_with_bool_false_renders_nothing() {
            let classes: Classes = ("foo", false).into();
            assert_that!(classes.to_class_string()).is_equal_to(String::new());
        }

        #[test]
        fn with_reactive_mix_renders_only_active_entries() {
            let classes = Classes::builder()
                .with_reactive("always", true)
                .with_reactive("never", false)
                .with_reactive("also-always", true)
                .build();
            assert_that!(classes.to_class_string()).is_equal_to("always also-always".to_string());
        }
    }

    mod toggle {
        use super::*;

        #[test]
        fn renders_true_branch_when_active() {
            let classes = Classes::new().add_toggle(true, "active", "inactive");
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn renders_false_branch_when_inactive() {
            let classes = Classes::new().add_toggle(false, "active", "inactive");
            assert_that!(classes.to_class_string()).is_equal_to("inactive".to_string());
        }

        #[test]
        fn static_bool_true_is_not_reactive() {
            let classes = Classes::new().add_toggle(true, "active", "inactive");
            assert_that!(classes.is_reactive()).is_false();
        }

        #[test]
        fn static_bool_false_is_not_reactive() {
            let classes = Classes::new().add_toggle(false, "active", "inactive");
            assert_that!(classes.is_reactive()).is_false();
        }

        #[test]
        fn chained_with_add_keeps_order() {
            let classes = Classes::from("base")
                .add_toggle(true, "on", "off")
                .add("extra");
            assert_that!(classes.to_class_string()).is_equal_to("base on extra".to_string());
        }

        #[test]
        fn builder_renders_true_branch() {
            let classes = Classes::builder()
                .with("base")
                .with_toggle(true, "on", "off")
                .build();
            assert_that!(classes.to_class_string()).is_equal_to("base on".to_string());
        }

        #[test]
        fn builder_renders_false_branch() {
            let classes = Classes::builder().with_toggle(false, "on", "off").build();
            assert_that!(classes.to_class_string()).is_equal_to("off".to_string());
        }
    }

    mod parsing {
        use super::*;

        #[test]
        fn empty_or_whitespace_only_yields_empty() {
            assert_that!(Classes::parse("").to_class_string()).is_equal_to(String::new());
            assert_that!(Classes::parse("   \t\n").to_class_string()).is_equal_to(String::new());
        }

        #[test]
        fn multiple_tokens_preserve_order() {
            let classes = Classes::parse("btn btn-primary btn-large");
            assert_that!(classes.to_class_string())
                .is_equal_to("btn btn-primary btn-large".to_string());
        }

        #[test]
        fn collapses_mixed_whitespace_separators() {
            let classes = Classes::parse("  foo\tbar\n\nbaz   ");
            assert_that!(classes.to_class_string()).is_equal_to("foo bar baz".to_string());
        }

        #[test]
        fn splits_on_non_breaking_space() {
            // U+00A0 NO-BREAK SPACE sneaks in when text is pasted from rich-text sources.
            // `parse` splits on Unicode whitespace, so this separates tokens cleanly instead of
            // landing as one whitespace-bearing token that would then fail validation.
            let classes = Classes::parse("foo\u{00A0}bar");
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn splits_on_mixed_ascii_and_unicode_whitespace() {
            // ASCII space, NBSP, and line separator (U+2028) all separate tokens.
            let classes = Classes::parse("foo bar\u{00A0}baz\u{2028}qux");
            assert_that!(classes.to_class_string()).is_equal_to("foo bar baz qux".to_string());
        }

        #[test]
        fn unicode_whitespace_only_yields_empty() {
            assert_that!(Classes::parse("\u{00A0}\u{2028}").to_class_string())
                .is_equal_to(String::new());
        }

        #[test]
        fn result_is_not_reactive() {
            let classes = Classes::parse("foo bar");
            assert_that!(classes.is_reactive()).is_false();
        }

        #[test]
        fn add_parsed_appends_to_existing() {
            let classes = Classes::from("base").add_parsed("primary large");
            assert_that!(classes.to_class_string()).is_equal_to("base primary large".to_string());
        }

        #[test]
        fn add_parsed_chains_with_add_and_toggle() {
            let classes = Classes::from("base")
                .add_parsed("middle tail")
                .add("extra")
                .add_toggle(true, "on", "off");
            assert_that!(classes.to_class_string())
                .is_equal_to("base middle tail extra on".to_string());
        }

        #[test]
        fn add_parsed_empty_input_is_noop() {
            let classes = Classes::from("base").add_parsed("");
            assert_that!(classes.to_class_string()).is_equal_to("base".to_string());
        }

        #[test]
        fn with_parsed_in_builder() {
            let classes = Classes::builder()
                .with("base")
                .with_parsed("middle tail")
                .with_reactive("extra", true)
                .build();
            assert_that!(classes.to_class_string())
                .is_equal_to("base middle tail extra".to_string());
        }

        #[test]
        fn mixing_parsed_with_reactive_entry_makes_list_reactive() {
            let (is_active, set_is_active) = signal(true);
            let classes = Classes::parse("base middle").add_reactive("trailing", is_active);

            assert_that!(classes.is_reactive()).is_true();
            assert_that!(classes.to_class_string()).is_equal_to("base middle trailing".to_string());

            set_is_active.set(false);
            assert_that!(classes.to_class_string()).is_equal_to("base middle".to_string());
        }

        #[test]
        #[should_panic(expected = "was registered with Classes more than once")]
        fn parse_panics_on_intra_input_duplicate() {
            let _ = Classes::parse("foo foo");
        }

        #[test]
        #[should_panic(expected = "was registered with Classes more than once")]
        fn add_parsed_panics_on_collision_with_existing_entry() {
            let _ = Classes::from("base").add_parsed("base extra");
        }
    }

    mod reactivity {
        use super::*;

        #[test]
        fn signal_flip_updates_active_entry() {
            let (is_active, set_is_active) = signal(true);
            let classes = Classes::from(("active", is_active));

            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());

            set_is_active.set(false);
            assert_that!(classes.to_class_string()).is_equal_to(String::new());
        }

        #[test]
        fn signal_flip_swaps_toggle_branch() {
            let (is_active, set_is_active) = signal(true);
            let classes = Classes::new().add_toggle(is_active, "active", "inactive");

            assert_that!(classes.is_reactive()).is_true();
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());

            set_is_active.set(false);
            assert_that!(classes.to_class_string()).is_equal_to("inactive".to_string());

            set_is_active.set(true);
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn closure_drives_toggle_reactivity() {
            let (is_active, set_is_active) = signal(true);
            let classes = Classes::new().add_toggle(move || is_active.get(), "active", "inactive");

            assert_that!(classes.is_reactive()).is_true();
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());

            set_is_active.set(false);
            assert_that!(classes.to_class_string()).is_equal_to("inactive".to_string());
        }
    }

    mod validation {
        use super::*;

        #[test]
        #[should_panic(expected = "Class name is empty or whitespace-only")]
        fn empty_input_panics() {
            let _ = Classes::from("");
        }

        #[test]
        #[should_panic(expected = "Class name is empty or whitespace-only")]
        fn whitespace_only_input_panics() {
            let _ = Classes::builder().with("   ").build();
        }

        #[test]
        #[should_panic(expected = "Class names must not be whitespace-separated")]
        fn whitespace_separated_input_panics() {
            let _ = Classes::from("foo bar");
        }

        #[test]
        #[should_panic(expected = "Class names must not be whitespace-separated")]
        fn whitespace_around_input_panics() {
            let _ = Classes::from(" foo ");
        }

        #[test]
        #[should_panic(expected = "Class names must not be whitespace-separated")]
        fn non_breaking_space_inside_token_panics() {
            // Single-token construction must reject NBSP just like ASCII whitespace, otherwise
            // a token rendered into a `class` attribute would contain an unprintable space and
            // not match any CSS selector the user expects.
            let _ = Classes::from("foo\u{00A0}bar");
        }

        #[test]
        #[should_panic(expected = "Class name is empty or whitespace-only")]
        fn unicode_whitespace_only_input_panics() {
            // NBSP alone classifies as whitespace-only under the Unicode definition.
            let _ = Classes::from("\u{00A0}\u{00A0}");
        }

        #[test]
        #[should_panic(expected = "Class name is empty or whitespace-only")]
        fn add_with_empty_panics() {
            let _ = Classes::from("base").add("");
        }

        #[test]
        #[should_panic(expected = "Class name is empty or whitespace-only")]
        fn toggle_branch_empty_panics() {
            let _ = Classes::from("base").add_toggle(false, "active", "");
        }

        #[test]
        #[should_panic(expected = "Class name is empty or whitespace-only")]
        fn add_all_panics_on_first_invalid_item() {
            let _ = Classes::new().add_all(["foo", ""]);
        }

        #[test]
        #[should_panic(expected = "add_toggle requires two distinct branch names")]
        fn add_toggle_with_identical_branches_panics() {
            let _ = Classes::new().add_toggle(true, "foo", "foo");
        }
    }

    mod rendering {
        use super::*;

        #[test]
        fn only_writes_active_classes() {
            let (is_active, set_active) = signal(false);
            let classes = Classes::builder()
                .with_reactive("never", ClassCondition::never())
                .with_reactive("always", ClassCondition::always())
                .with_reactive("sometimes", ClassCondition::when_signal(is_active))
                .build();

            let mut rendered = String::new();
            classes.write_active_classes(&mut rendered);
            assert_that!(rendered).is_equal_to("always");

            set_active.set(true);
            let mut rendered = String::new();
            classes.write_active_classes(&mut rendered);
            assert_that!(rendered).is_equal_to("always sometimes");
        }

        #[test]
        fn write_appends_to_non_empty_buffer_with_separator() {
            let classes = Classes::builder().with("foo").with("bar").build();

            let mut rendered = String::from("existing");
            classes.write_active_classes(&mut rendered);
            assert_that!(rendered).is_equal_to("existing foo bar");
        }

        #[test]
        fn no_entries_skips_separator() {
            let classes = Classes::new();
            let mut rendered = String::from("existing");
            classes.write_active_classes(&mut rendered);
            assert_that!(rendered).is_equal_to("existing");
        }

        #[test]
        fn all_inactive_skips_separator() {
            let classes = Classes::from(("inactive", false));
            let mut rendered = String::from("existing");
            classes.write_active_classes(&mut rendered);
            assert_that!(rendered).is_equal_to("existing");
        }
    }

    mod merge {
        use super::*;

        /// Guards the recommended default in [`MergeStrategy`]'s docs and the
        /// `MergeStrategy::default()` call in `Classes::merge`'s doctest: if the `#[default]`
        /// attribute ever moves to a different variant, this test fires before the docs go stale.
        #[test]
        fn default_strategy_is_union_conditions() {
            assert_that!(MergeStrategy::default()).is_equal_to(MergeStrategy::UnionConditions);
        }

        mod using_the_panic_on_conflict_strategy {
            use super::*;

            mod without_collisions {
                use super::*;

                #[test]
                fn non_overlapping_appends_in_order() {
                    let a = Classes::from("foo");
                    let b = Classes::from("bar");
                    let merged = a.merge(b, MergeStrategy::PanicOnConflict);
                    assert_that!(merged.to_class_string()).is_equal_to("foo bar".to_string());
                }

                #[test]
                fn empty_other_is_identity() {
                    let a = Classes::from("foo");
                    let merged = a.merge(Classes::new(), MergeStrategy::PanicOnConflict);
                    assert_that!(merged.to_class_string()).is_equal_to("foo".to_string());
                }

                #[test]
                fn empty_self_yields_other() {
                    let merged =
                        Classes::new().merge(Classes::from("foo"), MergeStrategy::PanicOnConflict);
                    assert_that!(merged.to_class_string()).is_equal_to("foo".to_string());
                }

                #[test]
                fn preserves_reactivity_from_other() {
                    let (is_active, set_active) = signal(true);
                    let a = Classes::from("base");
                    let b = Classes::from(("active", is_active));
                    let merged = a.merge(b, MergeStrategy::PanicOnConflict);

                    assert_that!(merged.is_reactive()).is_true();
                    assert_that!(merged.to_class_string()).is_equal_to("base active");
                    set_active.set(false);
                    assert_that!(merged.to_class_string()).is_equal_to("base");
                }

                #[test]
                fn preserves_non_colliding_toggle_from_other() {
                    let (is_active, set_active) = signal(true);
                    let a = Classes::from("base");
                    let b = Classes::new().add_toggle(is_active, "on", "off");
                    let merged = a.merge(b, MergeStrategy::PanicOnConflict);

                    assert_that!(merged.to_class_string()).is_equal_to("base on");
                    set_active.set(false);
                    assert_that!(merged.to_class_string()).is_equal_to("base off");
                }
            }

            mod with_collisions {
                use super::*;

                /// Builds the exact panic message `panic_duplicate` emits for `token`.
                /// Kept inline so the tests assert against a string that is independent of any
                /// helper in `src/class_list.rs` (defense against a silent message rewording).
                fn duplicate_message(token: &str) -> String {
                    format!(
                        "class token `{token}` was registered with Classes more than \
                         once. Each class name may appear in at most one entry; \
                         combine conditions instead (e.g. add_reactive(\"{token}\", \
                         move || a.get() || b.get()))."
                    )
                }

                #[test]
                fn panics_on_single_collision() {
                    let a = Classes::from("foo");
                    let b = Classes::from("foo");
                    assert_that_panic_by(|| a.merge(b, MergeStrategy::PanicOnConflict))
                        .has_type::<String>()
                        .is_equal_to(duplicate_message("foo"));
                }

                #[test]
                fn panics_on_toggle_half_collision() {
                    let a = Classes::new().add_toggle(true, "on", "off");
                    let b = Classes::from("on");
                    assert_that_panic_by(|| a.merge(b, MergeStrategy::PanicOnConflict))
                        .has_type::<String>()
                        .is_equal_to(duplicate_message("on"));
                }
            }
        }

        mod using_the_keep_self_strategy {
            use super::*;

            #[test]
            fn reactivity_is_preserved_and_only_depends_on_own_classes() {
                let (is_active, _) = signal(false);
                let a = Classes::from("foo");
                let b = Classes::from(("foo", is_active)).add("bar");
                let merged = a.merge(b, MergeStrategy::KeepSelf);

                assert_that!(merged.to_class_string()).is_equal_to("foo bar");
                assert_that!(merged.is_reactive()).is_false();
            }

            #[test]
            fn preserves_self_toggle_against_other_collision() {
                let (is_active, set_active) = signal(true);
                let a = Classes::new().add_toggle(is_active, "on", "off");
                let b = Classes::from("on");
                let merged = a.merge(b, MergeStrategy::KeepSelf);

                assert_that!(merged.is_reactive()).is_true();
                assert_that!(merged.to_class_string()).is_equal_to("on");
                set_active.set(false);
                assert_that!(merged.to_class_string()).is_equal_to("off");
            }
        }

        mod using_the_union_conditions_strategy {
            use super::*;

            #[test]
            fn or_connects_conditions_rendering_when_either_signal_is_true() {
                let (a_sig, set_a_sig) = signal(false);
                let (b_sig, set_b_sig) = signal(false);
                let a = Classes::from(("foo", a_sig));
                let b = Classes::from(("foo", b_sig));
                let merged = a.merge(b, MergeStrategy::UnionConditions);

                assert_that!(merged.is_reactive()).is_true();
                assert_that!(merged.to_class_string()).is_equal_to("");
                set_a_sig.set(true);
                assert_that!(merged.to_class_string()).is_equal_to("foo");
                set_a_sig.set(false);
                set_b_sig.set(true);
                assert_that!(merged.to_class_string()).is_equal_to("foo");
                set_a_sig.set(true);
                assert_that!(merged.to_class_string()).is_equal_to("foo");
            }

            #[test]
            fn always_collapses_to_always() {
                let (is_active, set_active) = signal(false);
                let a = Classes::from("foo");
                let b = Classes::from(("foo", is_active));
                let merged = a.merge(b, MergeStrategy::UnionConditions);

                assert_that!(merged.is_reactive()).is_false();
                assert_that!(merged.to_class_string()).is_equal_to("foo");
                set_active.set(true);
                assert_that!(merged.to_class_string()).is_equal_to("foo");
            }
        }

        mod in_builder_chain {
            use super::*;

            #[test]
            fn with_merged_merges_classes() {
                let merged = Classes::builder()
                    .with("base")
                    .with_merged(Classes::from("extra"), MergeStrategy::default())
                    .with("tail")
                    .build();
                assert_that!(merged.to_class_string()).is_equal_to("base extra tail");
            }
        }
    }
}

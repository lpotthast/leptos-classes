use leptos::typed_builder::TypedBuilder;

use crate::{
    ClassList,
    class_entry::{ClassEntry, ClassName, IntoClassCondition, ToggleEntry},
};

/// Leptos component-prop-utility to drill down a list of classes.
///
/// # Duplicate Handling
///
/// Duplicate class names are allowed and will not be deduplicated. However, in debug builds,
/// a warning will be logged when the same class name is added multiple times. This behavior
/// helps identify potential bugs where classes are unintentionally added twice.
///
/// The final class string produced by [`Classes::to_class_string`] may contain repeated
/// class names if duplicates were added.
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
/// /// The lowest-level component renders the class-list into an actual HTML element.
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
/// /// Root component defines the initial classes using a builder pattern.
/// #[component]
/// fn ProvidingClasses() -> impl IntoView {
///     let (show_second, _) = signal(true);
///     view! {
///         <ExtendingClasses classes="single-class"/>
///         <ExtendingClasses classes=Classes::builder()
///             .with("first")
///             .with(("second", show_second))
///             .build()/>
///     }
/// }
/// ```
#[derive(Clone, Debug, Default, TypedBuilder)]
#[builder(mutators(
    pub fn with(&mut self, class: impl Into<ClassEntry>) {
        self.classes.push(class.into());
    }
    pub fn with_all<C: Into<ClassEntry>>(&mut self, iter: impl IntoIterator<Item = C>) {
        for class in iter {
            self.classes.push(class.into());
        }
    }
    /// Add a pair of mutually exclusive reactive classes.
    ///
    /// The `when_true` class is active when the signal is `true`,
    /// the `when_false` class is active when the signal is `false`.
    pub fn with_toggle(&mut self, when: impl IntoClassCondition, when_true: impl Into<ClassName>, when_false: impl Into<ClassName>) {
        self.classes
            .push(ToggleEntry::new(when, when_true, when_false));
    }
))]
pub struct Classes {
    #[builder(via_mutators)]
    pub(crate) classes: ClassList,
}

impl Classes {
    #[must_use]
    pub fn new() -> Self {
        Self {
            classes: ClassList::default(),
        }
    }

    /// Add one additional class to this class-list.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, entry: impl Into<ClassEntry>) -> Self {
        self.classes.push(entry.into());
        self
    }

    /// Add multiple classes to this class-list in place.
    pub fn add_all<C: Into<ClassEntry>>(&mut self, iter: impl IntoIterator<Item = C>) {
        for class in iter {
            self.classes.push(class.into());
        }
    }

    /// Add a pair of mutually exclusive reactive classes.
    ///
    /// The `when_true` class is active when the signal is `true`,
    /// the `when_false` class is active when the signal is `false`.
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
        when: impl IntoClassCondition,
        when_true: impl Into<ClassName>,
        when_false: impl Into<ClassName>,
    ) -> Self {
        self.classes
            .push(ToggleEntry::new(when, when_true, when_false));
        self
    }

    /// Appends all active classes to the given string buffer.
    ///
    /// This method is zero-allocation when the buffer has sufficient capacity.
    pub fn write_class_string(&self, buf: &mut String) {
        self.write_active_classes(buf);
    }

    /// Reactively combines all defined classes into one space-separated String.
    ///
    /// Note: Prefer using `class=classes` directly (via `IntoClass`) for better performance,
    /// as it reuses the string buffer across reactive updates.
    #[must_use]
    pub fn to_class_string(&self) -> String {
        let mut s = String::new();
        self.write_class_string(&mut s);
        s
    }

    pub(crate) fn write_active_classes(&self, buf: &mut String) {
        self.classes.write_active_classes(buf);
    }

    pub(crate) fn html_len_estimate(&self) -> usize {
        self.classes.html_len_estimate()
    }

    pub(crate) fn is_reactive(&self) -> bool {
        self.classes.is_reactive()
    }

    pub(crate) fn touch_reactive_dependencies(&self) {
        self.classes.touch_reactive_dependencies();
    }
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;
    use leptos::prelude::{Set, signal};

    use super::Classes;

    #[test]
    fn test_single_class() {
        let classes: Classes = "foo".into();
        assert_that!(classes.to_class_string()).is_equal_to("foo");
    }

    #[test]
    fn test_multiple_classes() {
        let classes = Classes::builder().with("foo").with("bar").build();
        assert_that!(classes.to_class_string()).is_equal_to("foo bar");
    }

    #[test]
    fn test_conditional_class_true() {
        let classes: Classes = ("foo", true).into();
        assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
    }

    #[test]
    fn test_conditional_class_false() {
        let classes: Classes = ("foo", false).into();
        assert_that!(classes.to_class_string()).is_equal_to(String::new());
    }

    #[test]
    fn test_mixed_conditional_classes() {
        let classes = Classes::builder()
            .with(("always", true))
            .with(("never", false))
            .with(("also-always", true))
            .build();
        assert_that!(classes.to_class_string()).is_equal_to("always also-always".to_string());
    }

    #[test]
    fn test_add_method() {
        let classes = Classes::new().add("foo").add("bar");
        assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
    }

    #[test]
    fn test_empty_classes() {
        let classes = Classes::new();
        assert_that!(classes.to_class_string()).is_equal_to(String::new());
    }

    #[test]
    fn test_drilling_down() {
        let initial: Classes = "base".into();
        let extended = initial.add("extended");
        let final_classes = extended.add("final");
        assert_that!(final_classes.to_class_string())
            .is_equal_to("base extended final".to_string());
    }

    #[test]
    fn test_add_toggle_when_true() {
        let classes = Classes::new().add_toggle(true, "active", "inactive");
        assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
    }

    #[test]
    fn test_add_toggle_bool_true_is_static() {
        let classes = Classes::new().add_toggle(true, "active", "inactive");
        assert_that!(classes.is_reactive()).is_false();
    }

    #[test]
    fn test_add_toggle_when_false() {
        let classes = Classes::new().add_toggle(false, "active", "inactive");
        assert_that!(classes.to_class_string()).is_equal_to("inactive".to_string());
    }

    #[test]
    fn test_add_toggle_bool_false_is_static() {
        let classes = Classes::new().add_toggle(false, "active", "inactive");
        assert_that!(classes.is_reactive()).is_false();
    }

    #[test]
    fn test_add_toggle_chained() {
        let classes = Classes::from("base")
            .add_toggle(true, "on", "off")
            .add("extra");
        assert_that!(classes.to_class_string()).is_equal_to("base on extra".to_string());
    }

    #[test]
    fn test_with_toggle_builder() {
        let classes = Classes::builder()
            .with("base")
            .with_toggle(true, "on", "off")
            .build();
        assert_that!(classes.to_class_string()).is_equal_to("base on".to_string());
    }

    #[test]
    fn test_with_toggle_builder_false() {
        let classes = Classes::builder().with_toggle(false, "on", "off").build();
        assert_that!(classes.to_class_string()).is_equal_to("off".to_string());
    }

    #[test]
    fn test_add_all_accepts_into_iterator() {
        let mut classes = Classes::new();
        classes.add_all(vec!["foo", "bar"]);
        assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
    }

    #[test]
    fn test_with_all_accepts_into_iterator() {
        let classes = Classes::builder().with_all(vec!["foo", "bar"]).build();
        assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
    }

    #[test]
    fn test_write_class_string_appends_with_space() {
        let classes = Classes::builder().with("foo").with("bar").build();
        let mut rendered = String::from("existing");

        classes.write_class_string(&mut rendered);

        assert_that!(rendered).is_equal_to("existing foo bar".to_string());
    }

    #[test]
    fn test_write_class_string_skips_space_when_no_active_classes() {
        let classes = Classes::from(("inactive", false));
        let mut rendered = String::from("existing");

        classes.write_class_string(&mut rendered);

        assert_that!(rendered).is_equal_to("existing".to_string());
    }

    #[test]
    fn test_reactive_entry_rerenders_after_signal_change() {
        let (is_active, set_is_active) = signal(true);
        let classes = Classes::from(("active", is_active));

        assert_that!(classes.to_class_string()).is_equal_to("active".to_string());

        set_is_active.set(false);

        assert_that!(classes.to_class_string()).is_equal_to(String::new());
    }

    #[test]
    fn test_toggle_rerenders_after_signal_change() {
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
    fn test_empty_class_name_is_ignored() {
        let classes = Classes::from("");
        assert_that!(classes.to_class_string()).is_equal_to(String::new());
    }

    #[test]
    fn test_whitespace_only_class_name_is_ignored() {
        let classes = Classes::builder().with("   ").build();
        assert_that!(classes.to_class_string()).is_equal_to(String::new());
    }

    #[test]
    fn test_adding_empty_class_name_preserves_spacing() {
        let classes = Classes::from("base").add("");
        assert_that!(classes.to_class_string()).is_equal_to("base".to_string());
    }

    #[test]
    fn test_empty_toggle_branch_is_ignored_without_trailing_space() {
        let classes = Classes::from("base").add_toggle(false, "active", "");
        assert_that!(classes.to_class_string()).is_equal_to("base".to_string());
    }

    #[test]
    fn test_empty_entries_do_not_break_spacing_between_valid_classes() {
        let classes = Classes::builder()
            .with("base")
            .with(" ")
            .with(("middle", true))
            .with("")
            .with("tail")
            .build();
        assert_that!(classes.to_class_string()).is_equal_to("base middle tail".to_string());
    }
}

// Coherence invariant: the slice and array impls fall into two pairs - "name only" (`&[N]`,
// `[N; M]`) and "name + condition" (`&[(N, C)]`, `[(N, C); M]`). The pairs coexist only because no
// `From<(_, _)> for ClassName` impl exists in the crate. Adding such an impl would make the plain
// and tuple variants overlap.

use std::borrow::Cow;

use crate::Classes;
use crate::class_name::ClassName;
use crate::condition::ClassCondition;

/// Creates a `Classes` containing a single always-active class name from a `&'static str`.
///
/// The input is treated as **one** class name. Panics if the string is empty, whitespace-only,
/// or contains any whitespace (Unicode definition). See [`Classes::add`] for the validation
/// policy.
///
/// For a runtime string that may contain multiple whitespace-separated tokens, use
/// [`Classes::parse`] instead. For runtime input you want to handle without a panic, pre-validate
/// with [`ClassName::try_new`].
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos_classes::Classes;
///
/// let c: Classes = "btn-primary".into();
/// assert_that!(c.to_class_string()).is_equal_to("btn-primary");
/// ```
impl From<&'static str> for Classes {
    fn from(name: &'static str) -> Self {
        Classes::new().add(name)
    }
}

/// Creates a `Classes` containing a single always-active class token from an owned `String`.
///
/// Same validation as [`Classes::add`]. Use [`Classes::parse`] for a runtime `String` that may
/// contain whitespace-separated tokens.
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos_classes::Classes;
///
/// let c: Classes = String::from("btn-primary").into();
/// assert_that!(c.to_class_string()).is_equal_to("btn-primary");
/// ```
impl From<String> for Classes {
    fn from(name: String) -> Self {
        Classes::new().add(name)
    }
}

/// Creates a `Classes` containing a single always-active class token from a `Cow<'static, str>`.
///
/// Same validation as [`Classes::add`]. Useful when the caller already holds a `Cow`,
/// e.g. from a configuration lookup. Use [`Classes::parse`] for runtime input that may contain
/// whitespace-separated tokens.
///
/// # Examples
/// ```
/// use std::borrow::Cow;
/// use assertr::prelude::*;
/// use leptos_classes::Classes;
///
/// let c: Classes = Cow::Borrowed("btn-primary").into();
/// assert_that!(c.to_class_string()).is_equal_to("btn-primary");
/// ```
impl From<Cow<'static, str>> for Classes {
    fn from(name: Cow<'static, str>) -> Self {
        Classes::new().add(name)
    }
}

/// Creates a `Classes` containing a single always-active entry from a pre-validated [`ClassName`].
///
/// Use this when you constructed a [`ClassName`] via [`ClassName::try_new`] (the non-panicking
/// constructor); the conversion itself cannot fail.
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos_classes::{ClassName, Classes};
///
/// let name = ClassName::try_new("btn-primary").unwrap();
/// let c: Classes = name.into();
/// assert_that!(c.to_class_string()).is_equal_to("btn-primary");
/// ```
impl From<ClassName> for Classes {
    fn from(name: ClassName) -> Self {
        Classes::new().add(name)
    }
}

/// Creates a `Classes` from a slice of class tokens, each added as an always-active entry.
///
/// Each element is validated independently per [`Classes::add`]: any invalid token panics.
/// Duplicate tokens within the slice also panic; see the `# Duplicate Handling` section on
/// [`Classes`].
///
/// `N` must be `Clone` because the impl clones each element during conversion. All four
/// built-in `Into<ClassName>` sources (`&'static str`, `String`, `Cow<'static, str>`,
/// `ClassName`) satisfy this.
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos_classes::Classes;
///
/// let names: &[&'static str] = &["btn", "btn-primary"];
/// let c: Classes = names.into();
/// assert_that!(c.to_class_string()).is_equal_to("btn btn-primary");
/// ```
impl<N: Clone + Into<ClassName>> From<&[N]> for Classes {
    fn from(names: &[N]) -> Self {
        Classes::new().add_all(names.iter().cloned())
    }
}

/// Creates a `Classes` from an array of class tokens, each added as an always-active entry.
///
/// Each element is validated independently per [`Classes::add`]: any invalid token panics.
/// Duplicate tokens within the array also panic; see the `# Duplicate Handling` section on
/// [`Classes`].
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos_classes::Classes;
///
/// let c: Classes = ["btn", "btn-primary", "btn-large"].into();
/// assert_that!(c.to_class_string()).is_equal_to("btn btn-primary btn-large");
/// ```
impl<N: Into<ClassName>, const M: usize> From<[N; M]> for Classes {
    fn from(names: [N; M]) -> Self {
        Classes::new().add_all(names)
    }
}

/// Creates a `Classes` containing a single reactive class entry from a `(name, condition)` tuple.
///
/// `name` is validated per [`Classes::add`]; see [`Classes::add_reactive`] for the accepted
/// condition shapes.
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos::prelude::*;
/// use leptos_classes::Classes;
///
/// let (is_active, _) = signal(true);
/// let c: Classes = ("active", is_active).into();
/// assert_that!(c.to_class_string()).is_equal_to("active");
/// ```
impl<N: Into<ClassName>, C: Into<ClassCondition>> From<(N, C)> for Classes {
    fn from((name, when): (N, C)) -> Self {
        Classes::new().add_reactive(name, when)
    }
}

/// Creates a `Classes` from a slice of `(name, condition)` pairs, each added as a reactive entry.
///
/// Names are validated per [`Classes::add`]; see [`Classes::add_reactive`] for the accepted
/// condition shapes. Duplicate tokens within the slice also panic; see the `# Duplicate Handling`
/// section on [`Classes`].
///
/// Both `N` and `C` must be `Clone` because the impl clones each element during conversion.
/// All five built-in `Into<ClassCondition>` sources (`bool`, `Signal<bool>`, `ReadSignal<bool>`,
/// `RwSignal<bool>`, `Memo<bool>`) are `Clone`; bare closures are not and must be placed in an
/// array (`[(N, C); M]`) instead.
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos::prelude::*;
/// use leptos_classes::Classes;
///
/// let (is_first, _) = signal(true);
/// let (is_second, _) = signal(false);
/// let entries: &[(&'static str, ReadSignal<bool>)] =
///     &[("first", is_first), ("second", is_second)];
/// let c: Classes = entries.into();
/// assert_that!(c.to_class_string()).is_equal_to("first");
/// ```
impl<N: Clone + Into<ClassName>, C: Clone + Into<ClassCondition>> From<&[(N, C)]> for Classes {
    fn from(entries: &[(N, C)]) -> Self {
        let mut classes = Classes::new();
        for (name, when) in entries.iter().cloned() {
            classes = classes.add_reactive(name, when);
        }
        classes
    }
}

/// Creates a `Classes` from an array of `(name, condition)` pairs, each added as a reactive entry.
///
/// Names are validated per [`Classes::add`]; see [`Classes::add_reactive`] for the accepted
/// condition shapes. Duplicate tokens within the array also panic; see the `# Duplicate Handling`
/// section on [`Classes`].
///
/// # Examples
/// ```
/// use assertr::prelude::*;
/// use leptos::prelude::*;
/// use leptos_classes::Classes;
///
/// let (is_first, _) = signal(true);
/// let (is_second, _) = signal(false);
/// let c: Classes = [("first", is_first), ("second", is_second)].into();
/// assert_that!(c.to_class_string()).is_equal_to("first");
/// ```
impl<N: Into<ClassName>, C: Into<ClassCondition>, const M: usize> From<[(N, C); M]> for Classes {
    fn from(entries: [(N, C); M]) -> Self {
        let mut classes = Classes::new();
        for (name, when) in entries {
            classes = classes.add_reactive(name, when);
        }
        classes
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use assertr::prelude::*;
    use leptos::prelude::{Get, Memo, ReadSignal, RwSignal, Set, signal};

    use crate::{ClassName, Classes};

    mod from_unconditional {
        use super::*;

        #[test]
        fn static_str_renders_token() {
            let classes: Classes = "foo".into();
            assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
        }

        #[test]
        fn string_renders_token() {
            let classes: Classes = String::from("foo").into();
            assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
        }

        #[test]
        fn cow_borrowed_renders_token() {
            let classes: Classes = Cow::Borrowed("foo").into();
            assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
        }

        #[test]
        fn cow_owned_renders_token() {
            let cow: Cow<'static, str> = Cow::Owned(String::from("foo"));
            let classes: Classes = cow.into();
            assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
        }

        #[test]
        fn class_name_renders_token() {
            let name = ClassName::try_new("foo").unwrap();
            let classes: Classes = name.into();
            assert_that!(classes.to_class_string()).is_equal_to("foo".to_string());
        }

        #[test]
        fn slice_of_static_str_renders_all_tokens_in_order() {
            let names: &[&'static str] = &["foo", "bar", "baz"];
            let classes: Classes = names.into();
            assert_that!(classes.to_class_string()).is_equal_to("foo bar baz".to_string());
        }

        #[test]
        fn slice_of_string_renders_all_tokens_in_order() {
            let names: &[String] = &[String::from("foo"), String::from("bar")];
            let classes: Classes = names.into();
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn array_of_static_str_renders_all_tokens_in_order() {
            let classes = Classes::from(["foo", "bar", "baz"]);
            assert_that!(classes.to_class_string()).is_equal_to("foo bar baz".to_string());
        }

        #[test]
        fn array_of_string_renders_all_tokens_in_order() {
            let classes = Classes::from([String::from("foo"), String::from("bar")]);
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn slice_of_class_name_renders_all_tokens_in_order() {
            let names: &[ClassName] = &[
                ClassName::try_new("foo").unwrap(),
                ClassName::try_new("bar").unwrap(),
            ];
            let classes: Classes = names.into();
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn array_of_class_name_renders_all_tokens_in_order() {
            let classes = Classes::from([
                ClassName::try_new("foo").unwrap(),
                ClassName::try_new("bar").unwrap(),
            ]);
            assert_that!(classes.to_class_string()).is_equal_to("foo bar".to_string());
        }

        #[test]
        fn empty_slice_yields_empty_classes() {
            let names: &[&'static str] = &[];
            let classes: Classes = names.into();
            assert_that!(classes.to_class_string()).is_equal_to(String::new());
        }

        #[test]
        fn empty_array_yields_empty_classes() {
            let names: [&'static str; 0] = [];
            let classes = Classes::from(names);
            assert_that!(classes.to_class_string()).is_equal_to(String::new());
        }
    }

    mod from_conditional {
        use super::*;

        // Each construction test verifies that a particular condition shape converts into
        // `Classes` and renders its initial value. Mutation-propagation lives in a single
        // shared test below: the runtime path is identical once a value reaches
        // `ClassCondition` (via any `From` impl).

        #[test]
        fn read_signal_renders_initial_active_token() {
            let (is_active, _) = signal(true);
            let classes = Classes::from(("active", is_active));
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn rw_signal_renders_initial_active_token() {
            let is_active = RwSignal::new(true);
            let classes = Classes::from(("active", is_active));
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn memo_renders_initial_active_token() {
            let backing = RwSignal::new(true);
            let memo = Memo::new(move |_| backing.get());
            let classes = Classes::from(("active", memo));
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn closure_renders_initial_active_token() {
            let (is_active, _) = signal(true);
            let classes = Classes::from(("active", move || is_active.get()));
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn string_name_with_signal_renders() {
            let (is_active, _) = signal(true);
            let classes = Classes::from((String::from("active"), is_active));
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }

        #[test]
        fn array_of_tuples_renders_active_entries_only() {
            let (is_first, _) = signal(true);
            let (is_second, _) = signal(false);
            let classes = Classes::from([("first", is_first), ("second", is_second)]);
            assert_that!(classes.to_class_string()).is_equal_to("first".to_string());
        }

        #[test]
        fn slice_of_tuples_renders_active_entries_only() {
            let (is_first, _) = signal(true);
            let (is_second, _) = signal(false);
            let entries: &[(&'static str, ReadSignal<bool>)] =
                &[("first", is_first), ("second", is_second)];
            let classes: Classes = entries.into();
            assert_that!(classes.to_class_string()).is_equal_to("first".to_string());
        }

        #[test]
        fn signal_mutation_propagates_to_render() {
            // One shared reactivity test. Other condition shapes (`RwSignal`, `Memo`,
            // closures) route through the same `When(Signal<bool>)` arm, so covering
            // `ReadSignal` covers them.
            let (is_active, set_is_active) = signal(true);
            let classes = Classes::from(("active", is_active));

            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());

            set_is_active.set(false);
            assert_that!(classes.to_class_string()).is_equal_to(String::new());

            set_is_active.set(true);
            assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
        }
    }
}

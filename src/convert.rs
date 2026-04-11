use leptos::prelude::Signal;
use reactive_graph::signal::ReadSignal;

use crate::{ClassEntry, Classes};

impl From<&'static str> for ClassEntry {
    fn from(s: &'static str) -> Self {
        ClassEntry::always(s)
    }
}

impl From<String> for ClassEntry {
    fn from(s: String) -> Self {
        ClassEntry::always(s)
    }
}

impl From<(&'static str, bool)> for ClassEntry {
    fn from((name, when): (&'static str, bool)) -> Self {
        ClassEntry::conditional(name, when)
    }
}

impl From<(&'static str, ReadSignal<bool>)> for ClassEntry {
    fn from((name, when): (&'static str, ReadSignal<bool>)) -> Self {
        let when: Signal<bool> = when.into();
        ClassEntry::reactive(name, when)
    }
}

impl From<(&'static str, Signal<bool>)> for ClassEntry {
    fn from((name, when): (&'static str, Signal<bool>)) -> Self {
        ClassEntry::reactive(name, when)
    }
}

impl From<(String, bool)> for ClassEntry {
    fn from((name, when): (String, bool)) -> Self {
        ClassEntry::conditional(name, when)
    }
}

impl From<(String, Signal<bool>)> for ClassEntry {
    fn from((name, when): (String, Signal<bool>)) -> Self {
        ClassEntry::reactive(name, when)
    }
}

impl From<(String, ReadSignal<bool>)> for ClassEntry {
    fn from((name, when): (String, ReadSignal<bool>)) -> Self {
        let when: Signal<bool> = when.into();
        ClassEntry::reactive(name, when)
    }
}

impl From<&'static str> for Classes {
    fn from(name: &'static str) -> Self {
        Classes::builder().with(name).build()
    }
}

impl From<String> for Classes {
    fn from(name: String) -> Self {
        Classes::builder().with(name).build()
    }
}

impl From<&[&'static str]> for Classes {
    fn from(names: &[&'static str]) -> Self {
        Classes::builder().with_all(names.iter().copied()).build()
    }
}

impl<const N: usize> From<[&'static str; N]> for Classes {
    fn from(names: [&'static str; N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

impl From<(&'static str, bool)> for Classes {
    fn from((name, when): (&'static str, bool)) -> Self {
        Classes::builder().with((name, when)).build()
    }
}

impl From<(&'static str, Signal<bool>)> for Classes {
    fn from((name, when): (&'static str, Signal<bool>)) -> Self {
        Classes::builder().with((name, when)).build()
    }
}

impl From<(&'static str, ReadSignal<bool>)> for Classes {
    fn from((name, when): (&'static str, ReadSignal<bool>)) -> Self {
        Classes::builder().with((name, when)).build()
    }
}

impl From<(String, bool)> for Classes {
    fn from((name, when): (String, bool)) -> Self {
        Classes::builder().with((name, when)).build()
    }
}

impl From<(String, Signal<bool>)> for Classes {
    fn from((name, when): (String, Signal<bool>)) -> Self {
        Classes::builder().with((name, when)).build()
    }
}

impl From<(String, ReadSignal<bool>)> for Classes {
    fn from((name, when): (String, ReadSignal<bool>)) -> Self {
        Classes::builder().with((name, when)).build()
    }
}

impl<const N: usize> From<[(&'static str, bool); N]> for Classes {
    fn from(names: [(&'static str, bool); N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

impl<const N: usize> From<[(&'static str, Signal<bool>); N]> for Classes {
    fn from(names: [(&'static str, Signal<bool>); N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

impl<const N: usize> From<[(&'static str, ReadSignal<bool>); N]> for Classes {
    fn from(names: [(&'static str, ReadSignal<bool>); N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

impl<const N: usize> From<[(String, bool); N]> for Classes {
    fn from(names: [(String, bool); N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

impl<const N: usize> From<[(String, Signal<bool>); N]> for Classes {
    fn from(names: [(String, Signal<bool>); N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

impl<const N: usize> From<[(String, ReadSignal<bool>); N]> for Classes {
    fn from(names: [(String, ReadSignal<bool>); N]) -> Self {
        Classes::builder().with_all(names).build()
    }
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;
    use leptos::prelude::{Set, signal};

    use crate::Classes;

    #[test]
    fn test_from_str_array() {
        let classes = Classes::from(["foo", "bar", "baz"]);
        assert_that!(classes.to_class_string()).is_equal_to("foo bar baz".to_string());
    }

    #[test]
    fn test_from_read_signal_array() {
        let (is_first, _) = leptos::prelude::signal(true);
        let (is_second, _) = leptos::prelude::signal(false);

        let classes = Classes::from([("first", is_first), ("second", is_second)]);

        assert_that!(classes.to_class_string()).is_equal_to("first".to_string());
    }

    #[test]
    fn test_from_string_read_signal() {
        let (is_active, _) = signal(true);

        let classes = Classes::from((String::from("active"), is_active));

        assert_that!(classes.to_class_string()).is_equal_to("active".to_string());
    }

    #[test]
    fn test_from_read_signal_array_rerenders_after_signal_change() {
        let (is_first, set_is_first) = signal(true);
        let (is_second, set_is_second) = signal(false);

        let classes = Classes::from([("first", is_first), ("second", is_second)]);

        assert_that!(classes.to_class_string()).is_equal_to("first".to_string());

        set_is_first.set(false);
        set_is_second.set(true);

        assert_that!(classes.to_class_string()).is_equal_to("second".to_string());
    }

    #[test]
    fn test_from_string_read_signal_rerenders_after_signal_change() {
        let (is_active, set_is_active) = signal(true);

        let classes = Classes::from((String::from("active"), is_active));

        assert_that!(classes.to_class_string()).is_equal_to("active".to_string());

        set_is_active.set(false);

        assert_that!(classes.to_class_string()).is_equal_to(String::new());
    }
}

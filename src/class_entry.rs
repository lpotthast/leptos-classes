use std::borrow::Cow;

use leptos::prelude::{Get, Signal};
use reactive_graph::signal::ReadSignal;

/// A class name that can be either a static string slice or an owned `String`.
///
/// Use `Cow<'static, str>` to avoid allocations for static class names while
/// still supporting dynamically constructed class names when needed.
pub type ClassName = Cow<'static, str>;

pub(crate) fn is_effectively_empty_class_name(name: &str) -> bool {
    name.trim().is_empty()
}

fn append_class_name(buf: &mut String, first: &mut bool, name: &str) {
    if is_effectively_empty_class_name(name) {
        return;
    }

    if !*first {
        buf.push(' ');
    }
    *first = false;
    buf.push_str(name);
}

fn html_len_estimate_for_name(name: &str) -> usize {
    if is_effectively_empty_class_name(name) {
        0
    } else {
        name.len() + 1
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct ClassCondition(ClassConditionKind);

#[derive(Clone, Debug)]
enum ClassConditionKind {
    Always,
    Never,
    When(Signal<bool>),
}

impl ClassCondition {
    pub(crate) fn always() -> Self {
        Self(ClassConditionKind::Always)
    }

    pub(crate) fn never() -> Self {
        Self(ClassConditionKind::Never)
    }

    pub(crate) fn when(when: impl Into<Signal<bool>>) -> Self {
        Self(ClassConditionKind::When(when.into()))
    }

    pub(crate) fn conditional(when: bool) -> Self {
        if when { Self::always() } else { Self::never() }
    }

    pub(crate) fn is_active(&self) -> bool {
        match &self.0 {
            ClassConditionKind::Always => true,
            ClassConditionKind::Never => false,
            ClassConditionKind::When(when) => when.get(),
        }
    }

    pub(crate) fn is_reactive(&self) -> bool {
        matches!(self.0, ClassConditionKind::When(_))
    }

    pub(crate) fn touch_reactive_dependency(&self) {
        if let ClassConditionKind::When(when) = &self.0 {
            let _ = when.get();
        }
    }
}

#[doc(hidden)]
pub trait IntoClassCondition {
    fn into_class_condition(self) -> ClassCondition;
}

impl IntoClassCondition for bool {
    fn into_class_condition(self) -> ClassCondition {
        ClassCondition::conditional(self)
    }
}

impl IntoClassCondition for Signal<bool> {
    fn into_class_condition(self) -> ClassCondition {
        ClassCondition::when(self)
    }
}

impl IntoClassCondition for ReadSignal<bool> {
    fn into_class_condition(self) -> ClassCondition {
        ClassCondition::when(self)
    }
}

#[derive(Clone, Debug)]
pub struct ClassEntry {
    name: ClassName,
    when: ClassCondition,
}

impl ClassEntry {
    pub fn reactive(name: impl Into<ClassName>, when: impl Into<Signal<bool>>) -> Self {
        Self {
            name: name.into(),
            when: ClassCondition::when(when),
        }
    }

    pub fn always(name: impl Into<ClassName>) -> Self {
        Self {
            name: name.into(),
            when: ClassCondition::always(),
        }
    }

    pub(crate) fn conditional(name: impl Into<ClassName>, when: bool) -> Self {
        Self {
            name: name.into(),
            when: ClassCondition::conditional(when),
        }
    }

    /// Returns a reference to the class name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ToggleEntry {
    when: ClassCondition,
    when_true: ClassName,
    when_false: ClassName,
}

impl ToggleEntry {
    pub(crate) fn new(
        when: impl IntoClassCondition,
        when_true: impl Into<ClassName>,
        when_false: impl Into<ClassName>,
    ) -> Self {
        Self {
            when: when.into_class_condition(),
            when_true: when_true.into(),
            when_false: when_false.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ClassItem {
    Entry(ClassEntry),
    Toggle(ToggleEntry),
}

impl ClassItem {
    pub(crate) fn append_active_classes(&self, buf: &mut String, first: &mut bool) {
        match self {
            Self::Entry(entry) => {
                if entry.when.is_active() {
                    append_class_name(buf, first, entry.name());
                }
            }
            Self::Toggle(entry) => {
                if entry.when.is_active() {
                    append_class_name(buf, first, &entry.when_true);
                } else {
                    append_class_name(buf, first, &entry.when_false);
                }
            }
        }
    }

    pub(crate) fn html_len_estimate(&self) -> usize {
        match self {
            Self::Entry(entry) => html_len_estimate_for_name(entry.name()),
            Self::Toggle(entry) => html_len_estimate_for_name(&entry.when_true)
                .max(html_len_estimate_for_name(&entry.when_false)),
        }
    }

    pub(crate) fn is_reactive(&self) -> bool {
        match self {
            Self::Entry(entry) => entry.when.is_reactive(),
            Self::Toggle(entry) => entry.when.is_reactive(),
        }
    }

    pub(crate) fn has_name(&self, name: &str) -> bool {
        if is_effectively_empty_class_name(name) {
            return false;
        }

        match self {
            Self::Entry(entry) => {
                !is_effectively_empty_class_name(entry.name()) && entry.name() == name
            }
            Self::Toggle(entry) => {
                (!is_effectively_empty_class_name(&entry.when_true) && entry.when_true == name)
                    || (!is_effectively_empty_class_name(&entry.when_false)
                        && entry.when_false == name)
            }
        }
    }

    pub(crate) fn for_each_name(&self, mut f: impl FnMut(&str)) {
        match self {
            Self::Entry(entry) => f(entry.name()),
            Self::Toggle(entry) => {
                f(&entry.when_true);
                f(&entry.when_false);
            }
        }
    }

    pub(crate) fn touch_reactive_dependencies(&self) {
        match self {
            Self::Entry(entry) => entry.when.touch_reactive_dependency(),
            Self::Toggle(entry) => entry.when.touch_reactive_dependency(),
        }
    }
}

impl From<ClassEntry> for ClassItem {
    fn from(value: ClassEntry) -> Self {
        Self::Entry(value)
    }
}

impl From<ToggleEntry> for ClassItem {
    fn from(value: ToggleEntry) -> Self {
        Self::Toggle(value)
    }
}

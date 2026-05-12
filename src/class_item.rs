pub(crate) mod single;
pub(crate) mod toggle;

use crate::class_name::ClassName;

use self::single::Single;
use self::toggle::Toggle;

/// Sum type unifying the single-name [`Single`] and the two-branch [`Toggle`] entry shapes.
///
/// Render-time operations (active-class emission, reactivity check, dependency tracking)
/// dispatch through the enum.
#[derive(Clone, Debug)]
pub(crate) enum ClassItem {
    Single(Single),
    Toggle(Toggle),
}

impl ClassItem {
    pub(crate) fn append_if_active(&self, buf: &mut String, first: &mut bool) {
        /// Appends `name` to `buf` with a leading space, unless this is the first valid name
        /// written (tracked by `first`, which is flipped to `false` on the first append).
        fn append_class_name(buf: &mut String, first: &mut bool, name: &str) {
            if !*first {
                buf.push(' ');
            }
            *first = false;
            buf.push_str(name);
        }

        match self {
            Self::Single(entry) => {
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

    /// Upper bound on the byte length this item contributes to the rendered class string. For a
    /// `Toggle` only one branch can render at a time, so the contribution is
    /// `max(when_true.len(), when_false.len())`, not the sum.
    pub(crate) fn rendered_byte_estimate(&self) -> usize {
        match self {
            Self::Single(entry) => entry.name().len(),
            Self::Toggle(entry) => entry.when_true.len().max(entry.when_false.len()),
        }
    }

    pub(crate) fn is_reactive(&self) -> bool {
        match self {
            Self::Single(entry) => entry.when.is_reactive(),
            Self::Toggle(entry) => entry.when.is_reactive(),
        }
    }

    pub(crate) fn has_name(&self, name: &ClassName) -> bool {
        match self {
            Self::Single(entry) => &entry.name == name,
            Self::Toggle(entry) => &entry.when_true == name || &entry.when_false == name,
        }
    }

    pub(crate) fn touch_reactive_dependencies(&self) {
        match self {
            Self::Single(entry) => entry.when.touch_reactive_dependency(),
            Self::Toggle(entry) => entry.when.touch_reactive_dependency(),
        }
    }
}

impl From<Single> for ClassItem {
    fn from(value: Single) -> Self {
        Self::Single(value)
    }
}

impl From<Toggle> for ClassItem {
    fn from(value: Toggle) -> Self {
        Self::Toggle(value)
    }
}

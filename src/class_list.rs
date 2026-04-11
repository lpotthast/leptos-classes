use smallvec::SmallVec;

use crate::class_entry::{ClassItem, is_effectively_empty_class_name};

/// Internal wrapper around `SmallVec<[ClassItem; 3]>` that provides duplicate detection in debug builds.
///
/// Uses `SmallVec` to avoid heap allocation for the common case of ≤3 class entries.
///
/// Note: The TypedBuilder derive exposes this type in generated builder signatures.
#[doc(hidden)]
#[derive(Clone, Debug, Default)]
pub struct ClassList(SmallVec<[ClassItem; 3]>);

impl ClassList {
    pub(crate) fn push(&mut self, class: impl Into<ClassItem>) {
        let class = class.into();
        class.for_each_name(|name| {
            if is_effectively_empty_class_name(name) {
                tracing::warn!(
                    "Empty or whitespace-only class name added to Classes. This entry will be ignored."
                );
                return;
            }

            #[cfg(debug_assertions)]
            if self.0.iter().any(|it| it.has_name(name)) {
                let backtrace = std::backtrace::Backtrace::force_capture();
                tracing::warn!(
                    "Duplicate class '{}' added to Classes. This may indicate a bug. At: {backtrace}",
                    name
                );
            }
        });
        self.0.push(class);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ClassItem> {
        self.0.iter()
    }

    pub(crate) fn write_active_classes(&self, buf: &mut String) {
        let mut first = buf.is_empty();
        for entry in self.iter() {
            entry.append_active_classes(buf, &mut first);
        }
    }

    pub(crate) fn html_len_estimate(&self) -> usize {
        self.iter().map(ClassItem::html_len_estimate).sum()
    }

    pub(crate) fn is_reactive(&self) -> bool {
        self.iter().any(ClassItem::is_reactive)
    }

    pub(crate) fn touch_reactive_dependencies(&self) {
        for entry in self.iter() {
            entry.touch_reactive_dependencies();
        }
    }
}

use crate::class_name::ClassName;
use crate::condition::ClassCondition;

/// One class name and the condition that controls whether it is rendered.
///
/// Counterpart to [`Toggle`](super::toggle::Toggle): a `Single` stores exactly one class token
/// paired with the condition deciding when it is rendered.
#[derive(Clone, Debug)]
pub(crate) struct Single {
    pub(crate) name: ClassName,
    pub(crate) when: ClassCondition,
}

impl Single {
    /// Returns a reference to the class name.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

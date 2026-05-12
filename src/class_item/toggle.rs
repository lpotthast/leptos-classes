use crate::class_item::single::Single;
use crate::class_name::ClassName;
use crate::condition::ClassCondition;

/// A two-branch class entry that picks `when_true` if its condition is active and `when_false`
/// otherwise.
///
/// Used to model mutually exclusive class pairs (e.g. `"open"` vs `"closed"`) in a single entry,
/// which avoids the extra reactive subscription a pair of opposing [`Single`] values would need.
#[derive(Clone, Debug)]
pub(crate) struct Toggle {
    pub(crate) when: ClassCondition,
    pub(crate) when_true: ClassName,
    pub(crate) when_false: ClassName,
}

impl Toggle {
    /// Decomposes a toggle into two flat [`Single`]s with opposing conditions.
    ///
    /// The `when_true` branch keeps the original condition; the `when_false` branch carries the
    /// negation produced by [`ClassCondition::negate`], so the two resulting `Single`s are
    /// mutually exclusive over the same reactive source as the original toggle.
    pub(crate) fn into_singles(self) -> (Single, Single) {
        let when_false_cond = self.when.clone().negate();
        let when_true = Single {
            name: self.when_true,
            when: self.when,
        };
        let when_false = Single {
            name: self.when_false,
            when: when_false_cond,
        };
        (when_true, when_false)
    }
}

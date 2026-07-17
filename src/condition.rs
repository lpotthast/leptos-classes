use leptos::prelude::{Get, Signal, Track};
#[cfg(not(feature = "nightly"))]
use leptos::prelude::{Memo, RwSignal};
#[cfg(not(feature = "nightly"))]
use leptos::reactive::signal::ReadSignal;

/// A condition that controls whether a class entry is active.
///
/// `ClassCondition` accepts booleans, Leptos boolean signals, common reactive
/// boolean wrappers, and thread-safe reactive closures such as
/// `move || is_active.get()`.
#[derive(Clone, Debug)]
pub struct ClassCondition(ClassConditionKind);

/// Internal representation of a [`ClassCondition`].
///
/// `Always`/`Never` are the static, non-reactive constants produced from a plain `bool`;
/// `When(signal)` is the reactive arm produced from any signal/closure source. `Not(signal)` is
/// the structural negation of a `When(signal)`, produced by `negate` so that inverting a
/// reactive condition does not require allocating a fresh `Signal::derive` node. Splitting these
/// out lets `is_reactive` be a cheap match without touching the signal.
#[derive(Clone, Debug)]
enum ClassConditionKind {
    Always,
    Never,
    When(Signal<bool>),
    Not(Signal<bool>),
}

impl ClassCondition {
    pub(crate) fn always() -> Self {
        Self(ClassConditionKind::Always)
    }

    pub(crate) fn never() -> Self {
        Self(ClassConditionKind::Never)
    }

    pub(crate) fn when_signal(signal: impl Into<Signal<bool>>) -> Self {
        Self(ClassConditionKind::When(signal.into()))
    }

    pub(crate) fn when_predicate(predicate: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        Self::when_signal(Signal::derive(predicate))
    }

    pub(crate) fn is_active(&self) -> bool {
        match &self.0 {
            ClassConditionKind::Always => true,
            ClassConditionKind::Never => false,
            ClassConditionKind::When(when) => when.get(),
            ClassConditionKind::Not(when) => !when.get(),
        }
    }

    pub(crate) fn is_reactive(&self) -> bool {
        matches!(
            &self.0,
            ClassConditionKind::When(_) | ClassConditionKind::Not(_)
        )
    }

    pub(crate) fn touch_reactive_dependency(&self) {
        match &self.0 {
            ClassConditionKind::When(when) | ClassConditionKind::Not(when) => {
                when.track();
            }
            ClassConditionKind::Always | ClassConditionKind::Never => {}
        }
    }

    /// Returns the logical negation of this condition without allocating a new reactive node.
    ///
    /// `Always` <-> `Never` flip statically; `When(s)` <-> `Not(s)` swap variants while reusing
    /// the same underlying signal handle.
    pub(crate) fn negate(self) -> Self {
        Self(match self.0 {
            ClassConditionKind::Always => ClassConditionKind::Never,
            ClassConditionKind::Never => ClassConditionKind::Always,
            ClassConditionKind::When(when) => ClassConditionKind::Not(when),
            ClassConditionKind::Not(when) => ClassConditionKind::When(when),
        })
    }

    /// Returns a condition that is active when `self` or `other` is active.
    ///
    /// Static absorption avoids unnecessary derivations: `Always` absorbs
    /// (`Always || x == Always`) and `Never` is the identity (`Never || x == x`). The
    /// `Never`-identity case may pass an existing `Not(s)` arm through unchanged. All other
    /// reactive combinations allocate a fresh `Signal::derive` and wrap the combined expression
    /// in a `When(...)` arm.
    pub(crate) fn or(self, other: Self) -> Self {
        use ClassConditionKind::{Always, Never, Not, When};
        Self(match (self.0, other.0) {
            (Always, _) | (_, Always) => Always,
            (Never, k) | (k, Never) => k,
            (When(a), When(b)) => When(Signal::derive(move || a.get() || b.get())),
            (When(a), Not(b)) => When(Signal::derive(move || a.get() || !b.get())),
            (Not(a), When(b)) => When(Signal::derive(move || !a.get() || b.get())),
            (Not(a), Not(b)) => When(Signal::derive(move || !a.get() || !b.get())),
        })
    }
}

/// Creates a static (non-reactive) condition from a plain `bool`.
///
/// `true` becomes the always-active condition; `false` the never-active one. Neither
/// variant installs a reactive subscription.
impl From<bool> for ClassCondition {
    fn from(active: bool) -> Self {
        if active {
            Self::always()
        } else {
            Self::never()
        }
    }
}

/// Creates a reactive condition from a Leptos `Signal<bool>`.
///
/// When the signal updates, any owning [`Classes`](crate::Classes) value re-renders.
#[cfg(not(feature = "nightly"))]
impl From<Signal<bool>> for ClassCondition {
    fn from(signal: Signal<bool>) -> Self {
        Self::when_signal(signal)
    }
}

/// Creates a reactive condition from a Leptos `ReadSignal<bool>`, typically the read half
/// of a `signal(...)` pair.
#[cfg(not(feature = "nightly"))]
impl From<ReadSignal<bool>> for ClassCondition {
    fn from(signal: ReadSignal<bool>) -> Self {
        Self::when_signal(signal)
    }
}

/// Creates a reactive condition from a Leptos `RwSignal<bool>`.
#[cfg(not(feature = "nightly"))]
impl From<RwSignal<bool>> for ClassCondition {
    fn from(signal: RwSignal<bool>) -> Self {
        Self::when_signal(signal)
    }
}

/// Creates a reactive condition from a Leptos `Memo<bool>`.
#[cfg(not(feature = "nightly"))]
impl From<Memo<bool>> for ClassCondition {
    fn from(memo: Memo<bool>) -> Self {
        Self::when_signal(memo)
    }
}

/// Creates a reactive condition from a closure.
///
/// Accepts any `Fn() -> bool + Send + Sync + 'static`. The closure is wrapped in
/// `Signal::derive`, so it is re-evaluated whenever its tracked dependencies change -
/// useful for combining several signals, e.g. `move || is_active.get() && !disabled.get()`.
/// With the `nightly` feature, Leptos' reactive wrappers also implement `Fn()` and use
/// this conversion instead of their stable-only explicit conversions above.
impl<F> From<F> for ClassCondition
where
    F: Fn() -> bool + Send + Sync + 'static,
{
    fn from(predicate: F) -> Self {
        Self::when_predicate(predicate)
    }
}

#[cfg(test)]
mod negate {
    use assertr::prelude::*;
    use leptos::prelude::{Set, signal};

    use crate::condition::ClassCondition;

    #[test]
    fn always_negates_to_never() {
        let negated = ClassCondition::always().negate();
        assert_that!(negated.is_active()).is_false();
        assert_that!(negated.is_reactive()).is_false();
    }

    #[test]
    fn never_negates_to_always() {
        let negated = ClassCondition::never().negate();
        assert_that!(negated.is_active()).is_true();
        assert_that!(negated.is_reactive()).is_false();
    }

    #[test]
    fn when_negates_to_inverse_and_stays_reactive() {
        let (s, set_s) = signal(false);
        let negated = ClassCondition::when_signal(s).negate();
        assert_that!(negated.is_reactive()).is_true();
        assert_that!(negated.is_active()).is_true();
        set_s.set(true);
        assert_that!(negated.is_active()).is_false();
    }

    #[test]
    fn double_negation_restores_original_behavior() {
        let (s, set_s) = signal(false);
        let restored = ClassCondition::when_signal(s).negate().negate();
        assert_that!(restored.is_reactive()).is_true();
        assert_that!(restored.is_active()).is_false();
        set_s.set(true);
        assert_that!(restored.is_active()).is_true();
    }
}

#[cfg(test)]
mod or {
    use assertr::prelude::*;
    use leptos::prelude::{Set, signal};

    use crate::condition::ClassCondition;

    mod static_absorption {
        use super::*;

        #[test]
        fn always_or_when_is_always_and_not_reactive() {
            let (s, _) = signal(false);
            let combined = ClassCondition::always().or(ClassCondition::when_signal(s));
            assert_that!(combined.is_active()).is_true();
            assert_that!(combined.is_reactive()).is_false();
        }

        #[test]
        fn when_or_always_is_always() {
            let (s, _) = signal(false);
            let combined = ClassCondition::when_signal(s).or(ClassCondition::always());
            assert_that!(combined.is_active()).is_true();
            assert_that!(combined.is_reactive()).is_false();
        }

        #[test]
        fn never_or_never_is_never() {
            let combined = ClassCondition::never().or(ClassCondition::never());
            assert_that!(combined.is_active()).is_false();
            assert_that!(combined.is_reactive()).is_false();
        }

        #[test]
        fn never_or_when_yields_when() {
            let (s, set_s) = signal(false);
            let combined = ClassCondition::never().or(ClassCondition::when_signal(s));
            assert_that!(combined.is_reactive()).is_true();
            assert_that!(combined.is_active()).is_false();
            set_s.set(true);
            assert_that!(combined.is_active()).is_true();
        }
    }

    mod reactive_combinations {
        use super::*;

        #[test]
        fn when_or_when_tracks_both_signals() {
            let (a, set_a) = signal(false);
            let (b, set_b) = signal(false);
            let combined = ClassCondition::when_signal(a).or(ClassCondition::when_signal(b));
            assert_that!(combined.is_reactive()).is_true();
            assert_that!(combined.is_active()).is_false();
            set_a.set(true);
            assert_that!(combined.is_active()).is_true();
            set_a.set(false);
            set_b.set(true);
            assert_that!(combined.is_active()).is_true();
        }

        #[test]
        fn when_or_not_inverts_the_second_signal() {
            let (a, set_a) = signal(false);
            let (b, set_b) = signal(true);
            // when(a) || not(b) == a || !b.
            let combined =
                ClassCondition::when_signal(a).or(ClassCondition::when_signal(b).negate());
            assert_that!(combined.is_active()).is_false();
            set_b.set(false);
            assert_that!(combined.is_active()).is_true();
            set_b.set(true);
            set_a.set(true);
            assert_that!(combined.is_active()).is_true();
        }

        #[test]
        fn not_or_when_inverts_the_first_signal() {
            let (a, set_a) = signal(true);
            let (b, set_b) = signal(false);
            // not(a) || when(b) == !a || b.
            let combined = ClassCondition::when_signal(a)
                .negate()
                .or(ClassCondition::when_signal(b));
            assert_that!(combined.is_active()).is_false();
            set_a.set(false);
            assert_that!(combined.is_active()).is_true();
            set_a.set(true);
            set_b.set(true);
            assert_that!(combined.is_active()).is_true();
        }

        #[test]
        fn not_or_not_is_or_of_negations() {
            let (a, set_a) = signal(true);
            let (b, set_b) = signal(true);
            // !a || !b. Both true -> false.
            let combined = ClassCondition::when_signal(a)
                .negate()
                .or(ClassCondition::when_signal(b).negate());
            assert_that!(combined.is_active()).is_false();
            set_a.set(false);
            assert_that!(combined.is_active()).is_true();
            set_a.set(true);
            set_b.set(false);
            assert_that!(combined.is_active()).is_true();
        }
    }
}

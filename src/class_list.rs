use smallvec::SmallVec;

use crate::class_item::ClassItem;
use crate::class_item::single::Single;
use crate::class_item::toggle::Toggle;
use crate::class_name::ClassName;
use crate::classes::MergeStrategy;
use crate::condition::ClassCondition;

/// Ordered set of [`ClassItem`]s (either single-name entries or two-branch toggles) keyed by
/// class token.
///
/// Uniqueness is structural: each class name may appear in at most one stored item. Both
/// [`ClassList::add_single`] and [`ClassList::add_toggle`] panic in debug and release if an
/// incoming name collides with one already stored. The variant-aware storage preserves the
/// toggle "concept" through inserts and through non-colliding merges, which keeps the
/// `estimated_class_len` cache tight (`max(when_true.len(), when_false.len())` for a toggle, not
/// the sum) and reflects what the user actually wrote.
///
/// The inline capacity of three keeps small lists heap-free.
#[derive(Clone, Debug)]
pub(crate) struct ClassList {
    items: SmallVec<[ClassItem; 3]>,
    /// Cached so callers can check reactivity without re-scanning items. Updated on each
    /// insertion and after every merge.
    is_reactive: bool,
    /// Running upper bound on the rendered class-string byte length. Maintained incrementally
    /// at insertion. For a Toggle the contribution is `max(t.len(), f.len())` (only one branch
    /// can render at a time), not the sum of both branches.
    estimated_class_len: usize,
}

impl Default for ClassList {
    fn default() -> Self {
        Self::empty()
    }
}

impl ClassList {
    pub(crate) fn empty() -> Self {
        Self {
            items: SmallVec::new(),
            is_reactive: false,
            estimated_class_len: 0,
        }
    }

    /// Inserts a single-name entry. Panics if `name` is already stored on any item.
    pub(crate) fn add_single(&mut self, name: ClassName, when: ClassCondition) {
        self.check_unique(&name);
        self.push_item(ClassItem::Single(Single { name, when }));
    }

    /// Inserts a mutually exclusive two-branch entry. `when_true` renders when `when` is active;
    /// `when_false` renders when it isn't. Panics if `when_true == when_false` (checked first,
    /// so the "branches equal" diagnostic wins over a duplicate-name diagnostic when both
    /// apply) or if either branch collides with a name already stored on this list.
    pub(crate) fn add_toggle(
        &mut self,
        when: ClassCondition,
        when_true: ClassName,
        when_false: ClassName,
    ) {
        if when_true == when_false {
            panic_toggle_branches_equal(&when_false);
        }
        self.check_unique(&when_true);
        self.check_unique(&when_false);
        self.push_item(ClassItem::Toggle(Toggle {
            when,
            when_true,
            when_false,
        }));
    }

    /// Appends every item from `other` into `self`, applying `strategy` on token collisions.
    ///
    /// Fast path: an incoming item whose name(s) do not collide is pushed as-is, preserving
    /// `Toggle` structure across the merge. Slow path: when a collision is detected, the
    /// `Toggle` on the colliding side is dissolved into two flat [`Single`]s with opposing
    /// conditions and each half is routed through the strategy individually - whether the
    /// dissolved toggle lives in `other` (handled by `merge_one`) or in `self` (handled by
    /// `union_at` when an incoming `Single` collides with one of its halves). See
    /// [`MergeStrategy`] for the per-variant semantics.
    pub(crate) fn merge(&mut self, other: ClassList, strategy: MergeStrategy) {
        if other.items.is_empty() {
            return;
        }
        // Approximate upper bound: every incoming `Toggle` lands either structurally (1 slot) or
        // dissolved into two flat `Single`s (2 slots), so `2 * other.len()` covers the
        // incoming-side dissolution. It does not cover the symmetric case where an incoming
        // `Single` collides with one of a self-side `Toggle`'s branches and dissolves that
        // `Toggle` in place (net +1 slot per such collision); those rare cases fall back to
        // `SmallVec`'s normal reallocation. The over-reservation in the common case is the
        // accepted cost of avoiding the more expensive worst case.
        self.items.reserve(other.items.len().saturating_mul(2));
        for incoming in other.items {
            self.merge_one(incoming, strategy);
        }
        self.recompute_caches();
    }

    fn merge_one(&mut self, incoming: ClassItem, strategy: MergeStrategy) {
        match incoming {
            ClassItem::Single(single) => self.merge_incoming_single(single, strategy),
            ClassItem::Toggle(toggle) => {
                let collides = self.items.iter().any(|item| {
                    item.has_name(&toggle.when_true) || item.has_name(&toggle.when_false)
                });
                if collides {
                    let (s_true, s_false) = toggle.into_singles();
                    self.merge_incoming_single(s_true, strategy);
                    self.merge_incoming_single(s_false, strategy);
                } else {
                    self.items.push(ClassItem::Toggle(toggle));
                }
            }
        }
    }

    fn merge_incoming_single(&mut self, incoming: Single, strategy: MergeStrategy) {
        match self.find_item_index(&incoming.name) {
            None => self.items.push(ClassItem::Single(incoming)),
            Some(idx) => match strategy {
                MergeStrategy::PanicOnConflict => panic_duplicate(&incoming.name),
                MergeStrategy::KeepSelf => {}
                MergeStrategy::UnionConditions => self.union_at(idx, incoming),
            },
        }
    }

    /// Updates the entry at `idx` so the condition for `incoming.name` becomes the logical OR of
    /// the existing condition and `incoming.when`. If the entry at `idx` is a `Single`, its
    /// condition is swapped in place; if it is a `Toggle`, the toggle is dissolved into two
    /// `Single`s and only the half matching `incoming.name` is unioned, with the other half
    /// landing as an independent `Single` carrying its negated condition. The two resulting
    /// `Single`s are inserted in `when_true`-then-`when_false` order regardless of which half
    /// matched, so the user's insertion order survives the dissolution.
    fn union_at(&mut self, idx: usize, incoming: Single) {
        match &mut self.items[idx] {
            ClassItem::Single(existing) => {
                let prev = std::mem::replace(&mut existing.when, ClassCondition::never());
                existing.when = prev.or(incoming.when);
            }
            ClassItem::Toggle(_) => {
                let ClassItem::Toggle(toggle) = self.items.remove(idx) else {
                    unreachable!("matched on Toggle immediately above")
                };
                let (s_true, s_false) = toggle.into_singles();
                let (when_true_single, when_false_single) = if s_true.name == incoming.name {
                    let merged = Single {
                        name: s_true.name,
                        when: s_true.when.or(incoming.when),
                    };
                    (merged, s_false)
                } else {
                    let merged = Single {
                        name: s_false.name,
                        when: s_false.when.or(incoming.when),
                    };
                    (s_true, merged)
                };
                self.items.insert(idx, ClassItem::Single(when_true_single));
                self.items
                    .insert(idx + 1, ClassItem::Single(when_false_single));
            }
        }
    }

    fn find_item_index(&self, name: &ClassName) -> Option<usize> {
        self.items.iter().position(|item| item.has_name(name))
    }

    fn check_unique(&self, name: &ClassName) {
        if self.find_item_index(name).is_some() {
            panic_duplicate(name);
        }
    }

    fn push_item(&mut self, item: ClassItem) {
        let separator_cost = usize::from(!self.items.is_empty());
        self.estimated_class_len += item.rendered_byte_estimate() + separator_cost;
        if item.is_reactive() {
            self.is_reactive = true;
        }
        self.items.push(item);
    }

    /// Rebuilds `is_reactive` and `estimated_class_len` from scratch.
    ///
    /// The merge paths (`merge_one`, `merge_incoming_single`, `union_at`) all bypass
    /// [`Self::push_item`]'s incremental cache maintenance and write directly to `self.items`,
    /// so this is called once at the end of [`Self::merge`] to repair both caches. Do not call
    /// `push_item` from inside the merge loop for "consistency" - the cache writes there would
    /// double-count items that later get removed or replaced.
    fn recompute_caches(&mut self) {
        let mut is_reactive = false;
        let mut estimated_class_len = 0;
        for (i, item) in self.items.iter().enumerate() {
            if item.is_reactive() {
                is_reactive = true;
            }
            estimated_class_len += item.rendered_byte_estimate() + usize::from(i > 0);
        }
        self.is_reactive = is_reactive;
        self.estimated_class_len = estimated_class_len;
    }

    pub(crate) fn write_active_classes(&self, buf: &mut String) {
        let mut first = buf.is_empty();
        for item in &self.items {
            item.append_if_active(buf, &mut first);
        }
    }

    pub(crate) fn estimated_class_len(&self) -> usize {
        self.estimated_class_len
    }

    pub(crate) fn is_reactive(&self) -> bool {
        self.is_reactive
    }

    pub(crate) fn touch_reactive_dependencies(&self) {
        for item in &self.items {
            item.touch_reactive_dependencies();
        }
    }
}

fn panic_duplicate(name: &ClassName) -> ! {
    let name = name.as_str();
    panic!(
        "class token `{name}` was registered with Classes more than once. \
         Each class name may appear in at most one entry; \
         combine conditions instead \
         (e.g. add_reactive({name:?}, move || a.get() || b.get()))."
    );
}

fn panic_toggle_branches_equal(name: &ClassName) -> ! {
    let name = name.as_str();
    panic!(
        "add_toggle requires two distinct branch names, but both branches were `{name}`. \
         A toggle whose true/false branches share a name has no effect; \
         use Classes::add (or Classes::add_reactive) instead."
    );
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;
    use leptos::prelude::{Set, signal};

    use crate::class_name::ClassName;
    use crate::condition::ClassCondition;

    use super::ClassList;

    #[test]
    fn add_single_distinct_names_render_in_order() {
        let mut list = ClassList::empty();
        list.add_single(ClassName::from("foo"), ClassCondition::always());
        list.add_single(ClassName::from("bar"), ClassCondition::always());

        let mut buf = String::new();
        list.write_active_classes(&mut buf);
        assert_that!(buf).is_equal_to("foo bar".to_string());
    }

    #[test]
    #[should_panic(expected = "was registered with Classes more than once")]
    fn add_single_duplicate_name_panics() {
        let mut list = ClassList::empty();
        list.add_single(ClassName::from("foo"), ClassCondition::always());
        list.add_single(ClassName::from("foo"), ClassCondition::always());
    }

    #[test]
    #[should_panic(expected = "was registered with Classes more than once")]
    fn add_single_duplicate_across_active_and_inactive_panics() {
        let mut list = ClassList::empty();
        list.add_single(ClassName::from("foo"), ClassCondition::always());
        list.add_single(ClassName::from("foo"), ClassCondition::never());
    }

    #[test]
    #[should_panic(expected = "was registered with Classes more than once")]
    fn add_toggle_collides_with_existing_single_panics() {
        let mut list = ClassList::empty();
        list.add_single(ClassName::from("foo"), ClassCondition::always());
        list.add_toggle(
            ClassCondition::always(),
            ClassName::from("foo"),
            ClassName::from("bar"),
        );
    }

    #[test]
    #[should_panic(expected = "add_toggle requires two distinct branch names")]
    fn add_toggle_with_identical_branches_panics() {
        let mut list = ClassList::empty();
        list.add_toggle(
            ClassCondition::always(),
            ClassName::from("foo"),
            ClassName::from("foo"),
        );
    }

    #[test]
    #[should_panic(expected = "was registered with Classes more than once")]
    fn add_single_collides_with_toggle_false_branch_panics() {
        let mut list = ClassList::empty();
        list.add_toggle(
            ClassCondition::always(),
            ClassName::from("a"),
            ClassName::from("b"),
        );
        list.add_single(ClassName::from("b"), ClassCondition::always());
    }

    #[test]
    fn add_toggle_renders_true_branch_when_active() {
        let mut list = ClassList::empty();
        list.add_toggle(
            ClassCondition::always(),
            ClassName::from("on"),
            ClassName::from("off"),
        );
        let mut buf = String::new();
        list.write_active_classes(&mut buf);
        assert_that!(buf).is_equal_to("on".to_string());
    }

    #[test]
    fn add_toggle_renders_false_branch_when_inactive() {
        let mut list = ClassList::empty();
        list.add_toggle(
            ClassCondition::never(),
            ClassName::from("on"),
            ClassName::from("off"),
        );
        let mut buf = String::new();
        list.write_active_classes(&mut buf);
        assert_that!(buf).is_equal_to("off".to_string());
    }

    #[test]
    fn add_toggle_with_signal_flips_branch_reactively() {
        let (active, set_active) = signal(true);
        let mut list = ClassList::empty();
        list.add_toggle(
            ClassCondition::when_signal(active),
            ClassName::from("on"),
            ClassName::from("off"),
        );
        assert_that!(list.is_reactive()).is_true();

        let mut buf = String::new();
        list.write_active_classes(&mut buf);
        assert_that!(buf).is_equal_to("on".to_string());

        set_active.set(false);
        let mut buf = String::new();
        list.write_active_classes(&mut buf);
        assert_that!(buf).is_equal_to("off".to_string());
    }

    #[test]
    fn estimated_len_for_toggle_uses_longer_branch() {
        // The toggle item contributes max("active-state".len(), "off".len()) = 12 bytes.
        let mut list = ClassList::empty();
        list.add_toggle(
            ClassCondition::always(),
            ClassName::from("active-state"),
            ClassName::from("off"),
        );
        assert_that!(list.estimated_class_len()).is_equal_to(12);
    }

    #[test]
    fn estimated_len_sums_with_separator_after_first_entry() {
        let mut list = ClassList::empty();
        list.add_single(ClassName::from("base"), ClassCondition::always());
        list.add_single(ClassName::from("tail"), ClassCondition::always());
        // "base" (4) + separator (1) + "tail" (4) = 9.
        assert_that!(list.estimated_class_len()).is_equal_to(9);
    }

    mod merge {
        use assertr::prelude::*;
        use leptos::prelude::{Set, signal};

        use crate::class_list::ClassList;
        use crate::class_name::ClassName;
        use crate::classes::MergeStrategy;
        use crate::condition::ClassCondition;

        pub(super) fn list_with(entries: &[(&'static str, ClassCondition)]) -> ClassList {
            let mut list = ClassList::empty();
            for (name, when) in entries {
                list.add_single(ClassName::from(*name), when.clone());
            }
            list
        }

        pub(super) fn rendered(list: &ClassList) -> String {
            let mut buf = String::new();
            list.write_active_classes(&mut buf);
            buf
        }

        mod panic_on_conflict {
            use super::*;

            #[test]
            fn non_overlapping_appends_in_order() {
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                let b = list_with(&[("bar", ClassCondition::always())]);
                a.merge(b, MergeStrategy::PanicOnConflict);
                assert_that!(rendered(&a)).is_equal_to("foo bar".to_string());
            }

            #[test]
            fn empty_other_is_identity() {
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                a.merge(ClassList::empty(), MergeStrategy::PanicOnConflict);
                assert_that!(rendered(&a)).is_equal_to("foo".to_string());
            }

            #[test]
            fn empty_self_yields_other() {
                let mut a = ClassList::empty();
                let b = list_with(&[("foo", ClassCondition::always())]);
                a.merge(b, MergeStrategy::PanicOnConflict);
                assert_that!(rendered(&a)).is_equal_to("foo".to_string());
            }

            #[test]
            #[should_panic(expected = "was registered with Classes more than once")]
            fn panics_on_single_collision() {
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                let b = list_with(&[("foo", ClassCondition::always())]);
                a.merge(b, MergeStrategy::PanicOnConflict);
            }

            #[test]
            #[should_panic(expected = "was registered with Classes more than once")]
            fn panics_on_toggle_half_collision() {
                let mut a = ClassList::empty();
                a.add_toggle(
                    ClassCondition::always(),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                let b = list_with(&[("on", ClassCondition::always())]);
                a.merge(b, MergeStrategy::PanicOnConflict);
            }

            #[test]
            fn recomputed_caches_match_manual_construction() {
                let (s, _) = signal(true);
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                let b = list_with(&[("bar", ClassCondition::when_signal(s))]);
                a.merge(b, MergeStrategy::PanicOnConflict);

                // "foo" (3) + sep (1) + "bar" (3) = 7.
                assert_that!(a.estimated_class_len()).is_equal_to(7);
                assert_that!(a.is_reactive()).is_true();
            }

            #[test]
            fn non_colliding_toggle_from_other_is_preserved_structurally() {
                // The toggle survives as a single item: its contribution to the post-merge
                // estimate is max("active-state".len(), "x".len()) = 12, not the sum.
                // With "foo" (3) + sep (1) + toggle (12) the estimate should be 16.
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                let mut b = ClassList::empty();
                b.add_toggle(
                    ClassCondition::always(),
                    ClassName::from("active-state"),
                    ClassName::from("x"),
                );
                a.merge(b, MergeStrategy::PanicOnConflict);
                assert_that!(a.estimated_class_len()).is_equal_to(16);
                assert_that!(rendered(&a)).is_equal_to("foo active-state".to_string());
            }
        }

        mod keep_self {
            use super::*;

            #[test]
            fn drops_other_entry_on_collision() {
                let (s, _) = signal(false);
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                let b = list_with(&[
                    ("foo", ClassCondition::when_signal(s)),
                    ("bar", ClassCondition::always()),
                ]);
                a.merge(b, MergeStrategy::KeepSelf);
                assert_that!(rendered(&a)).is_equal_to("foo bar".to_string());
                assert_that!(a.is_reactive()).is_false();
            }

            #[test]
            fn preserves_self_toggle_when_other_collides() {
                let (s, set_s) = signal(true);
                let mut a = ClassList::empty();
                a.add_toggle(
                    ClassCondition::when_signal(s),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                let b = list_with(&[("on", ClassCondition::always())]);
                a.merge(b, MergeStrategy::KeepSelf);

                assert_that!(rendered(&a)).is_equal_to("on".to_string());
                set_s.set(false);
                assert_that!(rendered(&a)).is_equal_to("off".to_string());
            }

            #[test]
            fn lets_other_toggle_orphan_survive_as_flat_entry() {
                // a has "on" Always, which collides with b's toggle "on" -> drop b's "on".
                // b's "off" (Not(s)) is non-colliding and lands as a flat entry.
                let (s, set_s) = signal(true);
                let mut a = list_with(&[("on", ClassCondition::always())]);
                let mut b = ClassList::empty();
                b.add_toggle(
                    ClassCondition::when_signal(s),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                a.merge(b, MergeStrategy::KeepSelf);

                assert_that!(rendered(&a)).is_equal_to("on".to_string());
                set_s.set(false);
                assert_that!(rendered(&a)).is_equal_to("on off".to_string());
            }

            #[test]
            fn other_toggle_false_branch_collision_orphans_true_branch() {
                // Symmetric to lets_other_toggle_orphan_survive_as_flat_entry but the collision
                // is on b's `when_false` half. b's "off" is dropped; b's "on" (When(s)) survives
                // as a flat entry behind a's existing "off".
                let (s, set_s) = signal(false);
                let mut a = list_with(&[("off", ClassCondition::always())]);
                let mut b = ClassList::empty();
                b.add_toggle(
                    ClassCondition::when_signal(s),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                a.merge(b, MergeStrategy::KeepSelf);

                assert_that!(rendered(&a)).is_equal_to("off".to_string());
                set_s.set(true);
                assert_that!(rendered(&a)).is_equal_to("off on".to_string());
            }

            #[test]
            fn preserves_self_toggle_when_other_collides_on_false_branch() {
                // Mirror of preserves_self_toggle_when_other_collides but the incoming Single
                // matches the toggle's `when_false`. Self's toggle stays whole (KeepSelf drops
                // the incoming Single without touching anything in self).
                let (s, set_s) = signal(true);
                let mut a = ClassList::empty();
                a.add_toggle(
                    ClassCondition::when_signal(s),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                let b = list_with(&[("off", ClassCondition::always())]);
                a.merge(b, MergeStrategy::KeepSelf);

                assert_that!(rendered(&a)).is_equal_to("on".to_string());
                set_s.set(false);
                assert_that!(rendered(&a)).is_equal_to("off".to_string());
            }
        }

        mod union_conditions {
            use super::*;

            #[test]
            fn always_collapses_to_always() {
                let (s, _) = signal(false);
                let mut a = list_with(&[("foo", ClassCondition::always())]);
                let b = list_with(&[("foo", ClassCondition::when_signal(s))]);
                a.merge(b, MergeStrategy::UnionConditions);
                assert_that!(rendered(&a)).is_equal_to("foo".to_string());
                assert_that!(a.is_reactive()).is_false();
            }

            #[test]
            fn renders_if_either_signal_is_true() {
                let (a_sig, set_a) = signal(false);
                let (b_sig, set_b) = signal(false);
                let mut a = list_with(&[("foo", ClassCondition::when_signal(a_sig))]);
                let b = list_with(&[("foo", ClassCondition::when_signal(b_sig))]);
                a.merge(b, MergeStrategy::UnionConditions);

                assert_that!(a.is_reactive()).is_true();
                assert_that!(rendered(&a)).is_equal_to(String::new());
                set_a.set(true);
                assert_that!(rendered(&a)).is_equal_to("foo".to_string());
                set_a.set(false);
                set_b.set(true);
                assert_that!(rendered(&a)).is_equal_to("foo".to_string());
            }

            #[test]
            fn preserves_entry_order_of_self() {
                let mut a = list_with(&[
                    ("first", ClassCondition::always()),
                    ("second", ClassCondition::always()),
                    ("third", ClassCondition::always()),
                ]);
                let b = list_with(&[("second", ClassCondition::always())]);
                a.merge(b, MergeStrategy::UnionConditions);
                assert_that!(rendered(&a)).is_equal_to("first second third".to_string());
            }

            #[test]
            fn dissolves_self_toggle_when_incoming_single_collides() {
                // a has a Toggle("on"/"off") under signal s. b brings Single("on") under signal o.
                // After merge: "on" should render when s.get() || o.get(); "off" should render
                // when !s.get(). The two can both be active simultaneously (s=false, o=true),
                // which would be impossible for a structural toggle pair.
                let (s, set_s) = signal(false);
                let (o, set_o) = signal(false);
                let mut a = ClassList::empty();
                a.add_toggle(
                    ClassCondition::when_signal(s),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                let b = list_with(&[("on", ClassCondition::when_signal(o))]);
                a.merge(b, MergeStrategy::UnionConditions);

                assert_that!(rendered(&a)).is_equal_to("off".to_string());
                set_o.set(true);
                assert_that!(rendered(&a)).is_equal_to("on off".to_string());
                set_s.set(true);
                assert_that!(rendered(&a)).is_equal_to("on".to_string());
            }

            #[test]
            fn dissolved_self_toggle_keeps_true_before_false_when_false_branch_collides() {
                // Same shape as the previous test, but the incoming Single collides with the
                // toggle's `when_false` half. The dissolution must still lay out the resulting
                // entries in `when_true` then `when_false` order so render order matches what
                // the user wrote (toggling rendered "on" before "off"). With both halves
                // simultaneously active, the output must be "on off", never "off on".
                let (s, set_s) = signal(true);
                let (o, set_o) = signal(false);
                let mut a = ClassList::empty();
                a.add_toggle(
                    ClassCondition::when_signal(s),
                    ClassName::from("on"),
                    ClassName::from("off"),
                );
                let b = list_with(&[("off", ClassCondition::when_signal(o))]);
                a.merge(b, MergeStrategy::UnionConditions);

                assert_that!(rendered(&a)).is_equal_to("on".to_string());
                set_o.set(true);
                // Both "on" (via s) and "off" (via o) are active; order must be true-then-false.
                assert_that!(rendered(&a)).is_equal_to("on off".to_string());
                set_s.set(false);
                assert_that!(rendered(&a)).is_equal_to("off".to_string());
            }
        }
    }
}

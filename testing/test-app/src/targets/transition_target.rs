use leptos::prelude::*;
use leptos_classes::Classes;

/// Renders a target that toggles between a purely static `Classes` value and a reactive one,
/// depending on a signal.
///
/// Verifies that the `IntoClass` state machine correctly transitions between
/// the `Static` and `Reactive` branches: switching directions should keep the
/// rendered `class` attribute in sync without leaking the previous branch's
/// state, and reactive updates after a transition should still propagate.
///
/// Note: the `move || Classes::builder()...build()` closure deliberately rebuilds the entire
/// `Classes` value on every reactive read so the test exercises `IntoClass::rebuild` end-to-end.
/// This is the pessimal pattern for production code; user-facing code should compose `Classes`
/// once and store reactive entries inside it instead.
#[component]
pub fn TransitionTarget() -> impl IntoView {
    let (frozen, set_frozen) = signal(false);
    let (active, set_active) = signal(true);

    let classes = move || {
        if frozen.get() {
            Classes::from(["base", "frozen"])
        } else {
            Classes::builder()
                .with("base")
                .with_toggle(active, "active", "inactive")
                .build()
        }
    };

    view! {
        <section>
            <h2>"transition"</h2>
            <div id="transition-target" class=classes>
                "transition"
            </div>
            <button id="transition-freeze-static" on:click=move |_| set_frozen.set(true)>
                "Freeze static"
            </button>
            <button id="transition-enable-reactive" on:click=move |_| set_frozen.set(false)>
                "Enable reactive"
            </button>
            <button id="transition-set-false" on:click=move |_| set_active.set(false)>
                "Set inactive"
            </button>
            <button id="transition-set-true" on:click=move |_| set_active.set(true)>
                "Set active"
            </button>
        </section>
    }
}

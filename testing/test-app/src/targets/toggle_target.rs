use leptos::prelude::*;
use leptos_classes::Classes;

/// Renders a `with_toggle` pair driven by a boolean signal.
///
/// Verifies that the two branches are mutually exclusive in the rendered `class` attribute:
/// switching the signal swaps `active` and `inactive` without leaving both, and without losing the
/// surrounding static `base`.
#[component]
pub fn ToggleTarget() -> impl IntoView {
    let (active, set_active) = signal(true);

    let classes = Classes::builder()
        .with("base")
        .with_toggle(active, "active", "inactive")
        .build();

    view! {
        <section>
            <h2>"toggle"</h2>
            <div id="toggle-target" class=classes>
                "toggle"
            </div>
            <button id="toggle-set-false" on:click=move |_| set_active.set(false)>
                "Set inactive"
            </button>
            <button id="toggle-set-true" on:click=move |_| set_active.set(true)>
                "Set active"
            </button>
        </section>
    }
}

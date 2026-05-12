use leptos::prelude::*;
use leptos_classes::Classes;

/// Renders a single conditional class entry that the browser test toggles to `false` and back.
///
/// When all entries resolve to `false` the rendered class string is empty, and the `class`
/// attribute should be removed from the element entirely rather than left as `class=""`. Restoring
/// the value should re-create the attribute.
#[component]
pub fn EmptyTarget() -> impl IntoView {
    let (active, set_active) = signal(true);

    let classes = Classes::from(("ephemeral", active));

    view! {
        <section>
            <h2>"empty"</h2>
            <div id="empty-target" class=classes>
                "empty"
            </div>
            <button id="empty-clear" on:click=move |_| set_active.set(false)>
                "Clear class"
            </button>
            <button id="empty-restore" on:click=move |_| set_active.set(true)>
                "Restore class"
            </button>
        </section>
    }
}

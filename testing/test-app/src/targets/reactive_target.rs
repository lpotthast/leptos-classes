use leptos::prelude::*;
use leptos_classes::Classes;

/// Renders a class entry whose presence is driven by a boolean signal.
///
/// Buttons mutate the signal so the browser test can assert that the rendered `class` attribute
/// updates reactively, and that flipping the condition to `false` removes the class entirely
/// instead of leaving stale text behind.
#[component]
pub fn ReactiveTarget() -> impl IntoView {
    let (highlighted, set_highlighted) = signal(true);

    let classes = Classes::builder()
        .with("base")
        .with_reactive("highlighted", highlighted)
        .build();

    view! {
        <section>
            <h2>"reactive"</h2>
            <div id="reactive-target" class=classes>
                "reactive"
            </div>
            <button id="reactive-clear" on:click=move |_| set_highlighted.set(false)>
                "Clear highlighted"
            </button>
            <button id="reactive-restore" on:click=move |_| set_highlighted.set(true)>
                "Restore highlighted"
            </button>
        </section>
    }
}

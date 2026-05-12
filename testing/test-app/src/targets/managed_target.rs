use leptos::{html, prelude::*};
use leptos_classes::Classes;

/// Renders a reactive class-list and exposes a button that overwrites the rendered `class`
/// attribute directly through the DOM, bypassing Leptos.
///
/// `Classes` owns the entire `class` attribute, so a *value-changing* reactive update will write
/// the freshly rendered value to the DOM, overwriting any external mutation that happened in the
/// meantime. Re-setting the signal to the same value it already had renders the same string and
/// is *not* expected to reconcile, since `Classes` treats its in-memory `current` as
/// authoritative and external mutations are out of contract.
#[component]
pub fn ManagedTarget() -> impl IntoView {
    let target_ref = NodeRef::<html::Div>::new();
    let (active, set_active) = signal(true);

    let classes = Classes::builder()
        .with("managed")
        .with_reactive("active", active)
        .build();

    let mutate_target = move |_| {
        if let Some(el) = target_ref.get() {
            el.set_attribute("class", "externally-mutated")
                .expect("managed target class mutation should succeed");
        }
    };

    view! {
        <section>
            <h2>"managed"</h2>
            <div id="managed-target" node_ref=target_ref class=classes>
                "managed"
            </div>
            <button id="managed-mutate" on:click=mutate_target>
                "Mutate managed class"
            </button>
            <button id="managed-set-inactive" on:click=move |_| set_active.set(false)>
                "Set inactive"
            </button>
            <button id="managed-set-active" on:click=move |_| set_active.set(true)>
                "Set active"
            </button>
        </section>
    }
}

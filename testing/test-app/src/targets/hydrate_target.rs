use leptos::prelude::*;
use leptos_classes::Classes;

/// Renders one class during SSR and another during hydration.
///
/// The browser test sees the hydrated `"fresh"` value, proving the `class` attribute is rewritten
/// by the client-side runtime - i.e. that [`leptos_classes::Classes`] participates in hydration
/// rather than being frozen at SSR.
#[component]
pub fn HydrateTarget() -> impl IntoView {
    let class = if cfg!(feature = "ssr") {
        "stale"
    } else {
        "fresh"
    };
    let classes = Classes::builder().with("base").with(class).build();

    view! {
        <section>
            <h2>"hydrate"</h2>
            <div id="hydrate-target" class=classes>
                "hydrate"
            </div>
        </section>
    }
}

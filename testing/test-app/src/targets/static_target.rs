use leptos::prelude::*;
use leptos_classes::Classes;

/// Renders a class-list built statically from the [`Classes::builder`] API.
///
/// Confirms the most basic end-to-end path: a non-reactive `Classes` value flows through Leptos'
/// `IntoClass` rendering and lands as a single space-separated `class` attribute in declaration
/// order.
#[component]
pub fn StaticTarget() -> impl IntoView {
    let classes = Classes::builder()
        .with("foo")
        .with("bar")
        .with("baz")
        .build();
    view! {
        <section>
            <h2>"static"</h2>
            <div id="static-target" class=classes>
                "static"
            </div>
        </section>
    }
}

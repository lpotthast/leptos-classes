use leptos::prelude::*;
use leptos_classes::Classes;

/// Verifies the README's central pitch, passing `Classes` through component layers.
///
/// Each layer extends the inherited container with its own entry. The leaf renders the accumulated
/// `class=classes`. The browser test asserts the final declaration order matches the call order
/// (`layer-root` -> `layer-mid` -> `layer-leaf`).
#[component]
pub fn DrillingTarget() -> impl IntoView {
    view! {
        <section>
            <h2>"drilling"</h2>
            <DrillingMid classes=Classes::new().add("layer-root") />
        </section>
    }
}

#[component]
fn DrillingMid(#[prop(into, optional)] classes: Classes) -> impl IntoView {
    view! { <DrillingLeaf classes=classes.add("layer-mid") /> }
}

#[component]
fn DrillingLeaf(#[prop(into, optional)] classes: Classes) -> impl IntoView {
    view! {
        <div id="drilling-target" class=classes.add("layer-leaf")>
            "drilling"
        </div>
    }
}

use leptos::{html, prelude::*};
use leptos_classes::Classes;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <PageClasses />
    }
}

#[component]
fn PageClasses() -> impl IntoView {
    let managed_target_ref = NodeRef::<html::Div>::new();
    let (managed_active, set_managed_active) = signal(true);
    let (transition_active, set_transition_active) = signal(true);
    let (transition_static, set_transition_static) = signal(false);
    let (empty_active, set_empty_active) = signal(true);

    let mutate_managed_target = {
        let managed_target_ref = managed_target_ref.clone();
        move |_| {
            if let Some(el) = managed_target_ref.get() {
                el.set_attribute("class", "managed active external")
                    .expect("managed target class mutation should succeed");
            }
        }
    };

    view! {
        <main id="test-page-classes">
            <h1>"leptos-classes browser test"</h1>

            <section id="managed-section">
                <h2>"Managed class ownership"</h2>
                <div
                    id="managed-target"
                    node_ref=managed_target_ref
                    class=Classes::builder()
                        .with("managed")
                        .with(("active", managed_active))
                        .build()
                >
                    "Managed target"
                </div>
                <div>
                    <button id="managed-mutate" on:click=mutate_managed_target>
                        "Mutate managed class"
                    </button>
                    <button id="managed-set-same-active" on:click=move |_| set_managed_active.set(true)>
                        "Set active again"
                    </button>
                    <button id="managed-set-inactive" on:click=move |_| set_managed_active.set(false)>
                        "Set inactive"
                    </button>
                    <button id="managed-set-active" on:click=move |_| set_managed_active.set(true)>
                        "Set active"
                    </button>
                </div>
            </section>

            <section id="transition-section">
                <h2>"Static and reactive transitions"</h2>
                <div
                    id="transition-target"
                    class=move || {
                        if transition_static.get() {
                            Classes::from("base frozen")
                        } else {
                            Classes::builder()
                                .with("base")
                                .with_toggle(transition_active, "active", "inactive")
                                .build()
                        }
                    }
                >
                    "Transition target"
                </div>
                <div>
                    <button id="transition-freeze-static" on:click=move |_| set_transition_static.set(true)>
                        "Freeze static"
                    </button>
                    <button id="transition-enable-reactive" on:click=move |_| set_transition_static.set(false)>
                        "Enable reactive"
                    </button>
                    <button id="transition-set-false" on:click=move |_| set_transition_active.set(false)>
                        "Set false"
                    </button>
                    <button id="transition-set-true" on:click=move |_| set_transition_active.set(true)>
                        "Set true"
                    </button>
                </div>
            </section>

            <section id="empty-section">
                <h2>"Empty class removal"</h2>
                <div
                    id="empty-target"
                    class=Classes::from(("ephemeral", empty_active))
                >
                    "Empty target"
                </div>
                <div>
                    <button id="empty-set-false" on:click=move |_| set_empty_active.set(false)>
                        "Clear class"
                    </button>
                    <button id="empty-set-true" on:click=move |_| set_empty_active.set(true)>
                        "Restore class"
                    </button>
                </div>
            </section>
        </main>
    }
}

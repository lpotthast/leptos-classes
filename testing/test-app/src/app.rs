use leptos::prelude::*;

use crate::targets::{
    DrillingTarget, EmptyTarget, HydrateTarget, ManagedTarget, ReactiveTarget, StaticTarget,
    ToggleTarget, TransitionTarget,
};

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
        <main id="app">
            <h1>"leptos-classes test app"</h1>
            <StaticTarget />
            <HydrateTarget />
            <ReactiveTarget />
            <ToggleTarget />
            <DrillingTarget />
            <TransitionTarget />
            <ManagedTarget />
            <EmptyTarget />
        </main>
    }
}

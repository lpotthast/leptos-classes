//! One [`leptos::component`] per browser-test scenario.
//!
//! Each module exports a single self-contained component that owns its signals and renders its
//! assertion target plus any control buttons. To add a new browser-tested behavior, add a module
//! here, expose its component, and include it in `App` in `crate::app`.

mod drilling_target;
mod empty_target;
mod hydrate_target;
mod managed_target;
mod reactive_target;
mod static_target;
mod toggle_target;
mod transition_target;

pub use drilling_target::DrillingTarget;
pub use empty_target::EmptyTarget;
pub use hydrate_target::HydrateTarget;
pub use managed_target::ManagedTarget;
pub use reactive_target::ReactiveTarget;
pub use static_target::StaticTarget;
pub use toggle_target::ToggleTarget;
pub use transition_target::TransitionTarget;

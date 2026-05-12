#![doc = include_str!("../README.md")]

mod class_item;
mod class_list;
mod class_name;
mod classes;
mod condition;
mod convert;
mod into_class;

pub use class_name::{ClassName, InvalidClassName};
pub use classes::{Classes, ClassesBuilder, MergeStrategy};

/// Compile-time assertions that the public types stay `Send + Sync`. `Classes` is intended to
/// flow through Leptos component props, which require both bounds. If a future change to any
/// internal field weakens that, this `const _` will fail to compile and surface the regression at
/// the crate boundary instead of at a downstream consumer.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Classes>();
    assert_send_sync::<ClassesBuilder>();
    assert_send_sync::<ClassName>();
    assert_send_sync::<InvalidClassName>();
    assert_send_sync::<MergeStrategy>();
};

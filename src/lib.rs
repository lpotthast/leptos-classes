#![doc = include_str!("../README.md")]

mod class_entry;
mod class_list;
mod classes;
mod convert;
mod into_class;

pub use class_entry::{ClassEntry, ClassName};
/// Internal compatibility re-export for `TypedBuilder`-generated builder signatures.
///
/// `ClassList` remains an implementation detail with no public constructors. Changes to this
/// hidden re-export and the internal type it exposes are not covered by the crate's usual semver
/// expectations.
#[doc(hidden)]
pub use class_list::ClassList;
pub use classes::Classes;
pub use into_class::ClassesState;

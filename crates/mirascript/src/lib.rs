#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate self as mirascript;

/// The complete MiraScript virtual-machine API.
pub use mirascript_vm as vm;
pub use mirascript_vm::*;

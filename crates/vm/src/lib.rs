#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate self as mirascript_vm;

mod bytecode;
mod compile;
mod error;
mod interpreter;
mod operations;
mod runtime_providers;
mod standard_library;
mod value;

use std::{rc::Rc, time::Duration};

pub use core::{CompileConfig, DiagnosticPositionEncoding, InputMode};
pub use mirascript_core as core;
pub use mirascript_vm_derive::{MiraArray, MiraRecord, mira};

use bytecode::Program;
pub use compile::{MiraScript, ScriptId};
pub use error::*;
pub use interpreter::Runtime;
pub use runtime_providers::{RuntimeProviders, default_runtime_providers};
pub(crate) use value::MiraValueKind;
pub use value::{
    MiraArray, MiraExtern, MiraFunction, MiraHandle, MiraManageable, MiraModule, MiraNativeFn,
    MiraRecord, MiraShapedArray, MiraShapedRecord, MiraType, MiraValue, TryFromMira,
};

/// Limits and injectable providers used for each Runtime execution.
#[derive(Clone)]
pub struct RunOptions {
    /// Maximum wall-clock time allowed for one execution.
    pub timeout: Duration,
    /// Number of interpreter checkpoints between timeout-provider checks.
    pub checkpoint_interval: u32,
    /// Maximum nested script and native call depth.
    pub max_call_depth: u32,
    /// Maximum number of elements created by bounded array operations.
    pub max_array_len: usize,
    /// Host implementation for random numbers, time, and debug output.
    pub providers: Rc<dyn RuntimeProviders>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(100),
            checkpoint_interval: 100,
            max_call_depth: 128,
            max_array_len: 0x100_0000,
            providers: default_runtime_providers(),
        }
    }
}

/// Compile source with the default compiler configuration.
///
/// ```
/// use mirascript_vm::{MiraValue, Runtime, compile};
///
/// let script = compile("6 * 7")?;
/// let mut runtime = Runtime::new();
/// assert_eq!(runtime.run(&script)?, MiraValue::number(42.0));
/// # Ok::<(), Box<mirascript_vm::MiraError>>(())
/// ```
pub fn compile(source: &str) -> Result<MiraScript> {
    compile_with(source, &CompileConfig::default())
}

/// Compile source with an explicit [`CompileConfig`].
///
/// ```
/// use mirascript_vm::{MiraValue, Runtime, CompileConfig, InputMode, compile_with};
///
/// let mut config = CompileConfig::new();
/// config.input_mode = InputMode::Template;
/// let script = compile_with("Hello, $name!", &config)?;
/// let mut runtime = Runtime::new();
/// runtime.insert_global("name", &"Alice")?;
/// assert_eq!(runtime.run(&script)?.as_str(&runtime)?.unwrap(), "Hello, Alice!");
/// # Ok::<(), Box<mirascript_vm::MiraError>>(())
/// ```
pub fn compile_with(source: &str, config: &CompileConfig) -> Result<MiraScript> {
    let (chunk, diagnostics) = core::Compiler::compile(source, config);
    let chunk = chunk.ok_or_else(|| MiraError::compile(&diagnostics))?;
    Ok(MiraScript::new(Program::decode(&chunk)?))
}

/// Items used by the derive macros. They are not a stable user-facing API.
#[doc(hidden)]
pub mod __private {
    pub use crate::{MiraError, MiraManageable, MiraValue, Result};

    pub use crate::value::types::field::{
        MiraField, MiraFieldGetter, array_from_array, array_from_record, shaped_array_from_array,
        shaped_array_from_record, shaped_record_from_array, shaped_record_from_record,
    };

    pub use crate::value::types::function::{
        MiraFunction, MiraNativeFn,
        helper::{native_argument, native_result},
    };
}

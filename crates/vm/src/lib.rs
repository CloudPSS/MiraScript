#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate self as mirascript_vm;

mod bytecode;
mod error;
mod interpreter;
mod operations;
mod standard_library;
mod value;

use std::{
    num::NonZeroU64,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub use core::{CompileConfig, DiagnosticPositionEncoding, InputMode};
pub use mirascript_core as core;
pub use mirascript_vm_derive::{MiraArray, MiraRecord};

pub use error::*;
pub use interpreter::Runtime;
pub(crate) use value::MiraValueKind;
pub use value::{
    MiraArray, MiraExtern, MiraFunction, MiraHandle, MiraManageable, MiraModule, MiraNativeFn,
    MiraRecord, MiraShapedArray, MiraShapedRecord, MiraType, MiraValue,
};

use bytecode::Program;

static NEXT_SCRIPT_ID: AtomicU64 = AtomicU64::new(1);

/// Stable identity shared by clones of one compiled script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptId(NonZeroU64);

impl ScriptId {
    fn new() -> Self {
        let id = NEXT_SCRIPT_ID.fetch_add(1, Ordering::Relaxed);
        Self(NonZeroU64::new(id).expect("MiraScript identifier space exhausted"))
    }
}

/// A validated, reusable MiraScript program.
#[derive(Clone, Debug)]
pub struct MiraScript {
    id: ScriptId,
    program: Rc<Program>,
}

impl MiraScript {
    /// Return the stable identity shared by clones of this script.
    pub fn id(&self) -> ScriptId {
        self.id
    }
}

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

/// Host capabilities used by non-deterministic standard-library functions.
pub trait RuntimeProviders {
    /// Return a uniformly distributed random number in `[0, 1)`.
    fn random(&self) -> f64 {
        rand::random()
    }

    /// Return the current Unix timestamp in milliseconds.
    fn now_millis(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    /// Emit one debug message from a script.
    fn debug(&self, message: &str) {
        eprintln!("{message}");
    }
}

struct SystemRuntimeProviders;
impl RuntimeProviders for SystemRuntimeProviders {}

std::thread_local! {
    static DEFAULT_RUNTIME_PROVIDERS: Rc<dyn RuntimeProviders> = Rc::new(SystemRuntimeProviders);
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(100),
            checkpoint_interval: 100,
            max_call_depth: 128,
            max_array_len: 0x100_0000,
            providers: DEFAULT_RUNTIME_PROVIDERS.with(Clone::clone),
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
    Ok(MiraScript {
        id: ScriptId::new(),
        program: Rc::new(Program::decode(&chunk)?),
    })
}

/// Items used by the derive macros. They are not a stable user-facing API.
#[doc(hidden)]
pub mod __private {
    pub use crate::value::types::field::{
        MiraField, MiraFieldGetter, array_from_array, array_from_record, shaped_array_from_array,
        shaped_array_from_record, shaped_record_from_array, shaped_record_from_record,
    };
}

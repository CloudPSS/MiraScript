#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod bytecode;
mod context;
mod error;
mod interpreter;
mod operations;
mod standard_library;
mod value;

use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use context::MiraContext;
pub use error::{MiraError, Result};
pub use mira_vm_derive::{MiraArray, MiraExtern, MiraRecord};
pub use value::{
    MiraAny, MiraArray, MiraCallContext, MiraExtern, MiraFunction, MiraModule, MiraNativeFn,
    MiraRecord, MiraShared,
};

#[doc(hidden)]
pub mod __private {
    pub use crate::value::MiraBridge;
}

use bytecode::Program;

/// A validated, reusable MiraScript program.
#[derive(Clone)]
pub struct MiraScript {
    program: Program,
}

/// Limits and injectable providers for one execution.
#[derive(Clone)]
pub struct RunOptions {
    /// Maximum wall-clock time allowed for one execution.
    pub timeout: Duration,
    /// Number of interpreter checkpoints between timeout-provider checks.
    ///
    /// Values below one are treated as one.
    pub checkpoint_interval: u32,
    /// Maximum nested script, native, and extern call depth.
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
/// The returned program is validated once and can be reused with multiple
/// [`MiraContext`] values.
///
/// # Examples
///
/// ```
/// use mira_vm::{MiraAny, MiraContext, compile};
///
/// let script = compile("answer + 1")?;
/// let mut context = MiraContext::new();
/// context.insert("answer", 41);
/// assert_eq!(script.run(&context)?, MiraAny::Number(42.0));
/// # Ok::<(), mira_vm::MiraError>(())
/// ```
pub fn compile(source: &str) -> Result<MiraScript> {
    compile_with(source, &mira_core::Config::new())
}

/// Compile source with an explicit [`mira_core::Config`].
pub fn compile_with(source: &str, config: &mira_core::Config) -> Result<MiraScript> {
    let (chunk, diagnostics) = mira_core::Compiler::compile(source, config);
    let chunk = chunk.ok_or(MiraError::Compile { diagnostics })?;
    Ok(MiraScript {
        program: Program::decode(&chunk)?,
    })
}

/// Compile and execute source once with default [`RunOptions`].
///
/// # Examples
///
/// ```
/// use mira_vm::{MiraAny, MiraContext, eval};
///
/// assert_eq!(eval("6 * 7", &MiraContext::new())?, MiraAny::Number(42.0));
/// # Ok::<(), mira_vm::MiraError>(())
/// ```
pub fn eval(source: &str, context: &MiraContext) -> Result<MiraAny> {
    compile(source)?.run(context)
}

impl MiraScript {
    /// Execute this program with default [`RunOptions`].
    pub fn run(&self, context: &MiraContext) -> Result<MiraAny> {
        self.run_with(context, &RunOptions::default())
    }

    /// Execute this program with explicit limits and runtime providers.
    pub fn run_with(&self, context: &MiraContext, options: &RunOptions) -> Result<MiraAny> {
        interpreter::run(&self.program, context, options)
    }
}

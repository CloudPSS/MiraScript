//! A native, single-threaded MiraScript virtual machine.

mod bytecode;
mod context;
mod error;
mod interpreter;
mod operations;
mod standard_library;
mod value;

use std::time::Duration;

pub use context::MiraContext;
pub use error::{MiraError, Result};
pub use mira_vm_derive::{MiraArray, MiraExtern, MiraRecord};
pub use value::{
    MiraAny, MiraArray, MiraCallContext, MiraExtern, MiraExternValue, MiraFunction, MiraModule,
    MiraNativeFn, MiraRecord, MiraShared,
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
    pub timeout: Duration,
    pub checkpoint_interval: u32,
    pub max_call_depth: u32,
    pub max_array_len: usize,
    pub providers: RuntimeProviders,
}

/// Host capabilities used by non-deterministic standard-library functions.
#[derive(Clone)]
pub struct RuntimeProviders {
    pub random: std::rc::Rc<dyn Fn() -> f64>,
    pub now_millis: std::rc::Rc<dyn Fn() -> i64>,
    pub debug: std::rc::Rc<dyn Fn(&str)>,
}

std::thread_local! {
    static DEFAULT_RUNTIME_PROVIDERS: RuntimeProviders = RuntimeProviders::system();
}

impl RuntimeProviders {
    fn system() -> Self {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::{SystemTime, UNIX_EPOCH};

        let seed = Rc::new(Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        ));
        let random_seed = Rc::clone(&seed);
        Self {
            random: Rc::new(move || {
                // A small deterministic generator keeps the runtime dependency-free. Tests can
                // replace it through `RunOptions::providers`.
                let mut x = random_seed.get();
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                random_seed.set(x);
                (x >> 11) as f64 / ((1u64 << 53) as f64)
            }),
            now_millis: Rc::new(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64
            }),
            debug: Rc::new(|message| eprintln!("{message}")),
        }
    }
}

impl Default for RuntimeProviders {
    fn default() -> Self {
        DEFAULT_RUNTIME_PROVIDERS.with(Clone::clone)
    }
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(100),
            checkpoint_interval: 100,
            max_call_depth: 128,
            max_array_len: 0x100_0000,
            providers: RuntimeProviders::default(),
        }
    }
}

/// Compile source with the default compiler configuration.
pub fn compile(source: &str) -> Result<MiraScript> {
    compile_with(source, &mira_core::Config::new())
}

/// Compile source with an explicit compiler configuration.
pub fn compile_with(source: &str, config: &mira_core::Config) -> Result<MiraScript> {
    let (chunk, diagnostics) = mira_core::Compiler::compile(source, config);
    let chunk = chunk.ok_or(MiraError::Compile { diagnostics })?;
    Ok(MiraScript {
        program: Program::decode(&chunk)?,
    })
}

/// Compile and execute source once.
pub fn eval(source: &str, context: &MiraContext) -> Result<MiraAny> {
    compile(source)?.run(context)
}

impl MiraScript {
    pub fn run(&self, context: &MiraContext) -> Result<MiraAny> {
        self.run_with(context, &RunOptions::default())
    }

    pub fn run_with(&self, context: &MiraContext, options: &RunOptions) -> Result<MiraAny> {
        interpreter::run(&self.program, context, options)
    }
}

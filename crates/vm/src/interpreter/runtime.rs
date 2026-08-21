use std::{
    collections::HashMap,
    num::NonZeroU64,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use indexmap::IndexMap;

use crate::{
    MiraError, MiraManageable, MiraNativeFn, MiraScript, MiraValue, Result, RunOptions,
    RuntimeErrorKind, ScriptId,
    bytecode::{Constant, Program},
    compile,
    value::MiraArena,
};

use super::{CallStack, Flow, FrameArena, ROOT_FRAME_ID};

static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);

fn next_execution_id() -> ExecutionId {
    let id = NEXT_EXECUTION_ID.fetch_add(1, Ordering::Relaxed);
    ExecutionId(NonZeroU64::new(id).expect("MiraScript execution identifier space exhausted"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ExecutionId(NonZeroU64);

/// Persistent state used to execute compiled MiraScript programs.
///
/// A Runtime owns all dynamically allocated values. Values containing handles
/// therefore remain usable until this Runtime is dropped.
pub struct Runtime {
    pub(crate) execution: ExecutionId,
    pub(crate) options: RunOptions,
    pub(crate) globals: IndexMap<String, MiraValue>,
    pub(crate) arena: MiraArena,
    constant_cache: HashMap<(ScriptId, usize), MiraValue>,
    running: bool,
    active_script: Option<ScriptId>,
    pub(crate) program: Option<Rc<Program>>,
    pub(crate) started: Instant,
    pub(crate) checkpoint_remaining: u32,
    pub(crate) call_depth: u32,
    pub(super) frames: FrameArena,
    pub(super) call_stack: CallStack,
}

impl Runtime {
    /// Create a Runtime using default limits and install the standard library.
    pub fn new() -> Self {
        Self::with_options(RunOptions::default())
    }

    /// Create a Runtime using explicit execution limits and providers.
    pub fn with_options(options: RunOptions) -> Self {
        let checkpoint_remaining = options.checkpoint_interval.max(1);
        let mut runtime = Self {
            execution: next_execution_id(),
            options,
            globals: IndexMap::new(),
            arena: MiraArena::new(),
            constant_cache: HashMap::new(),
            running: false,
            active_script: None,
            program: None,
            started: Instant::now(),
            checkpoint_remaining,
            call_depth: 0,
            frames: FrameArena::new(0),
            call_stack: CallStack::new(),
        };
        crate::standard_library::install(&mut runtime);
        runtime
    }

    /// Compile and execute a MiraScript program in this Runtime.
    pub fn eval(&mut self, script: &str) -> Result<MiraValue> {
        let script = compile(script)?;
        self.run(&script)
    }

    /// Execute a compiled script in this Runtime.
    pub fn run(&mut self, script: &MiraScript) -> Result<MiraValue> {
        if self.running {
            return Err(MiraError::runtime(RuntimeErrorKind::ReentrantRun));
        }

        self.running = true;
        self.execution = next_execution_id();
        self.active_script = Some(script.id);
        self.program = Some(Rc::clone(&script.program));
        self.started = Instant::now();
        self.checkpoint_remaining = self.options.checkpoint_interval.max(1);
        self.call_depth = 0;
        self.frames = FrameArena::new(script.program.root.register_count);
        self.call_stack = CallStack::new();

        let result = (|| {
            let body = &script.program.root.body;
            let value = match self.execute_block(body, ROOT_FRAME_ID)? {
                Flow::Return(value) => value,
                Flow::Continue => MiraValue::Nil,
                Flow::Break | Flow::LoopContinue => {
                    return Err(MiraError::runtime(RuntimeErrorKind::InvalidControlFlow {
                        context: "root",
                    }));
                }
            };
            if self.contains_script_reference(value)? {
                return Err(MiraError::runtime(RuntimeErrorKind::EscapingClosure));
            }
            Ok(value)
        })();

        self.running = false;
        self.active_script = None;
        self.program = None;
        self.call_depth = 0;
        self.call_stack = CallStack::new();
        result
    }

    /// Return the Runtime's execution configuration.
    pub fn options(&self) -> &RunOptions {
        &self.options
    }

    /// Insert or replace a global after converting it into a Runtime value.
    pub fn insert_global(
        &mut self,
        name: impl Into<String>,
        value: impl Into<MiraManageable>,
    ) -> Result<Option<MiraValue>> {
        let value = self.insert(value)?;
        Ok(self.globals.insert(name.into(), value))
    }

    /// Insert a named native function into the global namespace.
    pub fn insert_fn(&mut self, name: impl Into<String>, function: impl Into<MiraNativeFn>) {
        let name = name.into();
        let function = function.into().with_name(name.clone());
        let handle = self
            .insert_function(function)
            .expect("a fresh native function arena slot must be available");
        self.globals
            .insert(name, MiraValue::Function(handle.erase_function()));
    }

    /// Clone a global value by name.
    pub fn get_global(&self, name: &str) -> Option<MiraValue> {
        self.globals.get(name).copied()
    }

    /// Return whether a global name is defined.
    pub fn contains_global(&self, name: &str) -> bool {
        self.globals.contains_key(name)
    }

    /// Iterate over global names in insertion order.
    pub fn global_names(&self) -> impl Iterator<Item = &str> {
        self.globals.keys().map(String::as_str)
    }

    /// Compare values using MiraScript's structural equality semantics.
    pub fn values_equal(&mut self, left: MiraValue, right: MiraValue) -> Result<bool> {
        crate::operations::host_equal(self, left, right)
    }

    pub(crate) fn active_program(&self) -> &Rc<Program> {
        self.program
            .as_ref()
            .expect("interpreter operations require an active program")
    }

    pub(crate) fn materialize_constant(&mut self, index: usize) -> Result<MiraValue> {
        let constant = &self.active_program().constants[index];
        match constant {
            Constant::Nil => Ok(MiraValue::Nil),
            Constant::True => Ok(MiraValue::Boolean(true)),
            Constant::False => Ok(MiraValue::Boolean(false)),
            Constant::Int(value) => Ok(MiraValue::Number(f64::from(*value))),
            Constant::Float(value) => Ok(MiraValue::Number(*value)),
            Constant::String(value) => {
                let script = self
                    .active_script
                    .expect("constant materialization requires an active script");
                if let Some(value) = self.constant_cache.get(&(script, index)) {
                    return Ok(*value);
                }
                let value = self.insert(value.clone().into_string())?;
                self.constant_cache.insert((script, index), value);
                Ok(value)
            }
        }
    }

    pub(crate) fn constant_string(&self, index: usize) -> Result<Option<&str>> {
        match &self.active_program().constants[index] {
            Constant::Nil => Ok(None),
            Constant::String(value) => Ok(Some(value)),
            _ => Err(MiraError::runtime(RuntimeErrorKind::InvalidConstantKind {
                index,
                expected: "string",
            })),
        }
    }

    #[cfg(test)]
    pub(crate) fn materialized_constant_count(&self) -> usize {
        self.constant_cache.len()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{MiraValue, Runtime, compile};

    #[test]
    fn string_constants_are_materialized_lazily_and_cached_per_script() {
        let skipped = compile("if false { 'unreachable' } else { 1 }").unwrap();
        let repeated = compile("'cached'").unwrap();
        let repeated_clone = repeated.clone();
        let distinct = compile("'cached'").unwrap();

        let mut first = Runtime::new();
        first.run(&skipped).unwrap();
        assert_eq!(first.materialized_constant_count(), 0);

        let first_value = first.run(&repeated).unwrap();
        assert_eq!(first_value.as_str(&first).unwrap(), Some("cached"));
        assert_eq!(first.materialized_constant_count(), 1);
        first.run(&repeated_clone).unwrap();
        assert_eq!(first.materialized_constant_count(), 1);
        first.run(&distinct).unwrap();
        assert_eq!(first.materialized_constant_count(), 2);

        let mut second = Runtime::new();
        let second_value = second.run(&repeated).unwrap();
        assert_eq!(second.materialized_constant_count(), 1);
        assert!(matches!(first_value, MiraValue::String(_)));
        assert!(matches!(second_value, MiraValue::String(_)));
        assert!(second.insert(first_value).is_err());
    }
}

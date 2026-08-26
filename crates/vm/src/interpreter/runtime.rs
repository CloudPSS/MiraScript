use std::{collections::HashMap, num::NonZeroU64, rc::Rc, time::Instant};

use crate::{
    MiraError, MiraScript, MiraValue, Result, RunOptions, RuntimeErrorKind, ScriptId,
    bytecode::{Constant, Program},
    compile,
    value::MiraArena,
};

use super::{CallStack, Flow, FrameArena, FrameId, Globals};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ExecutionId(NonZeroU64);

impl ExecutionId {
    const INITIAL: Self = Self(NonZeroU64::MIN);

    // Script references are resolved through this Runtime's arena before this
    // generation is compared, so foreign Runtime values remain guarded by the
    // arena identifier and the generation only needs to be Runtime-local.
    fn next(self) -> Self {
        let id = self
            .0
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .expect("MiraScript execution identifier space exhausted");
        Self(id)
    }
}

/// Persistent state used to execute compiled MiraScript programs.
///
/// A Runtime owns all dynamically allocated values. Values containing handles
/// therefore remain usable until this Runtime is dropped.
pub struct Runtime {
    pub(crate) execution: ExecutionId,
    pub(crate) options: RunOptions,
    pub(crate) globals: Globals,
    pub(crate) arena: MiraArena,
    constant_cache: HashMap<(ScriptId, usize), MiraValue>,
    running: bool,
    active_script: Option<ScriptId>,
    pub(crate) program: Option<Rc<Program>>,
    pub(crate) started: Instant,
    pub(crate) checkpoint_remaining: u32,
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
            execution: ExecutionId::INITIAL,
            options,
            globals: Globals::new(),
            arena: MiraArena::new(),
            constant_cache: HashMap::new(),
            running: false,
            active_script: None,
            program: None,
            started: Instant::now(),
            checkpoint_remaining,
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

    /// Compile and execute a MiraScript program in this Runtime, returning the result or panicking on error.
    pub fn eval_unchecked(&mut self, script: &str) -> MiraValue {
        self.eval(script).expect("evaluation failed")
    }

    /// Execute a compiled script in this Runtime.
    pub fn run(&mut self, script: &MiraScript) -> Result<MiraValue> {
        if self.running {
            return Err(MiraError::runtime(RuntimeErrorKind::ReentrantRun));
        }

        self.running = true;
        self.execution = self.execution.next();
        self.active_script = Some(script.id());
        self.program = Some(script.program());
        self.started = Instant::now();
        self.checkpoint_remaining = self.options.checkpoint_interval.max(1);
        self.frames
            .begin_run(script.program_ref().root.register_count);
        debug_assert_eq!(self.call_stack.depth(), 0);

        let result = (|| {
            let body = &script.program_ref().root.body;
            let value = match self.execute_block(body, FrameId::ROOT)? {
                Flow::Return(value) => value,
                Flow::Continue => MiraValue::NIL,
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
        debug_assert_eq!(self.call_stack.depth(), 0);
        result
    }

    /// Return the Runtime's execution configuration.
    pub fn options(&self) -> &RunOptions {
        &self.options
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
            Constant::Nil => Ok(MiraValue::NIL),
            Constant::True => Ok(MiraValue::boolean(true)),
            Constant::False => Ok(MiraValue::boolean(false)),
            Constant::Int(value) => Ok(MiraValue::number(f64::from(*value))),
            Constant::Float(value) => Ok(MiraValue::number(*value)),
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
    use crate::{Runtime, compile};

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
        assert!(first_value.is_string());
        assert!(second_value.is_string());
        assert!(second.insert(first_value).is_err());
    }
}

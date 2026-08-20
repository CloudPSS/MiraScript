use std::any::Any;

use super::*;
use crate::{MiraManageable, bytecode::Program};

pub(crate) struct ScriptFunction {
    pub(crate) execution: ExecutionId,
    pub(crate) program: Rc<Program>,
    pub(crate) function: usize,
    pub(crate) frame: FrameId,
    pub(crate) name: Option<Rc<str>>,
}

impl MiraFunction for ScriptFunction {
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable> {
        if self.execution != runtime.execution {
            return Err(MiraError::runtime(RuntimeErrorKind::ExecutionEnded));
        }
        let definition = &self.program.functions[self.function];
        runtime
            .call_script(definition, self.frame, args)
            .map(Into::into)
    }

    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("<anonymous>")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Runtime {
    pub(super) fn call_registers(
        &mut self,
        target: MiraValue,
        registers: &[usize],
        spreads: &[usize],
        frame: FrameId,
    ) -> Result<MiraValue> {
        if spreads.is_empty() {
            let arguments = registers
                .iter()
                .map(|register| self.read_register(frame, *register))
                .collect::<Result<Vec<_>>>()?;
            return self.call(target, &arguments);
        }

        let mut arguments = Vec::with_capacity(registers.len());
        for (index, register) in registers.iter().enumerate() {
            let value = self.read_register(frame, *register)?;
            if spreads.contains(&index) {
                arguments.extend(
                    operations::array_spread(self, value)?
                        .into_iter()
                        .map(operations::into_element),
                );
            } else {
                arguments.push(value);
            }
        }
        self.call(target, &arguments)
    }

    pub(super) fn get_global_slot(&self, slot: usize) -> Result<MiraValue> {
        let name = &self.active_program().global_names[slot];
        self.globals.get(name).copied().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::UndefinedGlobal { name: name.clone() })
        })
    }

    pub(super) fn get_global_name(&self, key: &str) -> Result<MiraValue> {
        self.globals.get(key).copied().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::UndefinedGlobal {
                name: key.to_owned(),
            })
        })
    }

    pub(super) fn has_value(&mut self, value: MiraValue, key: MiraValue) -> Result<bool> {
        operations::has(self, value, key, None)
    }

    pub(super) fn get_value(&mut self, value: MiraValue, key: MiraValue) -> Result<MiraValue> {
        operations::get_value(self, value, key, None)
    }

    /// Call a function value owned by this Runtime.
    pub fn call(&mut self, function: MiraValue, args: &[MiraValue]) -> Result<MiraValue> {
        self.checkpoint_now()?;
        if self.call_depth >= self.options.max_call_depth {
            return Err(MiraError::runtime(RuntimeErrorKind::MaxCallDepth {
                max: self.options.max_call_depth,
            }));
        }
        let MiraValue::Function(handle) = function else {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: function.value_type(),
            }));
        };

        self.call_depth += 1;
        let callable = self.get_function_dyn(handle)?;
        self.call_stack.push(Some(Rc::from(callable.name())));
        let result = callable
            .call(self, args)
            .and_then(|value| self.insert(value))
            .and_then(|value| {
                self.checkpoint_now()?;
                Ok(value)
            });
        self.call_stack.pop();
        self.call_depth -= 1;
        result
    }

    pub(super) fn call_script(
        &mut self,
        function: &FunctionDef,
        parent: FrameId,
        args: &[MiraValue],
    ) -> Result<MiraValue> {
        let frame = self.create_frame(function.register_count, Some(parent));
        if function.variadic {
            let fixed = function.arg_count.saturating_sub(1);
            for index in 0..fixed {
                self.write_register(
                    frame,
                    index + 1,
                    args.get(index).copied().unwrap_or(MiraValue::Nil),
                );
            }
            let rest = args
                .iter()
                .skip(fixed)
                .copied()
                .map(operations::into_element)
                .collect::<Vec<_>>();
            if function.arg_count > 0 {
                let rest = self.insert(rest)?;
                self.write_register(frame, function.arg_count, rest);
            }
        } else {
            for index in 0..function.arg_count {
                self.write_register(
                    frame,
                    index + 1,
                    args.get(index).copied().unwrap_or(MiraValue::Nil),
                );
            }
        }
        match self.execute_block(&function.body, frame)? {
            Flow::Return(value) => Ok(value),
            Flow::Continue => Ok(MiraValue::Nil),
            Flow::Break | Flow::LoopContinue => {
                Err(MiraError::runtime(RuntimeErrorKind::InvalidControlFlow {
                    context: "function",
                }))
            }
        }
    }

    pub(super) fn checkpoint_now(&mut self) -> Result<()> {
        let remaining = self.checkpoint_remaining;
        if remaining > 1 {
            self.checkpoint_remaining = remaining - 1;
            return Ok(());
        }
        self.checkpoint_remaining = self.options.checkpoint_interval.max(1);
        if self.started.elapsed() >= self.options.timeout {
            return Err(MiraError::runtime(RuntimeErrorKind::Timeout));
        }
        Ok(())
    }

    pub(crate) fn checkpoint(&mut self) -> Result<()> {
        self.checkpoint_now()
    }
}

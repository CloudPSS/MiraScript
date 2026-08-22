use std::borrow::Cow;

use crate::{MiraManageable, bytecode::Program};

use super::*;

pub(crate) struct ScriptFunction {
    pub(crate) execution: ExecutionId,
    pub(crate) program: Rc<Program>,
    pub(crate) function: usize,
    pub(crate) frame: FrameId,
    pub(crate) name: Cow<'static, str>,
}

impl MiraFunction for ScriptFunction {
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable> {
        if self.execution != runtime.execution {
            return Err(MiraError::runtime(RuntimeErrorKind::ExecutionEnded));
        }
        let definition = &self.program.functions[self.function];
        let frame = runtime.create_frame(definition.register_count, Some(self.frame));
        if definition.variadic {
            let fixed = definition.arg_count.saturating_sub(1);
            for index in 0..fixed {
                runtime.write_register(
                    frame,
                    RegisterId::new(index + 1),
                    args.get(index).cloned().unwrap_or(MiraValue::nil()),
                );
            }
            let rest = args
                .iter()
                .skip(fixed)
                .cloned()
                .map(operations::into_element)
                .collect::<Vec<_>>();
            if definition.arg_count > 0 {
                let rest = runtime.insert(rest)?;
                runtime.write_register(frame, RegisterId::new(definition.arg_count), rest);
            }
        } else {
            for index in 0..definition.arg_count {
                runtime.write_register(
                    frame,
                    RegisterId::new(index + 1),
                    args.get(index).cloned().unwrap_or(MiraValue::nil()),
                );
            }
        }
        match runtime.execute_block(&definition.body, frame)? {
            Flow::Return(value) => Ok(value.into()),
            Flow::Continue => Ok(MiraValue::nil().into()),
            Flow::Break | Flow::LoopContinue => {
                Err(MiraError::runtime(RuntimeErrorKind::InvalidControlFlow {
                    context: "function",
                }))
            }
        }
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Runtime {
    pub(super) fn call_registers(
        &mut self,
        target: MiraValue,
        registers: &[RegisterId],
        spreads: &[usize],
        frame: FrameId,
    ) -> Result<MiraValue> {
        if spreads.is_empty() {
            let arguments = registers
                .iter()
                .cloned()
                .map(|register| self.read_register(frame, register))
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
        let (name, std_slot) = &self.active_program().global_names[slot];

        if let Some(value) = self.globals.get_hint(name, *std_slot) {
            Ok(value)
        } else {
            self.undefined_global(name)
        }
    }

    pub(super) fn get_global_name(&self, key: &str) -> Result<MiraValue> {
        if let Some(value) = self.globals.get(key) {
            Ok(value)
        } else {
            self.undefined_global(key)
        }
    }
    #[cold]
    #[inline(never)]
    fn undefined_global(&self, name: &str) -> Result<MiraValue> {
        Err(MiraError::runtime(RuntimeErrorKind::UndefinedGlobal {
            name: name.to_owned(),
        }))
    }

    pub(super) fn has_value(&mut self, value: MiraValue, key: MiraValue) -> Result<bool> {
        operations::has(self, value, key, None)
    }

    pub(super) fn get_value(&mut self, value: MiraValue, key: MiraValue) -> Result<MiraValue> {
        operations::get_value(self, value, key, None)
    }

    /// Call a function value owned by this Runtime.
    pub fn call(&mut self, function: MiraValue, args: &[MiraValue]) -> Result<MiraValue> {
        self.checkpoint()?;
        if self.call_stack.depth() >= self.options.max_call_depth as usize {
            return self.err_call_depth();
        }
        let MiraValueKind::Function(handle) = function.kind() else {
            return self.err_not_callable(function);
        };

        let callable = self.get_function_dyn(handle)?;
        self.call_stack.push(handle);
        let result = callable
            .call(self, args)
            .and_then(|value| self.insert(value));
        self.call_stack.pop();
        result
    }

    #[cold]
    #[inline(never)]
    fn err_call_depth(&self) -> Result<MiraValue> {
        Err(MiraError::runtime(RuntimeErrorKind::MaxCallDepth {
            max: self.options.max_call_depth,
        }))
    }
    #[cold]
    #[inline(never)]
    fn err_not_callable(&self, value: MiraValue) -> Result<MiraValue> {
        Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
            actual: value.value_type(),
        }))
    }

    pub(crate) fn checkpoint(&mut self) -> Result<()> {
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
}

use super::*;

impl<'a> Runtime<'a> {
    pub(super) fn call_registers(
        &mut self,
        target: &MiraAny,
        registers: &[usize],
        spreads: &[usize],
        frame: usize,
    ) -> Result<MiraAny> {
        let argument = |register: usize| self.call_argument(frame, register);

        if spreads.is_empty() {
            return match registers {
                [] => self.call(target, &[]),
                [a] => self.call(target, &[argument(*a)?]),
                [a, b] => self.call(target, &[argument(*a)?, argument(*b)?]),
                [a, b, c] => self.call(target, &[argument(*a)?, argument(*b)?, argument(*c)?]),
                [a, b, c, d] => self.call(
                    target,
                    &[argument(*a)?, argument(*b)?, argument(*c)?, argument(*d)?],
                ),
                _ => {
                    let arguments = registers
                        .iter()
                        .map(|register| argument(*register))
                        .collect::<Result<Vec<_>>>()?;
                    self.call(target, &arguments)
                }
            };
        }

        let mut arguments = Vec::with_capacity(registers.len());
        for (index, register) in registers.iter().enumerate() {
            let value = argument(*register)?;
            if spreads.contains(&index) {
                for item in operations::array_spread(&value)? {
                    arguments.push(item.into_element()?);
                }
            } else {
                arguments.push(value);
            }
        }
        self.call(target, &arguments)
    }

    pub(super) fn call_argument(&self, frame: usize, register: usize) -> Result<MiraAny> {
        let value = self.read_register(frame, register);
        operations::assert_initialized(&value)?;
        Ok(value)
    }

    pub(super) fn get_global_slot(&self, slot: usize) -> Result<MiraAny> {
        self.get_global_slot_ref(slot).cloned()
    }

    pub(super) fn get_global_slot_ref(&self, slot: usize) -> Result<&'a MiraAny> {
        self.globals.get_ref(slot).ok_or_else(|| {
            MiraError::runtime(format!(
                "Global variable '{}' is not defined.",
                self.program.global_names[slot]
            ))
            .into()
        })
    }

    pub(super) fn get_global_name(&self, key: &str) -> Result<MiraAny> {
        self.context.get(key).ok_or_else(|| {
            MiraError::runtime(format!("Global variable '{key}' is not defined.")).into()
        })
    }

    pub(super) fn has_value(&self, value: &MiraAny, key: &MiraAny) -> Result<bool> {
        if let MiraAny::Module(module) = value
            && let MiraModule::Script(module) = module.as_ref()
        {
            if module.execution != self.execution {
                return Err(MiraError::ExecutionEnded.into());
            }
            return Ok(module.exports.contains_key(&operations::to_string(key)?));
        }
        operations::has(value, key)
    }

    pub(super) fn get_value(&self, value: &MiraAny, key: &MiraAny) -> Result<MiraAny> {
        if let MiraAny::Module(module) = value
            && let MiraModule::Script(module) = module.as_ref()
        {
            if module.execution != self.execution {
                return Err(MiraError::ExecutionEnded.into());
            }
            let key = operations::to_string(key)?;
            let value = module
                .exports
                .get(&key)
                .map(|register| self.read_register(module.frame, *register))
                .unwrap_or(MiraAny::Nil);
            operations::assert_initialized(&value)?;
            return Ok(value);
        }
        operations::get_value(value, key)
    }

    pub(super) fn call(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.checkpoint_now()?;
        if self.call_depth >= self.options.max_call_depth {
            return Err(MiraError::MaxCallDepth {
                max: self.options.max_call_depth,
            }
            .into());
        }
        self.call_depth += 1;
        let result = match function {
            MiraAny::Function(function) => match function.as_ref() {
                MiraFunction::Native(function) => {
                    self.call_stack.push(Some(function.shared_name()));
                    let mut context = MiraCallContext { runtime: self };
                    let result = function.call(&mut context, args).and_then(|value| {
                        context.runtime.checkpoint()?;
                        Ok(value)
                    });
                    self.call_stack.pop();
                    result
                }
                MiraFunction::Script {
                    execution,
                    function,
                    frame,
                    name,
                } => {
                    if *execution != self.execution {
                        Err(MiraError::ExecutionEnded.into())
                    } else {
                        let definition = self.program.functions[*function].clone();
                        self.call_stack.push(name.clone());
                        let result = self.call_script(&definition, *frame, args);
                        self.call_stack.pop();
                        result
                    }
                }
            },
            _ => Err(MiraError::runtime(format!(
                "Value is not callable: {}",
                operations::display(function)
            ))
            .into()),
        };
        self.call_depth -= 1;
        result.map(|value| {
            if matches!(value, MiraAny::Uninitialized) {
                MiraAny::Nil
            } else {
                value
            }
        })
    }

    pub(super) fn call_script(
        &mut self,
        function: &FunctionDef,
        parent: usize,
        args: &[MiraAny],
    ) -> Result<MiraAny> {
        let frame = self.create_frame(function.register_count, Some(parent));
        if function.variadic {
            let fixed = function.arg_count.saturating_sub(1);
            for index in 0..fixed {
                self.write_register(
                    frame,
                    index + 1,
                    args.get(index).cloned().unwrap_or(MiraAny::Nil),
                );
            }
            let rest = args
                .iter()
                .skip(fixed)
                .cloned()
                .map(MiraAny::into_element)
                .collect::<Result<Vec<_>>>()?;
            if function.arg_count > 0 {
                self.write_register(frame, function.arg_count, MiraAny::Array(rest.into()));
            }
        } else {
            for index in 0..function.arg_count {
                self.write_register(
                    frame,
                    index + 1,
                    args.get(index).cloned().unwrap_or(MiraAny::Nil),
                );
            }
        }
        match self.execute_block(&function.body, frame)? {
            Flow::Return(value) => Ok(value),
            Flow::Continue => Ok(MiraAny::Nil),
            Flow::Break | Flow::LoopContinue => {
                Err(MiraError::runtime("invalid function control flow").into())
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
            return Err(MiraError::Timeout.into());
        }
        Ok(())
    }
}

impl NativeRuntime for Runtime<'_> {
    fn call_value(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.call(function, args)
    }

    fn get_value(&self, value: &MiraAny, key: &MiraAny) -> Result<MiraAny> {
        Runtime::get_value(self, value, key)
    }

    fn options(&self) -> &RunOptions {
        self.options
    }

    fn checkpoint(&mut self) -> Result<()> {
        self.checkpoint_now()
    }
}

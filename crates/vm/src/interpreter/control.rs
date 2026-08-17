use super::*;

impl<'a> Runtime<'a> {
    pub(super) fn create_frame(&mut self, register_count: usize, parent: Option<usize>) -> usize {
        self.frames.push(Frame {
            registers: vec![MiraAny::Uninitialized; register_count + 1],
            parent,
        })
    }

    pub(super) fn read_register(&self, frame: usize, register: usize) -> MiraAny {
        if register == 0 {
            MiraAny::Nil
        } else {
            self.frames.get(frame).registers[register].clone()
        }
    }

    #[inline]
    pub(super) fn read_number(&self, frame: usize, register: usize) -> Result<f64> {
        if register == 0 {
            return operations::to_number(&MiraAny::Nil);
        }
        match &self.frames.get(frame).registers[register] {
            MiraAny::Number(value) => Ok(*value),
            value => operations::to_number(value),
        }
    }

    pub(super) fn write_register(&mut self, frame: usize, register: usize, value: MiraAny) {
        if register != 0 {
            self.frames.get_mut(frame).registers[register] = value;
        }
    }

    pub(super) fn parent_frame(&self, mut frame: usize, level: usize) -> Result<usize> {
        for _ in 0..level {
            frame = self
                .frames
                .get(frame)
                .parent
                .ok_or_else(|| MiraError::runtime("invalid upvalue level"))?;
        }
        Ok(frame)
    }

    pub(super) fn execute_block(&mut self, body: &[Instruction], frame: usize) -> Result<Flow> {
        for instruction in body {
            let result = self
                .execute_instruction(instruction, frame)
                .map_err(|error| self.with_runtime_context(error, instruction.offset))?;
            if !matches!(result, Flow::Continue) {
                return Ok(result);
            }
        }
        Ok(Flow::Continue)
    }

    pub(super) fn with_runtime_context(
        &self,
        error: Box<MiraError>,
        offset: usize,
    ) -> Box<MiraError> {
        let display = |name: &Option<Rc<str>>| name.as_deref().unwrap_or("<anonymous>").to_owned();
        let function = self.call_stack.last().map(display);
        let stack = self.call_stack.iter().map(display).collect();
        error.with_runtime_context(function, offset, stack).into()
    }

    pub(super) fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        frame: usize,
    ) -> Result<Flow> {
        match &instruction.kind {
            InstructionKind::Op(operation) => self.execute_op(operation, frame),
            InstructionKind::Function {
                destination,
                function,
            } => {
                self.write_register(
                    frame,
                    *destination,
                    MiraAny::Function(
                        MiraFunction::Script {
                            execution: self.execution,
                            function: *function,
                            frame,
                            name: None,
                        }
                        .into(),
                    ),
                );
                Ok(Flow::Continue)
            }
            InstructionKind::If {
                condition,
                register,
                then_body,
                else_body,
            } => {
                let value = self.read_register(frame, *register);
                let matches = match condition {
                    Condition::Truthy => operations::to_boolean(&value)?,
                    Condition::Falsy => !operations::to_boolean(&value)?,
                    Condition::Initialized => value.is_initialized(),
                    Condition::Uninitialized => !value.is_initialized(),
                    Condition::Nil => matches!(value, MiraAny::Nil),
                    Condition::NonNil => !matches!(value, MiraAny::Nil),
                };
                self.execute_block(if matches { then_body } else { else_body }, frame)
            }
            InstructionKind::Loop {
                register_count,
                kind,
                body,
                reuse_frame,
            } => self.execute_loop(*register_count, kind, body, frame, *reuse_frame),
            InstructionKind::Record {
                destination,
                elements,
            } => {
                let value = self.build_record(elements, frame)?;
                self.write_register(frame, *destination, value);
                Ok(Flow::Continue)
            }
            InstructionKind::Array {
                destination,
                elements,
            } => {
                let value = self.build_array(elements, frame)?;
                self.write_register(frame, *destination, value);
                Ok(Flow::Continue)
            }
            InstructionKind::Module {
                destination,
                name,
                fields,
            } => {
                let module = ScriptModule {
                    execution: self.execution,
                    frame,
                    exports: fields.iter().cloned().collect(),
                    name: Rc::from(name.as_str()),
                };
                self.write_register(
                    frame,
                    *destination,
                    MiraAny::Module(MiraModule::Script(Rc::new(module)).into()),
                );
                Ok(Flow::Continue)
            }
        }
    }

    pub(super) fn execute_loop(
        &mut self,
        register_count: usize,
        kind: &LoopKind,
        body: &[Instruction],
        parent: usize,
        reuse_frame: bool,
    ) -> Result<Flow> {
        let mut reusable_frame = None;
        match kind {
            LoopKind::Infinite => loop {
                self.checkpoint_now()?;
                let frame =
                    self.loop_frame(register_count, parent, reuse_frame, &mut reusable_frame);
                match self.execute_block(body, frame)? {
                    Flow::Continue | Flow::LoopContinue => {}
                    Flow::Break => return Ok(Flow::Continue),
                    flow @ Flow::Return(_) => return Ok(flow),
                }
            },
            LoopKind::Iterable { value } => {
                let items = operations::iterable(&self.read_register(parent, *value))?;
                for item in items {
                    self.checkpoint_now()?;
                    let frame =
                        self.loop_frame(register_count, parent, reuse_frame, &mut reusable_frame);
                    self.write_register(frame, 1, item);
                    match self.execute_block(body, frame)? {
                        Flow::Continue | Flow::LoopContinue => {}
                        Flow::Break => break,
                        flow @ Flow::Return(_) => return Ok(flow),
                    }
                }
                Ok(Flow::Continue)
            }
            LoopKind::Range {
                start,
                end,
                exclusive,
            } => {
                let mut value = operations::to_number(&self.read_register(parent, *start))?;
                let end = operations::to_number(&self.read_register(parent, *end))?;
                while if *exclusive {
                    value < end
                } else {
                    value <= end
                } {
                    self.checkpoint_now()?;
                    let frame =
                        self.loop_frame(register_count, parent, reuse_frame, &mut reusable_frame);
                    self.write_register(frame, 1, MiraAny::Number(value));
                    match self.execute_block(body, frame)? {
                        Flow::Continue | Flow::LoopContinue => {}
                        Flow::Break => break,
                        flow @ Flow::Return(_) => return Ok(flow),
                    }
                    value += 1.0;
                }
                Ok(Flow::Continue)
            }
        }
    }

    pub(super) fn loop_frame(
        &mut self,
        register_count: usize,
        parent: usize,
        reuse_frame: bool,
        reusable_frame: &mut Option<usize>,
    ) -> usize {
        if let Some(frame) = *reusable_frame {
            self.frames.reset(frame, Some(parent));
            return frame;
        }
        let frame = self.create_frame(register_count, Some(parent));
        if reuse_frame {
            *reusable_frame = Some(frame);
        }
        frame
    }

    pub(super) fn build_record(&self, elements: &[RecordElement], frame: usize) -> Result<MiraAny> {
        let mut record = IndexMap::new();
        for element in elements {
            match element {
                RecordElement::Field {
                    key,
                    value,
                    optional,
                } => {
                    let value = self.read_register(frame, *value);
                    operations::assert_initialized(&value)?;
                    if *optional
                        && (matches!(value, MiraAny::Nil)
                            || matches!(
                                value,
                                MiraAny::Function(_) | MiraAny::Module(_) | MiraAny::Extern(_)
                            ))
                    {
                        continue;
                    }
                    let key = match key {
                        RecordKey::Constant(key) => key.clone(),
                        RecordKey::Dynamic(register) => {
                            operations::to_string(&self.read_register(frame, *register))?
                        }
                        RecordKey::Index(index) => index.to_string(),
                    };
                    record.insert(key, value.into_element()?);
                }
                RecordElement::Spread(register) => {
                    let spread = operations::record_spread(&self.read_register(frame, *register))?;
                    record.extend(spread);
                }
            }
        }
        Ok(MiraAny::Record(record.into()))
    }

    pub(super) fn build_array(&self, elements: &[ArrayElement], frame: usize) -> Result<MiraAny> {
        let mut array = Vec::new();
        for element in elements {
            match element {
                ArrayElement::Item(register) => {
                    array.push(self.read_register(frame, *register).into_element()?)
                }
                ArrayElement::Spread(register) => {
                    for item in operations::array_spread(&self.read_register(frame, *register))? {
                        array.push(item.into_element()?);
                    }
                }
                ArrayElement::Range {
                    start,
                    end,
                    exclusive,
                } => {
                    let start = match start {
                        RangeEndpoint::Constant(value) => MiraAny::Number(*value as f64),
                        RangeEndpoint::Dynamic(register) => self.read_register(frame, *register),
                    };
                    let end = match end {
                        RangeEndpoint::Constant(value) => MiraAny::Number(*value as f64),
                        RangeEndpoint::Dynamic(register) => self.read_register(frame, *register),
                    };
                    array.extend(operations::array_range(
                        &start,
                        &end,
                        *exclusive,
                        self.options.max_array_len.saturating_sub(array.len()),
                    )?);
                }
            }
            if array.len() > self.options.max_array_len {
                return Err(MiraError::runtime(format!(
                    "Array length exceeds maximum limit of {}",
                    self.options.max_array_len
                ))
                .into());
            }
        }
        Ok(MiraAny::Array(array.into()))
    }
}

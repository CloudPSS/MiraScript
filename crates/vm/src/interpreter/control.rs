use indexmap::IndexMap;

use super::*;
use crate::{MiraHandle, MiraManageable, bytecode::Program, value::ANONYMOUS_FN_NAME};

pub(crate) struct ScriptModule {
    pub(crate) execution: ExecutionId,
    pub(crate) _program: Rc<Program>,
    pub(crate) frame: FrameId,
    pub(crate) exports: IndexMap<String, RegisterId>,
    pub(crate) name: Rc<str>,
}

impl MiraModule for ScriptModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.exports.len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.exports.get_index_of(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        self.exports
            .get_index(index)
            .map(|(key, _)| key.as_str())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraModule>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        if self.execution != runtime.execution {
            return Err(MiraError::runtime(RuntimeErrorKind::ExecutionEnded));
        }
        let (_, register) = self
            .exports
            .get_index(index)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))?;
        runtime.read_register(self.frame, *register).map(Into::into)
    }
}

impl Runtime {
    pub(super) fn create_frame(
        &mut self,
        register_count: usize,
        parent: Option<FrameId>,
    ) -> FrameId {
        self.frames.push(register_count, parent)
    }

    pub(super) fn parent_frame(&self, mut frame: FrameId, level: usize) -> Result<FrameId> {
        for _ in 0..level {
            frame = self.frames.get(frame).parent.ok_or_else(|| {
                MiraError::runtime(RuntimeErrorKind::InvalidUpvalueLevel { level })
            })?;
        }
        Ok(frame)
    }

    pub(super) fn execute_block(&mut self, body: &[Instruction], frame: FrameId) -> Result<Flow> {
        for instruction in body {
            let result = match self.execute_instruction(instruction, frame) {
                Ok(result) => result,
                Err(error) => {
                    return Err(self.with_runtime_context(*error, instruction.offset));
                }
            };
            if !matches!(result, Flow::Continue) {
                return Ok(result);
            }
        }
        Ok(Flow::Continue)
    }

    #[cold]
    #[inline(never)]
    pub(super) fn with_runtime_context(&self, error: MiraError, offset: usize) -> Box<MiraError> {
        let display = |handle| {
            self.get_function_dyn(handle)
                .expect("call stack function handles must remain valid")
                .name()
                .to_owned()
        };
        let function = self.call_stack.last().map(&display);
        let stack = self.call_stack.iter().map(display).collect();
        Box::new(error.with_runtime_context(function, offset, stack))
    }

    pub(super) fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        frame: FrameId,
    ) -> Result<Flow> {
        match &instruction.kind {
            InstructionKind::Op(operation) => self.execute_op(operation, frame),
            InstructionKind::Function {
                destination,
                function,
            } => {
                let function = ScriptFunction {
                    execution: self.execution,
                    program: Rc::clone(self.active_program()),
                    function: *function,
                    frame,
                    name: ANONYMOUS_FN_NAME.into(),
                };
                let value = self.insert(MiraManageable::from_function(function))?;
                self.write_register(frame, *destination, value);
                Ok(Flow::Continue)
            }
            InstructionKind::If {
                condition,
                register,
                then_body,
                else_body,
            } => {
                let raw = self.read_register_raw(frame, *register);
                let matches = match condition {
                    Condition::Truthy => operations::to_boolean(raw.ok_or_else(|| {
                        MiraError::runtime(RuntimeErrorKind::UninitializedValue)
                    })?)?,
                    Condition::Falsy => !operations::to_boolean(raw.ok_or_else(|| {
                        MiraError::runtime(RuntimeErrorKind::UninitializedValue)
                    })?)?,
                    Condition::Initialized => raw.is_some(),
                    Condition::Uninitialized => raw.is_none(),
                    Condition::Nil => matches!(raw, Some(MiraValue::Nil)),
                    Condition::NonNil => !matches!(raw, Some(MiraValue::Nil)),
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
                    _program: Rc::clone(self.active_program()),
                    frame,
                    exports: fields.iter().map(|(f, r)| (f.clone(), *r)).collect(),
                    name: Rc::from(name.as_str()),
                };
                let value = self.insert(MiraManageable::from_module(module))?;
                self.write_register(frame, *destination, value);
                Ok(Flow::Continue)
            }
        }
    }

    pub(super) fn execute_loop(
        &mut self,
        register_count: usize,
        kind: &LoopKind,
        body: &[Instruction],
        parent: FrameId,
        reuse_frame: bool,
    ) -> Result<Flow> {
        let mut reusable_frame = None;
        match kind {
            LoopKind::Infinite => loop {
                self.checkpoint()?;
                let frame =
                    self.loop_frame(register_count, parent, reuse_frame, &mut reusable_frame);
                match self.execute_block(body, frame)? {
                    Flow::Continue | Flow::LoopContinue => {}
                    Flow::Break => return Ok(Flow::Continue),
                    flow @ Flow::Return(_) => return Ok(flow),
                }
            },
            LoopKind::Iterable { value } => {
                let value = self.read_register(parent, *value)?;
                let items = operations::iterable(self, value)?;
                for item in items {
                    self.checkpoint()?;
                    let frame =
                        self.loop_frame(register_count, parent, reuse_frame, &mut reusable_frame);
                    self.write_register(frame, RegisterId::new(1), item);
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
                let mut value = self.read_number(parent, *start)?;
                let end = self.read_number(parent, *end)?;
                while if *exclusive {
                    value < end
                } else {
                    value <= end
                } {
                    self.checkpoint()?;
                    let frame =
                        self.loop_frame(register_count, parent, reuse_frame, &mut reusable_frame);
                    self.write_register(frame, RegisterId::new(1), MiraValue::Number(value));
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

    fn loop_frame(
        &mut self,
        register_count: usize,
        parent: FrameId,
        reuse_frame: bool,
        reusable_frame: &mut Option<FrameId>,
    ) -> FrameId {
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

    fn build_record(&mut self, elements: &[RecordElement], frame: FrameId) -> Result<MiraValue> {
        let mut record = IndexMap::with_capacity(elements.len());
        for element in elements {
            match element {
                RecordElement::Field {
                    key,
                    value,
                    optional,
                } => {
                    let value = self.read_register(frame, *value)?;
                    if *optional
                        && matches!(
                            value,
                            MiraValue::Nil | MiraValue::Function(_) | MiraValue::Module(_)
                        )
                    {
                        continue;
                    }
                    let key = match key {
                        RecordKey::Constant(key) => key.clone(),
                        RecordKey::Dynamic(register) => {
                            let key = self.read_register(frame, *register)?;
                            operations::to_string(self, key)?
                        }
                        RecordKey::Index(index) => index.to_string(),
                    };
                    record.insert(key, operations::into_element(value));
                }
                RecordElement::Spread(register) => {
                    let value = self.read_register(frame, *register)?;
                    record.extend(operations::record_spread(self, value)?);
                }
            }
        }
        self.insert(record)
    }

    fn build_array(&mut self, elements: &[ArrayElement], frame: FrameId) -> Result<MiraValue> {
        let mut array = Vec::with_capacity(elements.len());
        for element in elements {
            match element {
                ArrayElement::Item(register) => array.push(operations::into_element(
                    self.read_register(frame, *register)?,
                )),
                ArrayElement::Spread(register) => {
                    let value = self.read_register(frame, *register)?;
                    array.extend(
                        operations::array_spread(self, value)?
                            .into_iter()
                            .map(operations::into_element),
                    );
                }
                ArrayElement::Range {
                    start,
                    end,
                    exclusive,
                } => {
                    let start = match start {
                        RangeEndpoint::Constant(value) => MiraValue::Number(*value as f64),
                        RangeEndpoint::Dynamic(register) => self.read_register(frame, *register)?,
                    };
                    let end = match end {
                        RangeEndpoint::Constant(value) => MiraValue::Number(*value as f64),
                        RangeEndpoint::Dynamic(register) => self.read_register(frame, *register)?,
                    };
                    array.extend(operations::array_range(
                        self,
                        start,
                        end,
                        *exclusive,
                        self.options.max_array_len.saturating_sub(array.len()),
                    )?);
                }
            }
            if array.len() > self.options.max_array_len {
                return Err(MiraError::runtime(RuntimeErrorKind::ArrayLimit {
                    requested: array.len(),
                    max: self.options.max_array_len,
                }));
            }
        }
        self.insert(array)
    }
}

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

use crate::bytecode::{
    AccessKey, AccessOperation, ArrayElement, AssertOperation, BinaryOperation, CallTarget,
    Condition, FunctionDef, Instruction, InstructionKind, LoopKind, NumericOperation, Operation,
    PickOmitOperation, Program, RangeEndpoint, RecordElement, RecordKey, SliceBound,
    UnaryOperation, UpvalueOperation,
};
use crate::value::{MiraCallContext, NativeRuntime, ScriptModule};
use crate::{
    MiraAny, MiraContext, MiraError, MiraFunction, MiraModule, Result, RunOptions, operations,
};
use indexmap::IndexMap;

static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);
const INLINE_CALL_DEPTH: usize = 8;
const INLINE_GLOBAL_SLOTS: usize = 8;

struct Frame {
    registers: Vec<MiraAny>,
    parent: Option<usize>,
}

struct FrameArena {
    root: Frame,
    children: Vec<Frame>,
}

impl FrameArena {
    fn new(root_register_count: usize) -> Self {
        Self {
            root: Frame {
                registers: vec![MiraAny::Uninitialized; root_register_count + 1],
                parent: None,
            },
            children: Vec::new(),
        }
    }

    fn push(&mut self, frame: Frame) -> usize {
        self.children.push(frame);
        self.children.len()
    }

    fn get(&self, frame: usize) -> &Frame {
        if frame == 0 {
            &self.root
        } else {
            &self.children[frame - 1]
        }
    }

    fn get_mut(&mut self, frame: usize) -> &mut Frame {
        if frame == 0 {
            &mut self.root
        } else {
            &mut self.children[frame - 1]
        }
    }

    fn reset(&mut self, frame: usize, parent: Option<usize>) {
        let frame = self.get_mut(frame);
        frame.registers.fill(MiraAny::Uninitialized);
        frame.parent = parent;
    }
}

struct CallStack {
    inline: [Option<Rc<str>>; INLINE_CALL_DEPTH],
    overflow: Vec<Option<Rc<str>>>,
    len: usize,
}

enum GlobalSlots<'a> {
    Empty,
    One(Option<&'a MiraAny>),
    Two([Option<&'a MiraAny>; 2]),
    Inline([Option<&'a MiraAny>; INLINE_GLOBAL_SLOTS]),
    Overflow(Vec<Option<&'a MiraAny>>),
}

impl<'a> GlobalSlots<'a> {
    fn new(names: &[String], context: &'a MiraContext) -> Self {
        if names.is_empty() {
            Self::Empty
        } else if names.len() == 1 {
            Self::One(context.get_ref(&names[0]))
        } else if names.len() == 2 {
            Self::Two(std::array::from_fn(|index| context.get_ref(&names[index])))
        } else if names.len() <= INLINE_GLOBAL_SLOTS {
            Self::Inline(std::array::from_fn(|index| {
                names.get(index).and_then(|name| context.get_ref(name))
            }))
        } else {
            Self::Overflow(names.iter().map(|name| context.get_ref(name)).collect())
        }
    }

    fn get_ref(&self, slot: usize) -> Option<&'a MiraAny> {
        match self {
            Self::Empty => None,
            Self::One(value) => *value,
            Self::Two(values) => values[slot],
            Self::Inline(values) => values[slot],
            Self::Overflow(values) => values[slot],
        }
    }
}

impl CallStack {
    fn new() -> Self {
        Self {
            inline: std::array::from_fn(|_| None),
            overflow: Vec::new(),
            len: 0,
        }
    }

    fn push(&mut self, name: Option<Rc<str>>) {
        if self.len < INLINE_CALL_DEPTH {
            self.inline[self.len] = name;
        } else {
            self.overflow.push(name);
        }
        self.len += 1;
    }

    fn pop(&mut self) {
        if self.len == 0 {
            return;
        }
        self.len -= 1;
        if self.len < INLINE_CALL_DEPTH {
            self.inline[self.len] = None;
        } else {
            self.overflow.pop();
        }
    }

    fn last(&self) -> Option<&Option<Rc<str>>> {
        if self.len == 0 {
            None
        } else if self.len <= INLINE_CALL_DEPTH {
            Some(&self.inline[self.len - 1])
        } else {
            self.overflow.last()
        }
    }

    fn iter(&self) -> impl Iterator<Item = &Option<Rc<str>>> {
        self.inline[..self.len.min(INLINE_CALL_DEPTH)]
            .iter()
            .chain(self.overflow.iter())
    }
}

#[derive(Debug)]
enum Flow {
    Continue,
    Break,
    LoopContinue,
    Return(MiraAny),
}

pub(crate) fn run(
    program: &Program,
    context: &MiraContext,
    options: &RunOptions,
) -> Result<MiraAny> {
    let execution = NEXT_EXECUTION_ID.fetch_add(1, AtomicOrdering::Relaxed);
    let mut runtime = Runtime {
        program,
        context,
        options,
        globals: GlobalSlots::new(&program.global_names, context),
        execution,
        started: Instant::now(),
        checkpoint_remaining: options.checkpoint_interval.max(1),
        call_depth: 0,
        frames: FrameArena::new(program.root.register_count),
        call_stack: CallStack::new(),
    };
    let result = match runtime.execute_block(&program.root.body, 0)? {
        Flow::Return(value) => value,
        Flow::Continue => MiraAny::Nil,
        Flow::Break | Flow::LoopContinue => {
            return Err(MiraError::runtime("invalid root control flow"));
        }
    };
    if result.contains_script_reference(execution) {
        return Err(MiraError::EscapingClosure);
    }
    Ok(result)
}

struct Runtime<'a> {
    program: &'a Program,
    context: &'a MiraContext,
    options: &'a RunOptions,
    globals: GlobalSlots<'a>,
    execution: u64,
    started: Instant,
    checkpoint_remaining: u32,
    call_depth: u32,
    frames: FrameArena,
    call_stack: CallStack,
}

impl<'a> Runtime<'a> {
    fn create_frame(&mut self, register_count: usize, parent: Option<usize>) -> usize {
        self.frames.push(Frame {
            registers: vec![MiraAny::Uninitialized; register_count + 1],
            parent,
        })
    }

    fn read_register(&self, frame: usize, register: usize) -> MiraAny {
        if register == 0 {
            MiraAny::Nil
        } else {
            self.frames.get(frame).registers[register].clone()
        }
    }

    #[inline]
    fn read_number(&self, frame: usize, register: usize) -> Result<f64> {
        if register == 0 {
            return operations::to_number(&MiraAny::Nil);
        }
        match &self.frames.get(frame).registers[register] {
            MiraAny::Number(value) => Ok(*value),
            value => operations::to_number(value),
        }
    }

    fn write_register(&mut self, frame: usize, register: usize, value: MiraAny) {
        if register != 0 {
            self.frames.get_mut(frame).registers[register] = value;
        }
    }

    fn parent_frame(&self, mut frame: usize, level: usize) -> Result<usize> {
        for _ in 0..level {
            frame = self
                .frames
                .get(frame)
                .parent
                .ok_or_else(|| MiraError::runtime("invalid upvalue level"))?;
        }
        Ok(frame)
    }

    fn execute_block(&mut self, body: &[Instruction], frame: usize) -> Result<Flow> {
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

    fn with_runtime_context(&self, error: MiraError, offset: usize) -> MiraError {
        let display = |name: &Option<Rc<str>>| name.as_deref().unwrap_or("<anonymous>").to_owned();
        let function = self.call_stack.last().map(display);
        let stack = self.call_stack.iter().map(display).collect();
        error.with_runtime_context(function, offset, stack)
    }

    fn execute_instruction(&mut self, instruction: &Instruction, frame: usize) -> Result<Flow> {
        match &instruction.kind {
            InstructionKind::Op(operation) => self.execute_op(operation, frame),
            InstructionKind::Function {
                destination,
                function,
            } => {
                self.write_register(
                    frame,
                    *destination,
                    MiraAny::Function(MiraFunction::Script {
                        execution: self.execution,
                        function: *function,
                        frame,
                        name: None,
                    }),
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
                    MiraAny::Module(MiraModule::Script(Rc::new(module))),
                );
                Ok(Flow::Continue)
            }
        }
    }

    fn execute_loop(
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

    fn loop_frame(
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

    fn build_record(&self, elements: &[RecordElement], frame: usize) -> Result<MiraAny> {
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
        Ok(MiraAny::Record(record))
    }

    fn build_array(&self, elements: &[ArrayElement], frame: usize) -> Result<MiraAny> {
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
                )));
            }
        }
        Ok(MiraAny::Array(array))
    }

    fn execute_op(&mut self, operation: &Operation, frame: usize) -> Result<Flow> {
        match operation {
            Operation::Noop => {}
            Operation::Break => return Ok(Flow::Break),
            Operation::Continue => return Ok(Flow::LoopContinue),
            Operation::Return { value } => {
                return Ok(Flow::Return(self.read_register(frame, *value)));
            }
            Operation::Constant {
                destination,
                constant,
            } => self.write_register(
                frame,
                *destination,
                self.program.constants[*constant].clone(),
            ),
            Operation::Uninit { destination } => {
                self.write_register(frame, *destination, MiraAny::Uninitialized)
            }
            Operation::Unary {
                kind,
                destination,
                value,
            } => {
                let value = self.read_register(frame, *value);
                let result = match kind {
                    UnaryOperation::Pos | UnaryOperation::Plus => {
                        MiraAny::Number(operations::to_number(&value)?)
                    }
                    UnaryOperation::Neg => MiraAny::Number(-operations::to_number(&value)?),
                    UnaryOperation::Not => MiraAny::Boolean(!operations::to_boolean(&value)?),
                    UnaryOperation::Type => MiraAny::String(value.type_name().into()),
                    UnaryOperation::ToBoolean => MiraAny::Boolean(operations::to_boolean(&value)?),
                    UnaryOperation::ToNumber => MiraAny::Number(operations::to_number(&value)?),
                    UnaryOperation::ToString => MiraAny::String(operations::to_string(&value)?),
                    UnaryOperation::IsBoolean
                    | UnaryOperation::IsNumber
                    | UnaryOperation::IsString
                    | UnaryOperation::IsRecord
                    | UnaryOperation::IsArray => {
                        operations::assert_initialized(&value)?;
                        MiraAny::Boolean(match kind {
                            UnaryOperation::IsBoolean => matches!(value, MiraAny::Boolean(_)),
                            UnaryOperation::IsNumber => matches!(value, MiraAny::Number(_)),
                            UnaryOperation::IsString => matches!(value, MiraAny::String(_)),
                            UnaryOperation::IsRecord => {
                                matches!(value, MiraAny::Record(_) | MiraAny::RustRecord(_))
                            }
                            UnaryOperation::IsArray => {
                                matches!(value, MiraAny::Array(_) | MiraAny::RustArray(_))
                            }
                            _ => unreachable!(),
                        })
                    }
                    UnaryOperation::Assign => value,
                    UnaryOperation::Length => MiraAny::Number(operations::length(&value)? as f64),
                };
                self.write_register(frame, *destination, result);
            }
            Operation::Numeric {
                kind,
                destination,
                left,
                right,
            } => {
                let left = self.read_number(frame, *left)?;
                let right = self.read_number(frame, *right)?;
                let result = MiraAny::Number(match kind {
                    NumericOperation::Add => left + right,
                    NumericOperation::Sub => left - right,
                    NumericOperation::Mul => left * right,
                    NumericOperation::Div => left / right,
                    NumericOperation::Mod => left % right,
                    NumericOperation::Pow => left.powf(right),
                });
                self.write_register(frame, *destination, result);
            }
            Operation::Binary {
                kind,
                destination,
                left,
                right,
            } => {
                let left = self.read_register(frame, *left);
                let right = self.read_register(frame, *right);
                let result = match kind {
                    BinaryOperation::Eq
                    | BinaryOperation::Neq
                    | BinaryOperation::Same
                    | BinaryOperation::Nsame => {
                        operations::assert_initialized(&left)?;
                        operations::assert_initialized(&right)?;
                        let mut equal =
                            if matches!(kind, BinaryOperation::Eq | BinaryOperation::Neq) {
                                match (&left, &right) {
                                    (MiraAny::Number(a), MiraAny::Number(b)) => a == b,
                                    _ => left == right,
                                }
                            } else {
                                left == right
                            };
                        if matches!(kind, BinaryOperation::Neq | BinaryOperation::Nsame) {
                            equal = !equal;
                        }
                        MiraAny::Boolean(equal)
                    }
                    BinaryOperation::Aeq | BinaryOperation::Naeq => {
                        let mut equal = operations::approximately_equal(&left, &right)?;
                        if *kind == BinaryOperation::Naeq {
                            equal = !equal;
                        }
                        MiraAny::Boolean(equal)
                    }
                    BinaryOperation::Lt
                    | BinaryOperation::Lte
                    | BinaryOperation::Gt
                    | BinaryOperation::Gte => {
                        let ordering = operations::compare(&left, &right)?;
                        MiraAny::Boolean(match (kind, ordering) {
                            (_, None) => false,
                            (BinaryOperation::Lt, Some(value)) => value == Ordering::Less,
                            (BinaryOperation::Lte, Some(value)) => value != Ordering::Greater,
                            (BinaryOperation::Gt, Some(value)) => value == Ordering::Greater,
                            (BinaryOperation::Gte, Some(value)) => value != Ordering::Less,
                            _ => unreachable!(),
                        })
                    }
                    BinaryOperation::In => MiraAny::Boolean(operations::in_value(&left, &right)?),
                    BinaryOperation::And | BinaryOperation::Or => {
                        let left = operations::to_boolean(&left)?;
                        let right = operations::to_boolean(&right)?;
                        MiraAny::Boolean(if *kind == BinaryOperation::And {
                            left && right
                        } else {
                            left || right
                        })
                    }
                };
                self.write_register(frame, *destination, result);
            }
            Operation::Swap { left, right } => {
                let left_value = self.read_register(frame, *left);
                let right_value = self.read_register(frame, *right);
                self.write_register(frame, *left, right_value);
                self.write_register(frame, *right, left_value);
            }
            Operation::Upvalue {
                kind,
                value,
                level,
                register,
            } => {
                let owner = self.parent_frame(frame, *level)?;
                match kind {
                    UpvalueOperation::Get => {
                        let result = self.read_register(owner, *register);
                        operations::assert_initialized(&result)?;
                        self.write_register(frame, *value, result);
                    }
                    UpvalueOperation::Set => {
                        let result = self.read_register(frame, *value);
                        self.write_register(owner, *register, result);
                    }
                }
            }
            Operation::GetGlobal { destination, slot } => {
                let result = self.get_global_slot(*slot)?;
                self.write_register(frame, *destination, result);
            }
            Operation::GetGlobalDyn { destination, key } => {
                let key = operations::to_string(&self.read_register(frame, *key))?;
                let result = self.get_global_name(&key)?;
                self.write_register(frame, *destination, result);
            }
            Operation::InGlobal { destination, key } => {
                let key = operations::to_string(&self.read_register(frame, *key))?;
                self.write_register(
                    frame,
                    *destination,
                    MiraAny::Boolean(self.context.contains(&key)),
                );
            }
            Operation::Concat {
                destination,
                values,
            } => {
                let mut result = String::new();
                for value in values {
                    result.push_str(&operations::format_value(
                        &self.read_register(frame, *value),
                        None,
                    )?);
                }
                self.write_register(frame, *destination, MiraAny::String(result));
            }
            Operation::Format {
                destination,
                value,
                format,
            } => {
                let format = match &self.program.constants[*format] {
                    MiraAny::String(value) => Some(value.as_str()),
                    MiraAny::Nil => None,
                    _ => unreachable!("validated format constant"),
                };
                let result = operations::format_value(&self.read_register(frame, *value), format)?;
                self.write_register(frame, *destination, MiraAny::String(result));
            }
            Operation::Assert { kind, value } => {
                let value = self.read_register(frame, *value);
                match kind {
                    AssertOperation::Initialized => operations::assert_initialized(&value)?,
                    AssertOperation::NonNil => operations::assert_non_nil(&value)?,
                }
            }
            Operation::PickOmit {
                kind,
                destination,
                value,
                keys,
            } => {
                let keys: Result<Vec<_>> = keys
                    .iter()
                    .map(|index| operations::to_string(&self.program.constants[*index]))
                    .collect();
                let source = self.read_register(frame, *value);
                let result = match kind {
                    PickOmitOperation::Pick => operations::pick(&source, &keys?)?,
                    PickOmitOperation::Omit => operations::omit(&source, &keys?)?,
                };
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal0 { destination, slot } => {
                let target = self.get_global_slot_ref(*slot)?;
                let result = self.call(target, &[])?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal1 {
                destination,
                slot,
                argument,
            } => {
                let target = self.get_global_slot_ref(*slot)?;
                let argument = self.call_argument(frame, *argument)?;
                let result = self.call(target, &[argument])?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal1FromGlobal {
                destination,
                slot,
                argument_slot,
            } => {
                let target = self.get_global_slot_ref(*slot)?;
                let argument = self.get_global_slot_ref(*argument_slot)?;
                operations::assert_initialized(argument)?;
                let result = self.call(target, std::slice::from_ref(argument))?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal2 {
                destination,
                slot,
                arguments: [a, b],
            } => {
                let target = self.get_global_slot_ref(*slot)?;
                let arguments = [
                    self.call_argument(frame, *a)?,
                    self.call_argument(frame, *b)?,
                ];
                let result = self.call(target, &arguments)?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal3 {
                destination,
                slot,
                arguments: [a, b, c],
            } => {
                let target = self.get_global_slot_ref(*slot)?;
                let arguments = [
                    self.call_argument(frame, *a)?,
                    self.call_argument(frame, *b)?,
                    self.call_argument(frame, *c)?,
                ];
                let result = self.call(target, &arguments)?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal4 {
                destination,
                slot,
                arguments: [a, b, c, d],
            } => {
                let target = self.get_global_slot_ref(*slot)?;
                let arguments = [
                    self.call_argument(frame, *a)?,
                    self.call_argument(frame, *b)?,
                    self.call_argument(frame, *c)?,
                    self.call_argument(frame, *d)?,
                ];
                let result = self.call(target, &arguments)?;
                self.write_register(frame, *destination, result);
            }
            Operation::Call {
                destination,
                target,
                arguments,
                spreads,
            } => {
                let target = match target {
                    CallTarget::Global(constant) => self.get_global_slot(*constant)?,
                    CallTarget::Register(register) => self.read_register(frame, *register),
                };
                let result = self.call_registers(&target, arguments, spreads, frame)?;
                self.write_register(frame, *destination, result);
            }
            Operation::Access {
                kind,
                destination,
                value,
                key,
            } => {
                let key = match key {
                    AccessKey::Constant(constant) => self.program.constants[*constant].clone(),
                    AccessKey::Register(register) => self.read_register(frame, *register),
                    AccessKey::Index(index) => MiraAny::Number(*index as f64),
                };
                let source = self.read_register(frame, *value);
                match kind {
                    AccessOperation::Has => {
                        let result = MiraAny::Boolean(self.has_value(&source, &key)?);
                        self.write_register(frame, *destination, result);
                    }
                    AccessOperation::Get => {
                        let result = self.get_value(&source, &key)?;
                        self.write_register(frame, *destination, result);
                    }
                    AccessOperation::Set => {
                        let assigned = self.read_register(frame, *destination);
                        operations::set(&source, &key, assigned)?;
                    }
                }
            }
            Operation::Slice {
                destination,
                value,
                start,
                end,
                exclusive,
            } => {
                let bound = |bound: &SliceBound| match bound {
                    SliceBound::Constant(value) => MiraAny::Number(*value as f64),
                    SliceBound::Register(register) => self.read_register(frame, *register),
                };
                let start = start.as_ref().map(bound);
                let end = end.as_ref().map(bound);
                let result = operations::slice(
                    &self.read_register(frame, *value),
                    start.as_ref(),
                    end.as_ref(),
                    *exclusive,
                )?;
                self.write_register(frame, *destination, result);
            }
        }
        Ok(Flow::Continue)
    }

    fn call_registers(
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

    fn call_argument(&self, frame: usize, register: usize) -> Result<MiraAny> {
        let value = self.read_register(frame, register);
        operations::assert_initialized(&value)?;
        Ok(value)
    }

    fn get_global_slot(&self, slot: usize) -> Result<MiraAny> {
        self.get_global_slot_ref(slot).cloned()
    }

    fn get_global_slot_ref(&self, slot: usize) -> Result<&'a MiraAny> {
        self.globals.get_ref(slot).ok_or_else(|| {
            MiraError::runtime(format!(
                "Global variable '{}' is not defined.",
                self.program.global_names[slot]
            ))
        })
    }

    fn get_global_name(&self, key: &str) -> Result<MiraAny> {
        self.context
            .get(key)
            .ok_or_else(|| MiraError::runtime(format!("Global variable '{key}' is not defined.")))
    }

    fn has_value(&self, value: &MiraAny, key: &MiraAny) -> Result<bool> {
        if let MiraAny::Module(MiraModule::Script(module)) = value {
            if module.execution != self.execution {
                return Err(MiraError::ExecutionEnded);
            }
            return Ok(module.exports.contains_key(&operations::to_string(key)?));
        }
        operations::has(value, key)
    }

    fn get_value(&self, value: &MiraAny, key: &MiraAny) -> Result<MiraAny> {
        if let MiraAny::Module(MiraModule::Script(module)) = value {
            if module.execution != self.execution {
                return Err(MiraError::ExecutionEnded);
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

    fn call(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.checkpoint_now()?;
        if self.call_depth >= self.options.max_call_depth {
            return Err(MiraError::MaxCallDepth {
                max: self.options.max_call_depth,
            });
        }
        self.call_depth += 1;
        let result = match function {
            MiraAny::Function(MiraFunction::Native(function)) => {
                self.call_stack.push(Some(function.shared_name()));
                let mut context = MiraCallContext { runtime: self };
                let result = function.call(&mut context, args).and_then(|value| {
                    context.runtime.checkpoint()?;
                    Ok(value)
                });
                self.call_stack.pop();
                result
            }
            MiraAny::Function(MiraFunction::Script {
                execution,
                function,
                frame,
                name,
            }) => {
                if *execution != self.execution {
                    Err(MiraError::ExecutionEnded)
                } else {
                    let definition = self.program.functions[*function].clone();
                    self.call_stack.push(name.clone());
                    let result = self.call_script(&definition, *frame, args);
                    self.call_stack.pop();
                    result
                }
            }
            MiraAny::Extern(value) if value.is_callable()? => {
                let label = Rc::from(format!("<extern {}>", value.tag()?));
                self.call_stack.push(Some(label));
                let mut context = MiraCallContext { runtime: self };
                let result = value.call(&mut context, args).and_then(|value| {
                    context.runtime.checkpoint()?;
                    Ok(value)
                });
                self.call_stack.pop();
                result
            }
            _ => Err(MiraError::runtime(format!(
                "Value is not callable: {}",
                operations::display(function)
            ))),
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

    fn call_script(
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
                self.write_register(frame, function.arg_count, MiraAny::Array(rest));
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
                Err(MiraError::runtime("invalid function control flow"))
            }
        }
    }

    fn checkpoint_now(&mut self) -> Result<()> {
        let remaining = self.checkpoint_remaining;
        if remaining > 1 {
            self.checkpoint_remaining = remaining - 1;
            return Ok(());
        }
        self.checkpoint_remaining = self.options.checkpoint_interval.max(1);
        if self.started.elapsed() >= self.options.timeout {
            return Err(MiraError::Timeout);
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

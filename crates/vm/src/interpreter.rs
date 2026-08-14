use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

use indexmap::IndexMap;
use mira_core::OpCode;

use crate::bytecode::{
    ArrayElement, Condition, FunctionDef, Instruction, InstructionKind, LoopKind, Program,
    RangeEndpoint, RecordElement, RecordKey,
};
use crate::value::{MiraCallContext, NativeRuntime, ScriptModule};
use crate::{
    MiraAny, MiraContext, MiraError, MiraFunction, MiraModule, Result, RunOptions, operations,
};

static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);
const INLINE_CALL_DEPTH: usize = 8;

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
}

struct CallStack {
    inline: [Option<Rc<str>>; INLINE_CALL_DEPTH],
    overflow: Vec<Option<Rc<str>>>,
    len: usize,
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
    let runtime = Runtime {
        program,
        context,
        options,
        execution,
        started: Instant::now(),
        checkpoint_remaining: Cell::new(options.checkpoint_interval.max(1)),
        call_depth: Cell::new(0),
        frames: RefCell::new(FrameArena::new(program.root.register_count)),
        call_stack: RefCell::new(CallStack::new()),
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
    execution: u64,
    started: Instant,
    checkpoint_remaining: Cell<u32>,
    call_depth: Cell<u32>,
    frames: RefCell<FrameArena>,
    call_stack: RefCell<CallStack>,
}

impl Runtime<'_> {
    fn create_frame(&self, register_count: usize, parent: Option<usize>) -> usize {
        let mut frames = self.frames.borrow_mut();
        frames.push(Frame {
            registers: vec![MiraAny::Uninitialized; register_count + 1],
            parent,
        })
    }

    fn read_register(&self, frame: usize, register: usize) -> MiraAny {
        if register == 0 {
            MiraAny::Nil
        } else {
            self.frames.borrow().get(frame).registers[register].clone()
        }
    }

    fn write_register(&self, frame: usize, register: usize, value: MiraAny) {
        if register != 0 {
            self.frames.borrow_mut().get_mut(frame).registers[register] = value;
        }
    }

    fn parent_frame(&self, mut frame: usize, level: usize) -> Result<usize> {
        for _ in 0..level {
            frame = self
                .frames
                .borrow()
                .get(frame)
                .parent
                .ok_or_else(|| MiraError::runtime("invalid upvalue level"))?;
        }
        Ok(frame)
    }

    fn execute_block(&self, body: &[Instruction], frame: usize) -> Result<Flow> {
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
        let call_stack = self.call_stack.borrow();
        let display = |name: &Option<Rc<str>>| name.as_deref().unwrap_or("<anonymous>").to_owned();
        let function = call_stack.last().map(display);
        let stack = call_stack.iter().map(display).collect();
        error.with_runtime_context(function, offset, stack)
    }

    fn execute_instruction(&self, instruction: &Instruction, frame: usize) -> Result<Flow> {
        match &instruction.kind {
            InstructionKind::Op { opcode, params } => self.execute_op(*opcode, params, frame),
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
            } => self.execute_loop(*register_count, kind, body, frame),
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
        &self,
        register_count: usize,
        kind: &LoopKind,
        body: &[Instruction],
        parent: usize,
    ) -> Result<Flow> {
        match kind {
            LoopKind::Infinite => loop {
                self.checkpoint_now()?;
                let frame = self.create_frame(register_count, Some(parent));
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
                    let frame = self.create_frame(register_count, Some(parent));
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
                    let frame = self.create_frame(register_count, Some(parent));
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

    fn execute_op(&self, opcode: OpCode, params: &[i64], frame: usize) -> Result<Flow> {
        use OpCode::*;
        let reg = |index: usize| self.read_register(frame, params[index] as usize);
        let write = |index: usize, value: MiraAny| {
            self.write_register(frame, params[index] as usize, value);
        };
        match opcode {
            Noop => {}
            Break => return Ok(Flow::Break),
            Continue => return Ok(Flow::LoopContinue),
            Return => return Ok(Flow::Return(reg(0))),
            Constant => write(0, self.program.constants[params[1] as usize].clone()),
            Uninit => write(0, MiraAny::Uninitialized),
            Assign => write(0, reg(1)),
            Swap => {
                let left = reg(0);
                let right = reg(1);
                write(0, right);
                write(1, left);
            }
            GetUpvalue => {
                let owner = self.parent_frame(frame, params[1] as usize)?;
                let value = self.read_register(owner, params[2] as usize);
                operations::assert_initialized(&value)?;
                write(0, value);
            }
            SetUpvalue => {
                let owner = self.parent_frame(frame, params[1] as usize)?;
                self.write_register(owner, params[2] as usize, reg(0));
            }
            GetGlobal => {
                let key = self.constant_key(params[1] as usize)?;
                write(0, self.get_global(&key)?);
            }
            GetGlobalDyn => {
                let key = operations::to_string(&reg(1))?;
                write(0, self.get_global(&key)?);
            }
            InGlobal => {
                let key = operations::to_string(&reg(1))?;
                write(0, MiraAny::Boolean(self.context.contains(&key)));
            }
            Add | Sub | Mul | Div | Mod | Pow => {
                let left = operations::to_number(&reg(1))?;
                let right = operations::to_number(&reg(2))?;
                let value = match opcode {
                    Add => left + right,
                    Sub => left - right,
                    Mul => left * right,
                    Div => left / right,
                    Mod => left % right,
                    Pow => left.powf(right),
                    _ => unreachable!(),
                };
                write(0, MiraAny::Number(value));
            }
            Pos | Plus => write(0, MiraAny::Number(operations::to_number(&reg(1))?)),
            Neg => write(0, MiraAny::Number(-operations::to_number(&reg(1))?)),
            Not => write(0, MiraAny::Boolean(!operations::to_boolean(&reg(1))?)),
            And | Or => {
                let left = operations::to_boolean(&reg(1))?;
                let right = operations::to_boolean(&reg(2))?;
                write(
                    0,
                    MiraAny::Boolean(if opcode == And {
                        left && right
                    } else {
                        left || right
                    }),
                );
            }
            Eq | Neq | Same | Nsame => {
                let left = reg(1);
                let right = reg(2);
                operations::assert_initialized(&left)?;
                operations::assert_initialized(&right)?;
                let mut equal = if opcode == Eq || opcode == Neq {
                    match (&left, &right) {
                        (MiraAny::Number(a), MiraAny::Number(b)) => a == b,
                        _ => left == right,
                    }
                } else {
                    left == right
                };
                if opcode == Neq || opcode == Nsame {
                    equal = !equal;
                }
                write(0, MiraAny::Boolean(equal));
            }
            Aeq | Naeq => {
                let mut equal = operations::approximately_equal(&reg(1), &reg(2))?;
                if opcode == Naeq {
                    equal = !equal;
                }
                write(0, MiraAny::Boolean(equal));
            }
            Lt | Lte | Gt | Gte => {
                let ordering = operations::compare(&reg(1), &reg(2))?;
                let result = match (opcode, ordering) {
                    (_, None) => false,
                    (Lt, Some(value)) => value == Ordering::Less,
                    (Lte, Some(value)) => value != Ordering::Greater,
                    (Gt, Some(value)) => value == Ordering::Greater,
                    (Gte, Some(value)) => value != Ordering::Less,
                    _ => unreachable!(),
                };
                write(0, MiraAny::Boolean(result));
            }
            In => write(0, MiraAny::Boolean(operations::in_value(&reg(1), &reg(2))?)),
            Concat => {
                let count = params[1] as usize;
                let mut result = String::new();
                for index in 0..count {
                    result.push_str(&operations::format_value(&reg(index + 2), None)?);
                }
                write(0, MiraAny::String(result));
            }
            Format => {
                let format = match &self.program.constants[params[2] as usize] {
                    MiraAny::String(value) => Some(value.as_str()),
                    MiraAny::Nil => None,
                    _ => unreachable!("validated format constant"),
                };
                write(
                    0,
                    MiraAny::String(operations::format_value(&reg(1), format)?),
                );
            }
            AssertInit => operations::assert_initialized(&reg(0))?,
            AssertNonNil => operations::assert_non_nil(&reg(0))?,
            Type => write(0, MiraAny::String(reg(1).type_name().into())),
            ToBoolean => write(0, MiraAny::Boolean(operations::to_boolean(&reg(1))?)),
            ToNumber => write(0, MiraAny::Number(operations::to_number(&reg(1))?)),
            ToString => write(0, MiraAny::String(operations::to_string(&reg(1))?)),
            IsBoolean | IsNumber | IsString | IsRecord | IsArray => {
                let value = reg(1);
                operations::assert_initialized(&value)?;
                let result = match opcode {
                    IsBoolean => matches!(value, MiraAny::Boolean(_)),
                    IsNumber => matches!(value, MiraAny::Number(_)),
                    IsString => matches!(value, MiraAny::String(_)),
                    IsRecord => matches!(value, MiraAny::Record(_) | MiraAny::RustRecord(_)),
                    IsArray => matches!(value, MiraAny::Array(_) | MiraAny::RustArray(_)),
                    _ => unreachable!(),
                };
                write(0, MiraAny::Boolean(result));
            }
            Pick | Omit => {
                let count = params[2] as usize;
                let keys: Result<Vec<_>> = params[3..3 + count]
                    .iter()
                    .map(|index| operations::to_string(&self.program.constants[*index as usize]))
                    .collect();
                let value = if opcode == Pick {
                    operations::pick(&reg(1), &keys?)?
                } else {
                    operations::omit(&reg(1), &keys?)?
                };
                write(0, value);
            }
            Call | CallDyn => {
                let target = if opcode == Call {
                    let key = self.constant_key(params[1] as usize)?;
                    self.get_global(&key)?
                } else {
                    reg(1)
                };
                let arg_count = params[2] as usize;
                let raw_args = &params[3..3 + arg_count];
                let spread_count_index = 3 + arg_count;
                let spread_count = params[spread_count_index] as usize;
                let spreads =
                    &params[spread_count_index + 1..spread_count_index + 1 + spread_count];
                let result = self.call_registers(&target, raw_args, spreads, frame)?;
                write(0, result);
            }
            Has | HasDyn | HasIndex => {
                let key = match opcode {
                    Has => self.program.constants[params[2] as usize].clone(),
                    HasDyn => reg(2),
                    HasIndex => MiraAny::Number(params[2] as f64),
                    _ => unreachable!(),
                };
                write(0, MiraAny::Boolean(self.has_value(&reg(1), &key)?));
            }
            Get | GetDyn | GetIndex => {
                let key = match opcode {
                    Get => self.program.constants[params[2] as usize].clone(),
                    GetDyn => reg(2),
                    GetIndex => MiraAny::Number(params[2] as f64),
                    _ => unreachable!(),
                };
                write(0, self.get_value(&reg(1), &key)?);
            }
            Set | SetDyn | SetIndex => {
                let key = match opcode {
                    Set => self.program.constants[params[2] as usize].clone(),
                    SetDyn => reg(2),
                    SetIndex => MiraAny::Number(params[2] as f64),
                    _ => unreachable!(),
                };
                operations::set(&reg(1), &key, reg(0))?;
            }
            Slice => write(
                0,
                operations::slice(
                    &reg(1),
                    Some(&MiraAny::Number(params[2] as f64)),
                    Some(&MiraAny::Number(params[3] as f64)),
                    false,
                )?,
            ),
            SliceStart => write(
                0,
                operations::slice(
                    &reg(1),
                    None,
                    Some(&MiraAny::Number(params[2] as f64)),
                    false,
                )?,
            ),
            SliceEnd => write(
                0,
                operations::slice(
                    &reg(1),
                    Some(&MiraAny::Number(params[2] as f64)),
                    None,
                    false,
                )?,
            ),
            SliceDyn | SliceExclusiveDyn => write(
                0,
                operations::slice(
                    &reg(1),
                    Some(&reg(2)),
                    Some(&reg(3)),
                    opcode == SliceExclusiveDyn,
                )?,
            ),
            Length => write(0, MiraAny::Number(operations::length(&reg(1))? as f64)),
            _ => return Err(MiraError::runtime(format!("unimplemented opcode {opcode}"))),
        }
        Ok(Flow::Continue)
    }

    fn constant_key(&self, index: usize) -> Result<Cow<'_, str>> {
        match &self.program.constants[index] {
            MiraAny::String(value) => Ok(Cow::Borrowed(value)),
            value => operations::to_string(value).map(Cow::Owned),
        }
    }

    fn call_registers(
        &self,
        target: &MiraAny,
        registers: &[i64],
        spreads: &[i64],
        frame: usize,
    ) -> Result<MiraAny> {
        let argument = |register: i64| {
            let value = self.read_register(frame, register as usize);
            operations::assert_initialized(&value)?;
            Ok(value)
        };

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
            if spreads.contains(&(index as i64)) {
                for item in operations::array_spread(&value)? {
                    arguments.push(item.into_element()?);
                }
            } else {
                arguments.push(value);
            }
        }
        self.call(target, &arguments)
    }

    fn get_global(&self, key: &str) -> Result<MiraAny> {
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

    fn call(&self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.checkpoint_now()?;
        if self.call_depth.get() >= self.options.max_call_depth {
            return Err(MiraError::MaxCallDepth {
                max: self.options.max_call_depth,
            });
        }
        self.call_depth.set(self.call_depth.get() + 1);
        let result = match function {
            MiraAny::Function(MiraFunction::Native(function)) => {
                self.call_stack
                    .borrow_mut()
                    .push(Some(function.shared_name()));
                let mut context = MiraCallContext { runtime: self };
                let result = function.call(&mut context, args).and_then(|value| {
                    context.runtime.checkpoint()?;
                    Ok(value)
                });
                self.call_stack.borrow_mut().pop();
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
                    self.call_stack.borrow_mut().push(name.clone());
                    let result = self.call_script(&definition, *frame, args);
                    self.call_stack.borrow_mut().pop();
                    result
                }
            }
            MiraAny::Extern(value) if value.is_callable()? => {
                let label = Rc::from(format!("<extern {}>", value.tag()?));
                self.call_stack.borrow_mut().push(Some(label));
                let mut context = MiraCallContext { runtime: self };
                let result = value.call(&mut context, args).and_then(|value| {
                    context.runtime.checkpoint()?;
                    Ok(value)
                });
                self.call_stack.borrow_mut().pop();
                result
            }
            _ => Err(MiraError::runtime(format!(
                "Value is not callable: {}",
                operations::display(function)
            ))),
        };
        self.call_depth.set(self.call_depth.get() - 1);
        result.map(|value| {
            if matches!(value, MiraAny::Uninitialized) {
                MiraAny::Nil
            } else {
                value
            }
        })
    }

    fn call_script(
        &self,
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

    fn checkpoint_now(&self) -> Result<()> {
        let remaining = self.checkpoint_remaining.get();
        if remaining > 1 {
            self.checkpoint_remaining.set(remaining - 1);
            return Ok(());
        }
        self.checkpoint_remaining
            .set(self.options.checkpoint_interval.max(1));
        if self.started.elapsed() >= self.options.timeout {
            return Err(MiraError::Timeout);
        }
        Ok(())
    }
}

impl NativeRuntime for Runtime<'_> {
    fn call_value(&self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.call(function, args)
    }

    fn get_value(&self, value: &MiraAny, key: &MiraAny) -> Result<MiraAny> {
        Runtime::get_value(self, value, key)
    }

    fn options(&self) -> &RunOptions {
        self.options
    }

    fn checkpoint(&self) -> Result<()> {
        self.checkpoint_now()
    }
}

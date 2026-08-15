use std::rc::Rc;

use indexmap::IndexMap;
use mira_core::OpCode;
use mira_core::prelude::*;

use crate::{MiraAny, MiraError, Result};

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub constants: Rc<[MiraAny]>,
    pub global_names: Rc<[String]>,
    pub root: FunctionDef,
    pub functions: Rc<[FunctionDef]>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    #[allow(dead_code)] // Retained for function-level diagnostics and future source maps.
    pub offset: usize,
    pub arg_count: usize,
    pub register_count: usize,
    pub variadic: bool,
    pub body: Rc<[Instruction]>,
}

#[derive(Debug, Clone)]
pub(crate) struct Instruction {
    pub offset: usize,
    pub kind: InstructionKind,
}

#[derive(Debug, Clone)]
pub(crate) enum InstructionKind {
    Op(Operation),
    Function {
        destination: usize,
        function: usize,
    },
    If {
        condition: Condition,
        register: usize,
        then_body: Rc<[Instruction]>,
        else_body: Rc<[Instruction]>,
    },
    Loop {
        register_count: usize,
        kind: LoopKind,
        body: Rc<[Instruction]>,
        reuse_frame: bool,
    },
    Record {
        destination: usize,
        elements: Vec<RecordElement>,
    },
    Array {
        destination: usize,
        elements: Vec<ArrayElement>,
    },
    Module {
        destination: usize,
        name: String,
        fields: Vec<(String, usize)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Noop,
    Break,
    Continue,
    Return {
        value: usize,
    },
    Constant {
        destination: usize,
        constant: usize,
    },
    Uninit {
        destination: usize,
    },
    Unary {
        kind: UnaryOperation,
        destination: usize,
        value: usize,
    },
    Binary {
        kind: BinaryOperation,
        destination: usize,
        left: usize,
        right: usize,
    },
    Swap {
        left: usize,
        right: usize,
    },
    Upvalue {
        kind: UpvalueOperation,
        value: usize,
        level: usize,
        register: usize,
    },
    GetGlobal {
        destination: usize,
        slot: usize,
    },
    GetGlobalDyn {
        destination: usize,
        key: usize,
    },
    InGlobal {
        destination: usize,
        key: usize,
    },
    Concat {
        destination: usize,
        values: Box<[usize]>,
    },
    Format {
        destination: usize,
        value: usize,
        format: usize,
    },
    Assert {
        kind: AssertOperation,
        value: usize,
    },
    PickOmit {
        kind: PickOmitOperation,
        destination: usize,
        value: usize,
        keys: Box<[usize]>,
    },
    Call {
        destination: usize,
        target: CallTarget,
        arguments: Box<[usize]>,
        spreads: Box<[usize]>,
    },
    Access {
        kind: AccessOperation,
        destination: usize,
        value: usize,
        key: AccessKey,
    },
    Slice {
        destination: usize,
        value: usize,
        start: Option<SliceBound>,
        end: Option<SliceBound>,
        exclusive: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnaryOperation {
    Pos,
    Neg,
    Not,
    Plus,
    Type,
    ToBoolean,
    ToNumber,
    ToString,
    IsBoolean,
    IsNumber,
    IsString,
    IsRecord,
    IsArray,
    Assign,
    Length,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Aeq,
    Naeq,
    Same,
    Nsame,
    In,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AssertOperation {
    Initialized,
    NonNil,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UpvalueOperation {
    Get,
    Set,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PickOmitOperation {
    Pick,
    Omit,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccessOperation {
    Has,
    Get,
    Set,
}

#[derive(Debug, Clone)]
pub(crate) enum CallTarget {
    Global(usize),
    Register(usize),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccessKey {
    Constant(usize),
    Register(usize),
    Index(i64),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SliceBound {
    Constant(i64),
    Register(usize),
}

fn block_may_capture_frame(body: &[Instruction]) -> bool {
    body.iter().any(|instruction| match &instruction.kind {
        InstructionKind::Function { .. } | InstructionKind::Module { .. } => true,
        InstructionKind::If {
            then_body,
            else_body,
            ..
        } => block_may_capture_frame(then_body) || block_may_capture_frame(else_body),
        InstructionKind::Loop { body, .. } => block_may_capture_frame(body),
        InstructionKind::Op(_) | InstructionKind::Record { .. } | InstructionKind::Array { .. } => {
            false
        }
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Condition {
    Truthy,
    Falsy,
    Initialized,
    Uninitialized,
    Nil,
    NonNil,
}

#[derive(Debug, Clone)]
pub(crate) enum LoopKind {
    Infinite,
    Iterable {
        value: usize,
    },
    Range {
        start: usize,
        end: usize,
        exclusive: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum RecordKey {
    Constant(String),
    Dynamic(usize),
    Index(i64),
}

#[derive(Debug, Clone)]
pub(crate) enum RecordElement {
    Field {
        key: RecordKey,
        value: usize,
        optional: bool,
    },
    Spread(usize),
}

#[derive(Debug, Clone)]
pub(crate) enum ArrayElement {
    Item(usize),
    Range {
        start: RangeEndpoint,
        end: RangeEndpoint,
        exclusive: bool,
    },
    Spread(usize),
}

#[derive(Debug, Clone)]
pub(crate) enum RangeEndpoint {
    Constant(i64),
    Dynamic(usize),
}

impl Program {
    pub fn decode(chunk: &[u8]) -> Result<Self> {
        let (code, constants) = decode_chunk(chunk)?;
        let mut decoder = Decoder {
            code,
            offset: 0,
            constants,
            functions: Vec::new(),
            global_names: IndexMap::new(),
            scopes: Vec::new(),
            loop_depth: 0,
        };

        let (opcode, wide, offset) = decoder.read_opcode()?;
        if !matches!(opcode, OpCode::Func | OpCode::FuncVarg) {
            return Err(decoder.invalid(offset, "root instruction must be Func"));
        }
        let root = decoder.read_function(opcode, wide, true, offset)?.1;
        if decoder.offset != decoder.code.len() {
            return Err(decoder.invalid(decoder.offset, "trailing code after root FuncEnd"));
        }

        Ok(Self {
            constants: Rc::from(decoder.constants),
            global_names: Rc::from(decoder.global_names.into_keys().collect::<Vec<_>>()),
            root,
            functions: Rc::from(decoder.functions),
        })
    }
}

fn decode_chunk(chunk: &[u8]) -> Result<(&[u8], Vec<MiraAny>)> {
    if chunk.len() < 12 {
        return Err(MiraError::InvalidBytecode {
            offset: chunk.len(),
            reason: "chunk header is truncated".into(),
        });
    }
    let chunk_len = read_u32(chunk, 0)? as usize;
    if chunk_len != chunk.len() - 4 {
        return Err(MiraError::InvalidBytecode {
            offset: 0,
            reason: format!(
                "chunk length header is {chunk_len}, expected {}",
                chunk.len() - 4
            ),
        });
    }
    let code_len = read_u32(chunk, 4)? as usize;
    let constants_len_offset =
        8usize
            .checked_add(code_len)
            .ok_or_else(|| MiraError::InvalidBytecode {
                offset: 4,
                reason: "code length overflow".into(),
            })?;
    if constants_len_offset
        .checked_add(4)
        .is_none_or(|end| end > chunk.len())
    {
        return Err(MiraError::InvalidBytecode {
            offset: 4,
            reason: "code section exceeds chunk".into(),
        });
    }
    let constants_len = read_u32(chunk, constants_len_offset)? as usize;
    let constants_offset = constants_len_offset + 4;
    if constants_offset
        .checked_add(constants_len)
        .is_none_or(|end| end != chunk.len())
    {
        return Err(MiraError::InvalidBytecode {
            offset: constants_len_offset,
            reason: "constant section length does not match chunk".into(),
        });
    }
    let constants = decode_constants(&chunk[constants_offset..], constants_offset)?;
    Ok((&chunk[8..8 + code_len], constants))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let data: [u8; 4] = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| MiraError::InvalidBytecode {
            offset,
            reason: "truncated u32".into(),
        })?
        .try_into()
        .expect("checked length");
    Ok(u32::from_le_bytes(data))
}

fn decode_constants(bytes: &[u8], base_offset: usize) -> Result<Vec<MiraAny>> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let tag_offset = offset;
        let tag = bytes[offset];
        offset += 1;
        let value = match tag {
            0 => MiraAny::Nil,
            1 => MiraAny::Boolean(true),
            2 => MiraAny::Boolean(false),
            3 => {
                let raw =
                    bytes
                        .get(offset..offset + 4)
                        .ok_or_else(|| MiraError::InvalidBytecode {
                            offset: base_offset + offset,
                            reason: "truncated ordinal constant".into(),
                        })?;
                offset += 4;
                MiraAny::Number(i32::from_le_bytes(raw.try_into().expect("checked length")) as f64)
            }
            4 => {
                let raw =
                    bytes
                        .get(offset..offset + 8)
                        .ok_or_else(|| MiraError::InvalidBytecode {
                            offset: base_offset + offset,
                            reason: "truncated number constant".into(),
                        })?;
                offset += 8;
                MiraAny::Number(f64::from_le_bytes(raw.try_into().expect("checked length")))
            }
            5 => {
                let raw =
                    bytes
                        .get(offset..offset + 4)
                        .ok_or_else(|| MiraError::InvalidBytecode {
                            offset: base_offset + offset,
                            reason: "truncated string length".into(),
                        })?;
                let length = u32::from_le_bytes(raw.try_into().expect("checked length")) as usize;
                offset += 4;
                let raw = bytes.get(offset..offset + length).ok_or_else(|| {
                    MiraError::InvalidBytecode {
                        offset: base_offset + offset,
                        reason: "truncated string constant".into(),
                    }
                })?;
                let value =
                    std::str::from_utf8(raw).map_err(|error| MiraError::InvalidBytecode {
                        offset: base_offset + offset + error.valid_up_to(),
                        reason: "invalid UTF-8 string constant".into(),
                    })?;
                offset += length;
                MiraAny::String(value.to_owned())
            }
            _ => {
                return Err(MiraError::InvalidBytecode {
                    offset: base_offset + tag_offset,
                    reason: format!("unknown constant tag {tag}"),
                });
            }
        };
        result.push(value);
    }
    Ok(result)
}

struct Decoder<'a> {
    code: &'a [u8],
    offset: usize,
    constants: Vec<MiraAny>,
    functions: Vec<FunctionDef>,
    global_names: IndexMap<String, ()>,
    scopes: Vec<usize>,
    loop_depth: usize,
}

impl Decoder<'_> {
    fn invalid(&self, offset: usize, reason: impl Into<String>) -> MiraError {
        MiraError::InvalidBytecode {
            offset,
            reason: reason.into(),
        }
    }

    fn read_opcode(&mut self) -> Result<(OpCode, bool, usize)> {
        let offset = self.offset;
        let raw = *self.code.get(self.offset).ok_or_else(|| {
            self.invalid(self.offset, "unexpected end of code while reading opcode")
        })?;
        self.offset += 1;
        let wide = raw & OpCode::WIDE_MASK != 0;
        let code = raw & !OpCode::WIDE_MASK;
        let opcode = OpCode::VARIANTS
            .get(code as usize)
            .copied()
            .ok_or_else(|| self.invalid(offset, format!("unknown opcode 0x{code:02x}")))?;
        Ok((opcode, wide, offset))
    }

    fn read_param(&mut self, wide: bool, instruction_offset: usize) -> Result<usize> {
        let width = if wide { 4 } else { 1 };
        let raw = self
            .code
            .get(self.offset..self.offset + width)
            .ok_or_else(|| {
                self.invalid(
                    instruction_offset,
                    format!("truncated parameter at code offset {}", self.offset),
                )
            })?;
        self.offset += width;
        Ok(if wide {
            u32::from_le_bytes(raw.try_into().expect("checked length")) as usize
        } else {
            raw[0] as usize
        })
    }

    fn read_index(&mut self, wide: bool, instruction_offset: usize) -> Result<i64> {
        let width = if wide { 4 } else { 1 };
        let raw = self
            .code
            .get(self.offset..self.offset + width)
            .ok_or_else(|| {
                self.invalid(
                    instruction_offset,
                    format!("truncated signed parameter at code offset {}", self.offset),
                )
            })?;
        self.offset += width;
        Ok(if wide {
            i32::from_le_bytes(raw.try_into().expect("checked length")) as i64
        } else {
            (raw[0] as i8) as i64
        })
    }

    fn read_register(&mut self, wide: bool, instruction_offset: usize) -> Result<usize> {
        let register = self.read_param(wide, instruction_offset)?;
        self.validate_register(register, instruction_offset)?;
        Ok(register)
    }

    fn validate_register(&self, register: usize, offset: usize) -> Result<()> {
        let max = self.scopes.last().copied().unwrap_or(0);
        if register > max {
            return Err(self.invalid(
                offset,
                format!("register {register} is out of range 0..={max}"),
            ));
        }
        Ok(())
    }

    fn read_constant(&mut self, wide: bool, instruction_offset: usize) -> Result<usize> {
        let index = self.read_param(wide, instruction_offset)?;
        if index >= self.constants.len() {
            return Err(self.invalid(
                instruction_offset,
                format!("constant {index} is out of range"),
            ));
        }
        Ok(index)
    }

    fn read_string_constant(&mut self, wide: bool, instruction_offset: usize) -> Result<String> {
        let index = self.read_constant(wide, instruction_offset)?;
        match &self.constants[index] {
            MiraAny::String(value) => Ok(value.clone()),
            _ => Err(self.invalid(
                instruction_offset,
                format!("constant {index} is not a string"),
            )),
        }
    }

    fn read_global_slot(&mut self, wide: bool, instruction_offset: usize) -> Result<usize> {
        let constant = self.read_constant(wide, instruction_offset)?;
        let name = crate::operations::to_string(&self.constants[constant])?;
        if let Some(slot) = self.global_names.get_index_of(&name) {
            return Ok(slot);
        }
        let slot = self.global_names.len();
        self.global_names.insert(name, ());
        Ok(slot)
    }

    fn read_function(
        &mut self,
        opcode: OpCode,
        wide: bool,
        root: bool,
        offset: usize,
    ) -> Result<(usize, FunctionDef)> {
        let destination = self.read_param(wide, offset)?;
        if root {
            if destination != 0 {
                return Err(self.invalid(offset, "root function destination must be register 0"));
            }
        } else {
            self.validate_register(destination, offset)?;
        }
        let arg_count = self.read_param(wide, offset)?;
        let register_count = self.read_param(wide, offset)?;
        if arg_count > register_count {
            return Err(self.invalid(
                offset,
                format!("function has {arg_count} arguments but only {register_count} registers"),
            ));
        }

        self.scopes.push(register_count);
        let (body, terminal) = self.read_block(&[OpCode::FuncEnd])?;
        self.scopes.pop();
        if terminal != OpCode::FuncEnd {
            unreachable!();
        }
        let function = FunctionDef {
            offset,
            arg_count,
            register_count,
            variadic: opcode == OpCode::FuncVarg,
            body: Rc::from(body),
        };
        Ok((destination, function))
    }

    fn read_block(&mut self, terminals: &[OpCode]) -> Result<(Vec<Instruction>, OpCode)> {
        let mut body = Vec::new();
        loop {
            if self.offset >= self.code.len() {
                return Err(self.invalid(self.offset, "unterminated bytecode block"));
            }
            let saved = self.offset;
            let (opcode, wide, offset) = self.read_opcode()?;
            if terminals.contains(&opcode) {
                if wide {
                    return Err(self.invalid(offset, "block terminator cannot use wide encoding"));
                }
                return Ok((body, opcode));
            }
            self.offset = saved;
            body.push(self.read_instruction()?);
        }
    }

    fn read_instruction(&mut self) -> Result<Instruction> {
        let (opcode, wide, offset) = self.read_opcode()?;
        let kind = match opcode {
            OpCode::Func | OpCode::FuncVarg => {
                let (destination, function) = self.read_function(opcode, wide, false, offset)?;
                let function_id = self.functions.len();
                self.functions.push(function);
                InstructionKind::Function {
                    destination,
                    function: function_id,
                }
            }
            OpCode::If
            | OpCode::IfNot
            | OpCode::IfInit
            | OpCode::IfNotInit
            | OpCode::IfNil
            | OpCode::IfNotNil => self.read_if(opcode, wide, offset)?,
            OpCode::Loop | OpCode::LoopFor | OpCode::LoopRange | OpCode::LoopRangeExclusive => {
                self.read_loop(opcode, wide, offset)?
            }
            OpCode::Record => self.read_record(wide, offset)?,
            OpCode::Array => self.read_array(wide, offset)?,
            OpCode::Module => self.read_module(wide, offset)?,
            OpCode::Else
            | OpCode::IfEnd
            | OpCode::LoopEnd
            | OpCode::FuncEnd
            | OpCode::Field
            | OpCode::FieldDyn
            | OpCode::FieldIndex
            | OpCode::FieldOpt
            | OpCode::FieldOptDyn
            | OpCode::FieldOptIndex
            | OpCode::Item
            | OpCode::ItemRange
            | OpCode::ItemRangeDyn
            | OpCode::ItemRangeExclusiveDyn
            | OpCode::Spread
            | OpCode::Freeze => {
                return Err(self.invalid(offset, format!("unexpected structural opcode {opcode}")));
            }
            _ => self.read_simple(opcode, wide, offset)?,
        };
        Ok(Instruction { offset, kind })
    }

    fn read_if(&mut self, opcode: OpCode, wide: bool, offset: usize) -> Result<InstructionKind> {
        let register = self.read_register(wide, offset)?;
        let condition = match opcode {
            OpCode::If => Condition::Truthy,
            OpCode::IfNot => Condition::Falsy,
            OpCode::IfInit => Condition::Initialized,
            OpCode::IfNotInit => Condition::Uninitialized,
            OpCode::IfNil => Condition::Nil,
            OpCode::IfNotNil => Condition::NonNil,
            _ => unreachable!(),
        };
        let (then_body, terminal) = self.read_block(&[OpCode::Else, OpCode::IfEnd])?;
        let else_body = if terminal == OpCode::Else {
            self.read_block(&[OpCode::IfEnd])?.0
        } else {
            Vec::new()
        };
        Ok(InstructionKind::If {
            condition,
            register,
            then_body: Rc::from(then_body),
            else_body: Rc::from(else_body),
        })
    }

    fn read_loop(&mut self, opcode: OpCode, wide: bool, offset: usize) -> Result<InstructionKind> {
        let register_count = self.read_param(wide, offset)?;
        let kind = match opcode {
            OpCode::Loop => LoopKind::Infinite,
            OpCode::LoopFor => {
                if register_count == 0 {
                    return Err(self.invalid(offset, "for loop requires an iteration register"));
                }
                LoopKind::Iterable {
                    value: self.read_register(wide, offset)?,
                }
            }
            OpCode::LoopRange | OpCode::LoopRangeExclusive => {
                if register_count == 0 {
                    return Err(self.invalid(offset, "range loop requires an iteration register"));
                }
                LoopKind::Range {
                    start: self.read_register(wide, offset)?,
                    end: self.read_register(wide, offset)?,
                    exclusive: opcode == OpCode::LoopRangeExclusive,
                }
            }
            _ => unreachable!(),
        };

        self.scopes.push(register_count);
        self.loop_depth += 1;
        let block = self.read_block(&[OpCode::LoopEnd]);
        self.loop_depth -= 1;
        self.scopes.pop();
        let body = block?.0;
        let reuse_frame = !block_may_capture_frame(&body);
        Ok(InstructionKind::Loop {
            register_count,
            kind,
            body: Rc::from(body),
            reuse_frame,
        })
    }

    fn read_record(&mut self, wide: bool, offset: usize) -> Result<InstructionKind> {
        let destination = self.read_register(wide, offset)?;
        let mut elements = Vec::new();
        loop {
            let (opcode, element_wide, element_offset) = self.read_opcode()?;
            match opcode {
                OpCode::Field | OpCode::FieldOpt => {
                    let key = self.read_string_constant(element_wide, element_offset)?;
                    let value = self.read_register(element_wide, element_offset)?;
                    elements.push(RecordElement::Field {
                        key: RecordKey::Constant(key),
                        value,
                        optional: opcode == OpCode::FieldOpt,
                    });
                }
                OpCode::FieldDyn | OpCode::FieldOptDyn => {
                    let key = self.read_register(element_wide, element_offset)?;
                    let value = self.read_register(element_wide, element_offset)?;
                    elements.push(RecordElement::Field {
                        key: RecordKey::Dynamic(key),
                        value,
                        optional: opcode == OpCode::FieldOptDyn,
                    });
                }
                OpCode::FieldIndex | OpCode::FieldOptIndex => {
                    let key = self.read_index(element_wide, element_offset)?;
                    let value = self.read_register(element_wide, element_offset)?;
                    elements.push(RecordElement::Field {
                        key: RecordKey::Index(key),
                        value,
                        optional: opcode == OpCode::FieldOptIndex,
                    });
                }
                OpCode::Spread => {
                    elements.push(RecordElement::Spread(
                        self.read_register(element_wide, element_offset)?,
                    ));
                }
                OpCode::Freeze => {
                    if element_wide {
                        return Err(self.invalid(element_offset, "Freeze cannot use wide encoding"));
                    }
                    break;
                }
                _ => {
                    return Err(self.invalid(
                        element_offset,
                        format!("opcode {opcode} is not valid inside a record"),
                    ));
                }
            }
        }
        Ok(InstructionKind::Record {
            destination,
            elements,
        })
    }

    fn read_array(&mut self, wide: bool, offset: usize) -> Result<InstructionKind> {
        let destination = self.read_register(wide, offset)?;
        let mut elements = Vec::new();
        loop {
            let (opcode, element_wide, element_offset) = self.read_opcode()?;
            match opcode {
                OpCode::Item => elements.push(ArrayElement::Item(
                    self.read_register(element_wide, element_offset)?,
                )),
                OpCode::ItemRange => {
                    let start = self.read_index(element_wide, element_offset)?;
                    let end = self.read_index(element_wide, element_offset)?;
                    elements.push(ArrayElement::Range {
                        start: RangeEndpoint::Constant(start),
                        end: RangeEndpoint::Constant(end),
                        exclusive: false,
                    });
                }
                OpCode::ItemRangeDyn | OpCode::ItemRangeExclusiveDyn => {
                    let start = self.read_register(element_wide, element_offset)?;
                    let end = self.read_register(element_wide, element_offset)?;
                    elements.push(ArrayElement::Range {
                        start: RangeEndpoint::Dynamic(start),
                        end: RangeEndpoint::Dynamic(end),
                        exclusive: opcode == OpCode::ItemRangeExclusiveDyn,
                    });
                }
                OpCode::Spread => elements.push(ArrayElement::Spread(
                    self.read_register(element_wide, element_offset)?,
                )),
                OpCode::Freeze => {
                    if element_wide {
                        return Err(self.invalid(element_offset, "Freeze cannot use wide encoding"));
                    }
                    break;
                }
                _ => {
                    return Err(self.invalid(
                        element_offset,
                        format!("opcode {opcode} is not valid inside an array"),
                    ));
                }
            }
        }
        Ok(InstructionKind::Array {
            destination,
            elements,
        })
    }

    fn read_module(&mut self, wide: bool, offset: usize) -> Result<InstructionKind> {
        let destination = self.read_register(wide, offset)?;
        let name_index = self.read_index(wide, offset)?;
        let name = usize::try_from(name_index)
            .ok()
            .and_then(|index| self.constants.get(index))
            .and_then(|value| match value {
                MiraAny::String(value) => Some(value.clone()),
                _ => None,
            })
            .ok_or_else(|| self.invalid(offset, "module name must reference a string constant"))?;
        let mut fields = Vec::new();
        loop {
            let (opcode, field_wide, field_offset) = self.read_opcode()?;
            match opcode {
                OpCode::Field => {
                    let key = self.read_string_constant(field_wide, field_offset)?;
                    let value = self.read_register(field_wide, field_offset)?;
                    if fields.iter().any(|(existing, _)| existing == &key) {
                        return Err(
                            self.invalid(field_offset, format!("duplicate module export `{key}`"))
                        );
                    }
                    fields.push((key, value));
                }
                OpCode::Freeze => {
                    if field_wide {
                        return Err(self.invalid(field_offset, "Freeze cannot use wide encoding"));
                    }
                    break;
                }
                _ => {
                    return Err(self.invalid(
                        field_offset,
                        format!("opcode {opcode} is not valid inside a module"),
                    ));
                }
            }
        }
        Ok(InstructionKind::Module {
            destination,
            name,
            fields,
        })
    }

    fn read_simple(
        &mut self,
        opcode: OpCode,
        wide: bool,
        offset: usize,
    ) -> Result<InstructionKind> {
        use OpCode::*;
        let operation = match opcode {
            Noop => {
                if wide {
                    return Err(self.invalid(offset, "Noop cannot use wide encoding"));
                }
                Operation::Noop
            }
            Break => {
                if wide {
                    return Err(self.invalid(offset, "control opcode cannot use wide encoding"));
                }
                if self.loop_depth == 0 {
                    return Err(self.invalid(offset, format!("{opcode} outside a loop")));
                }
                Operation::Break
            }
            Continue => {
                if wide {
                    return Err(self.invalid(offset, "control opcode cannot use wide encoding"));
                }
                if self.loop_depth == 0 {
                    return Err(self.invalid(offset, format!("{opcode} outside a loop")));
                }
                Operation::Continue
            }
            Add | Sub | Mul | Div | Mod | Pow | Eq | Neq | Lt | Lte | Gt | Gte | Aeq | Naeq
            | Same | Nsame | In | And | Or => {
                let kind = match opcode {
                    Add => BinaryOperation::Add,
                    Sub => BinaryOperation::Sub,
                    Mul => BinaryOperation::Mul,
                    Div => BinaryOperation::Div,
                    Mod => BinaryOperation::Mod,
                    Pow => BinaryOperation::Pow,
                    Eq => BinaryOperation::Eq,
                    Neq => BinaryOperation::Neq,
                    Lt => BinaryOperation::Lt,
                    Lte => BinaryOperation::Lte,
                    Gt => BinaryOperation::Gt,
                    Gte => BinaryOperation::Gte,
                    Aeq => BinaryOperation::Aeq,
                    Naeq => BinaryOperation::Naeq,
                    Same => BinaryOperation::Same,
                    Nsame => BinaryOperation::Nsame,
                    In => BinaryOperation::In,
                    And => BinaryOperation::And,
                    Or => BinaryOperation::Or,
                    _ => unreachable!(),
                };
                Operation::Binary {
                    kind,
                    destination: self.read_register(wide, offset)?,
                    left: self.read_register(wide, offset)?,
                    right: self.read_register(wide, offset)?,
                }
            }
            Pos | Neg | Not | Plus | Type | ToBoolean | ToNumber | ToString | IsBoolean
            | IsNumber | IsString | IsRecord | IsArray | Assign | Length => {
                let kind = match opcode {
                    Pos => UnaryOperation::Pos,
                    Neg => UnaryOperation::Neg,
                    Not => UnaryOperation::Not,
                    Plus => UnaryOperation::Plus,
                    Type => UnaryOperation::Type,
                    ToBoolean => UnaryOperation::ToBoolean,
                    ToNumber => UnaryOperation::ToNumber,
                    ToString => UnaryOperation::ToString,
                    IsBoolean => UnaryOperation::IsBoolean,
                    IsNumber => UnaryOperation::IsNumber,
                    IsString => UnaryOperation::IsString,
                    IsRecord => UnaryOperation::IsRecord,
                    IsArray => UnaryOperation::IsArray,
                    Assign => UnaryOperation::Assign,
                    Length => UnaryOperation::Length,
                    _ => unreachable!(),
                };
                Operation::Unary {
                    kind,
                    destination: self.read_register(wide, offset)?,
                    value: self.read_register(wide, offset)?,
                }
            }
            AssertInit | AssertNonNil => Operation::Assert {
                kind: if opcode == AssertInit {
                    AssertOperation::Initialized
                } else {
                    AssertOperation::NonNil
                },
                value: self.read_register(wide, offset)?,
            },
            Uninit => Operation::Uninit {
                destination: self.read_register(wide, offset)?,
            },
            Return => Operation::Return {
                value: self.read_register(wide, offset)?,
            },
            Swap => Operation::Swap {
                left: self.read_register(wide, offset)?,
                right: self.read_register(wide, offset)?,
            },
            GetGlobalDyn => Operation::GetGlobalDyn {
                destination: self.read_register(wide, offset)?,
                key: self.read_register(wide, offset)?,
            },
            Constant => Operation::Constant {
                destination: self.read_register(wide, offset)?,
                constant: self.read_constant(wide, offset)?,
            },
            GetGlobal => Operation::GetGlobal {
                destination: self.read_register(wide, offset)?,
                slot: self.read_global_slot(wide, offset)?,
            },
            GetUpvalue | SetUpvalue => {
                let value = self.read_register(wide, offset)?;
                let level = self.read_param(wide, offset)?;
                if level == 0 || level >= self.scopes.len() {
                    return Err(self.invalid(offset, format!("invalid upvalue level {level}")));
                }
                let register = self.read_param(wide, offset)?;
                let scope = self.scopes[self.scopes.len() - 1 - level];
                if register > scope {
                    return Err(self.invalid(
                        offset,
                        format!("upvalue register {register} is out of range 0..={scope}"),
                    ));
                }
                Operation::Upvalue {
                    kind: if opcode == GetUpvalue {
                        UpvalueOperation::Get
                    } else {
                        UpvalueOperation::Set
                    },
                    value,
                    level,
                    register,
                }
            }
            Format => {
                let destination = self.read_register(wide, offset)?;
                let value = self.read_register(wide, offset)?;
                let format = self.read_constant(wide, offset)?;
                if !matches!(self.constants[format], MiraAny::String(_) | MiraAny::Nil) {
                    return Err(self.invalid(offset, "format constant must be string or nil"));
                }
                Operation::Format {
                    destination,
                    value,
                    format,
                }
            }
            InGlobal => Operation::InGlobal {
                destination: self.read_register(wide, offset)?,
                key: self.read_register(wide, offset)?,
            },
            Concat => {
                let destination = self.read_register(wide, offset)?;
                let count = self.read_count(wide, offset)?;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    values.push(self.read_register(wide, offset)?);
                }
                Operation::Concat {
                    destination,
                    values: values.into_boxed_slice(),
                }
            }
            Pick | Omit => {
                let destination = self.read_register(wide, offset)?;
                let value = self.read_register(wide, offset)?;
                let count = self.read_count(wide, offset)?;
                let mut keys = Vec::with_capacity(count);
                for _ in 0..count {
                    keys.push(self.read_constant(wide, offset)?);
                }
                Operation::PickOmit {
                    kind: if opcode == Pick {
                        PickOmitOperation::Pick
                    } else {
                        PickOmitOperation::Omit
                    },
                    destination,
                    value,
                    keys: keys.into_boxed_slice(),
                }
            }
            Call | CallDyn => {
                let destination = self.read_register(wide, offset)?;
                let target = if opcode == Call {
                    CallTarget::Global(self.read_global_slot(wide, offset)?)
                } else {
                    CallTarget::Register(self.read_register(wide, offset)?)
                };
                let arg_count = self.read_count(wide, offset)?;
                let mut arguments = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    arguments.push(self.read_register(wide, offset)?);
                }
                let spread_count = self.read_count(wide, offset)?;
                let mut spreads = Vec::with_capacity(spread_count);
                for _ in 0..spread_count {
                    let spread = self.read_param(wide, offset)?;
                    if spread >= arg_count {
                        return Err(self.invalid(
                            offset,
                            format!("spread argument index {spread} is out of range"),
                        ));
                    }
                    spreads.push(spread);
                }
                Operation::Call {
                    destination,
                    target,
                    arguments: arguments.into_boxed_slice(),
                    spreads: spreads.into_boxed_slice(),
                }
            }
            Has | Get | Set | HasDyn | GetDyn | SetDyn | HasIndex | GetIndex | SetIndex => {
                let kind = match opcode {
                    Has | HasDyn | HasIndex => AccessOperation::Has,
                    Get | GetDyn | GetIndex => AccessOperation::Get,
                    Set | SetDyn | SetIndex => AccessOperation::Set,
                    _ => unreachable!(),
                };
                let destination = self.read_register(wide, offset)?;
                let value = self.read_register(wide, offset)?;
                let key = match opcode {
                    Has | Get | Set => AccessKey::Constant(self.read_constant(wide, offset)?),
                    HasDyn | GetDyn | SetDyn => {
                        AccessKey::Register(self.read_register(wide, offset)?)
                    }
                    HasIndex | GetIndex | SetIndex => {
                        AccessKey::Index(self.read_index(wide, offset)?)
                    }
                    _ => unreachable!(),
                };
                Operation::Access {
                    kind,
                    destination,
                    value,
                    key,
                }
            }
            Slice | SliceStart | SliceEnd | SliceDyn | SliceExclusiveDyn => {
                let destination = self.read_register(wide, offset)?;
                let value = self.read_register(wide, offset)?;
                let (start, end, exclusive) = match opcode {
                    Slice => (
                        Some(SliceBound::Constant(self.read_index(wide, offset)?)),
                        Some(SliceBound::Constant(self.read_index(wide, offset)?)),
                        false,
                    ),
                    SliceStart => (
                        None,
                        Some(SliceBound::Constant(self.read_index(wide, offset)?)),
                        false,
                    ),
                    SliceEnd => (
                        Some(SliceBound::Constant(self.read_index(wide, offset)?)),
                        None,
                        false,
                    ),
                    SliceDyn | SliceExclusiveDyn => (
                        Some(SliceBound::Register(self.read_register(wide, offset)?)),
                        Some(SliceBound::Register(self.read_register(wide, offset)?)),
                        opcode == SliceExclusiveDyn,
                    ),
                    _ => unreachable!(),
                };
                Operation::Slice {
                    destination,
                    value,
                    start,
                    end,
                    exclusive,
                }
            }
            _ => return Err(self.invalid(offset, format!("unsupported simple opcode {opcode}"))),
        };
        Ok(InstructionKind::Op(operation))
    }

    fn read_count(&mut self, wide: bool, offset: usize) -> Result<usize> {
        let count = self.read_param(wide, offset)?;
        let width = if wide { 4 } else { 1 };
        if count > self.code.len().saturating_sub(self.offset) / width {
            return Err(self.invalid(
                offset,
                format!("dynamic parameter count {count} exceeds code"),
            ));
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(code: &[u8], constants: &[u8]) -> Vec<u8> {
        let length = 4 + code.len() + 4 + constants.len();
        let mut chunk = Vec::with_capacity(length + 4);
        chunk.extend_from_slice(&(length as u32).to_le_bytes());
        chunk.extend_from_slice(&(code.len() as u32).to_le_bytes());
        chunk.extend_from_slice(code);
        chunk.extend_from_slice(&(constants.len() as u32).to_le_bytes());
        chunk.extend_from_slice(constants);
        chunk
    }

    fn root(body: &[u8], register_count: u8) -> Vec<u8> {
        let mut code = vec![OpCode::Func as u8, 0, 0, register_count];
        code.extend_from_slice(body);
        code.push(OpCode::FuncEnd as u8);
        code
    }

    #[test]
    fn rejects_truncated_header() {
        let error = Program::decode(&[0; 3]).unwrap_err();
        assert!(matches!(error, MiraError::InvalidBytecode { .. }));
    }

    #[test]
    fn decodes_compiler_output() {
        let (chunk, diagnostics) = mira_core::Compiler::compile("1 + 2", &mira_core::Config::new());
        assert!(diagnostics.is_empty());
        Program::decode(&chunk.unwrap()).unwrap();
    }

    #[test]
    fn decodes_every_constant_encoding() {
        let mut constants = vec![0, 1, 2, 3];
        constants.extend_from_slice(&(-7_i32).to_le_bytes());
        constants.push(4);
        constants.extend_from_slice(&1.25_f64.to_le_bytes());
        constants.push(5);
        constants.extend_from_slice(&1_u32.to_le_bytes());
        constants.push(b'x');
        let program = Program::decode(&chunk(&root(&[], 0), &constants)).unwrap();
        assert_eq!(
            program.constants.as_ref(),
            &[
                MiraAny::Nil,
                MiraAny::Boolean(true),
                MiraAny::Boolean(false),
                MiraAny::Number(-7.0),
                MiraAny::Number(1.25),
                MiraAny::String("x".into()),
            ],
        );
    }

    #[test]
    fn decodes_wide_registers_and_constants() {
        let wide = OpCode::WIDE_MASK;
        let mut code = vec![OpCode::Func as u8 | wide];
        code.extend_from_slice(&0_u32.to_le_bytes());
        code.extend_from_slice(&0_u32.to_le_bytes());
        code.extend_from_slice(&300_u32.to_le_bytes());
        code.push(OpCode::Constant as u8 | wide);
        code.extend_from_slice(&300_u32.to_le_bytes());
        code.extend_from_slice(&0_u32.to_le_bytes());
        code.push(OpCode::Return as u8 | wide);
        code.extend_from_slice(&300_u32.to_le_bytes());
        code.push(OpCode::FuncEnd as u8);
        let program = Program::decode(&chunk(&code, &[0])).unwrap();
        assert_eq!(program.root.register_count, 300);
    }

    #[test]
    fn rejects_malformed_constants() {
        for constants in [vec![99], vec![4, 0], vec![5, 1, 0, 0, 0, 0xff]] {
            assert!(matches!(
                Program::decode(&chunk(&root(&[], 0), &constants)),
                Err(MiraError::InvalidBytecode { .. })
            ));
        }
    }

    #[test]
    fn rejects_unknown_truncated_and_out_of_range_instructions() {
        let cases = [
            root(&[0x7f], 0),
            root(&[OpCode::Constant as u8 | OpCode::WIDE_MASK, 0], 1),
            root(&[OpCode::Constant as u8, 1, 0], 1),
            root(&[OpCode::Uninit as u8, 1], 0),
        ];
        for code in cases {
            assert!(matches!(
                Program::decode(&chunk(&code, &[])),
                Err(MiraError::InvalidBytecode { .. })
            ));
        }
    }

    #[test]
    fn rejects_illegal_nesting_and_wide_terminators() {
        for code in [
            root(&[OpCode::IfEnd as u8], 0),
            vec![
                OpCode::Func as u8,
                0,
                0,
                0,
                OpCode::FuncEnd as u8 | OpCode::WIDE_MASK,
            ],
        ] {
            assert!(matches!(
                Program::decode(&chunk(&code, &[])),
                Err(MiraError::InvalidBytecode { .. })
            ));
        }
    }

    #[test]
    fn loop_frame_reuse_excludes_captured_environments() {
        fn first_loop(body: &[Instruction]) -> Option<bool> {
            body.iter().find_map(|instruction| match &instruction.kind {
                InstructionKind::Loop { reuse_frame, .. } => Some(*reuse_frame),
                InstructionKind::If {
                    then_body,
                    else_body,
                    ..
                } => first_loop(then_body).or_else(|| first_loop(else_body)),
                _ => None,
            })
        }

        let decode = |source: &str| {
            let (chunk, diagnostics) =
                mira_core::Compiler::compile(source, &mira_core::Config::new());
            assert!(diagnostics.is_empty());
            Program::decode(&chunk.unwrap()).unwrap()
        };

        let scalar = decode("let mut total = 0; for value in 1..10 { total += value; } total");
        assert_eq!(first_loop(&scalar.root.body), Some(true));

        let captured =
            decode("let mut first = nil; for value in 1..2 { first = fn { value }; } first()");
        assert_eq!(first_loop(&captured.root.body), Some(false));
    }
}

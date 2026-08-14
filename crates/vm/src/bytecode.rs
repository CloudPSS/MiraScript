use std::cell::RefCell;
use std::rc::Rc;

use mira_core::OpCode;
use mira_core::prelude::*;

use crate::{MiraAny, MiraError, Result};

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub constants: Rc<[MiraAny]>,
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
    Op {
        opcode: OpCode,
        params: Vec<i64>,
    },
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
    scopes: Vec<usize>,
    loop_depth: usize,
}

#[derive(Default)]
struct ParamList(RefCell<Vec<i64>>);

impl ParamList {
    fn push(&self, value: i64) {
        self.0.borrow_mut().push(value);
    }

    fn into_vec(self) -> Vec<i64> {
        self.0.into_inner()
    }
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
        Ok(InstructionKind::Loop {
            register_count,
            kind,
            body: Rc::from(body),
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
        let params = ParamList::default();
        let reg = |this: &mut Self| -> Result<()> {
            params.push(this.read_register(wide, offset)? as i64);
            Ok(())
        };
        match opcode {
            Noop => {
                if wide {
                    return Err(self.invalid(offset, "Noop cannot use wide encoding"));
                }
            }
            Break | Continue => {
                if wide {
                    return Err(self.invalid(offset, "control opcode cannot use wide encoding"));
                }
                if self.loop_depth == 0 {
                    return Err(self.invalid(offset, format!("{opcode} outside a loop")));
                }
            }
            Add | Sub | Mul | Div | Mod | Pow | Eq | Neq | Lt | Lte | Gt | Gte | Aeq | Naeq
            | Same | Nsame | In | And | Or => {
                reg(self)?;
                reg(self)?;
                reg(self)?;
            }
            Pos | Neg | Not | Plus | Type | ToBoolean | ToNumber | ToString | IsBoolean
            | IsNumber | IsString | IsRecord | IsArray | Assign | Length => {
                reg(self)?;
                reg(self)?;
            }
            AssertInit | AssertNonNil | Uninit | Return => reg(self)?,
            Swap | GetGlobalDyn => {
                reg(self)?;
                reg(self)?;
            }
            Constant | GetGlobal => {
                reg(self)?;
                params.push(self.read_constant(wide, offset)? as i64);
            }
            GetUpvalue | SetUpvalue => {
                reg(self)?;
                let level = self.read_param(wide, offset)?;
                if level == 0 || level >= self.scopes.len() {
                    return Err(self.invalid(offset, format!("invalid upvalue level {level}")));
                }
                let up = self.read_param(wide, offset)?;
                let scope = self.scopes[self.scopes.len() - 1 - level];
                if up > scope {
                    return Err(self.invalid(
                        offset,
                        format!("upvalue register {up} is out of range 0..={scope}"),
                    ));
                }
                params.push(level as i64);
                params.push(up as i64);
            }
            Format => {
                reg(self)?;
                reg(self)?;
                let constant = self.read_constant(wide, offset)?;
                if !matches!(self.constants[constant], MiraAny::String(_) | MiraAny::Nil) {
                    return Err(self.invalid(offset, "format constant must be string or nil"));
                }
                params.push(constant as i64);
            }
            InGlobal => {
                reg(self)?;
                reg(self)?;
            }
            Concat => {
                reg(self)?;
                let count = self.read_count(wide, offset)?;
                params.push(count as i64);
                for _ in 0..count {
                    reg(self)?;
                }
            }
            Pick | Omit => {
                reg(self)?;
                reg(self)?;
                let count = self.read_count(wide, offset)?;
                params.push(count as i64);
                for _ in 0..count {
                    params.push(self.read_constant(wide, offset)? as i64);
                }
            }
            Call | CallDyn => {
                reg(self)?;
                if opcode == Call {
                    params.push(self.read_constant(wide, offset)? as i64);
                } else {
                    reg(self)?;
                }
                let arg_count = self.read_count(wide, offset)?;
                params.push(arg_count as i64);
                for _ in 0..arg_count {
                    reg(self)?;
                }
                let spread_count = self.read_count(wide, offset)?;
                params.push(spread_count as i64);
                for _ in 0..spread_count {
                    let spread = self.read_param(wide, offset)?;
                    if spread >= arg_count {
                        return Err(self.invalid(
                            offset,
                            format!("spread argument index {spread} is out of range"),
                        ));
                    }
                    params.push(spread as i64);
                }
            }
            Has | Get | Set => {
                reg(self)?;
                reg(self)?;
                params.push(self.read_constant(wide, offset)? as i64);
            }
            HasDyn | GetDyn | SetDyn => {
                reg(self)?;
                reg(self)?;
                reg(self)?;
            }
            HasIndex | GetIndex | SetIndex => {
                reg(self)?;
                reg(self)?;
                params.push(self.read_index(wide, offset)?);
            }
            Slice => {
                reg(self)?;
                reg(self)?;
                params.push(self.read_index(wide, offset)?);
                params.push(self.read_index(wide, offset)?);
            }
            SliceStart | SliceEnd => {
                reg(self)?;
                reg(self)?;
                params.push(self.read_index(wide, offset)?);
            }
            SliceDyn | SliceExclusiveDyn => {
                reg(self)?;
                reg(self)?;
                reg(self)?;
                reg(self)?;
            }
            _ => return Err(self.invalid(offset, format!("unsupported simple opcode {opcode}"))),
        }
        Ok(InstructionKind::Op {
            opcode,
            params: params.into_vec(),
        })
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
}

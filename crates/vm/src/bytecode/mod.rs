mod chunk;
mod model;
mod simple;
mod structured;

#[cfg(test)]
mod tests;

use std::rc::Rc;

use indexmap::IndexMap;
use mira_core::OpCode;
use mira_core::prelude::*;

use crate::{MiraAny, MiraError, Result};

use chunk::decode_chunk;
pub(crate) use model::*;

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
            let mut instruction = self.read_instruction()?;
            if let (
                Some(Instruction {
                    kind:
                        InstructionKind::Op(Operation::GetGlobal {
                            destination: loaded,
                            slot: argument_slot,
                        }),
                    ..
                }),
                InstructionKind::Op(Operation::CallGlobal1 {
                    destination,
                    slot,
                    argument,
                }),
            ) = (body.last(), &instruction.kind)
                && loaded == argument
            {
                instruction.kind = InstructionKind::Op(Operation::CallGlobal1FromGlobal {
                    destination: *destination,
                    slot: *slot,
                    argument_slot: *argument_slot,
                });
            }
            body.push(instruction);
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
}

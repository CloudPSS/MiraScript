use super::*;

impl Decoder<'_> {
    pub(super) fn read_simple(
        &mut self,
        opcode: OpCode,
        wide: bool,
        offset: usize,
    ) -> Result<InstructionKind> {
        use OpCode::*;
        let operation = match opcode {
            Noop => Operation::Noop,
            Break => {
                if self.loop_depth == 0 {
                    return Err(MiraError::invalid_bytecode(
                        offset,
                        InvalidBytecodeReason::UnexpectedOpCode(opcode),
                    ));
                }
                Operation::Break
            }
            Continue => {
                if self.loop_depth == 0 {
                    return Err(MiraError::invalid_bytecode(
                        offset,
                        InvalidBytecodeReason::UnexpectedOpCode(opcode),
                    ));
                }
                Operation::Continue
            }
            Add | Sub | Mul | Div | Mod | Pow => {
                let kind = match opcode {
                    Add => NumericOperation::Add,
                    Sub => NumericOperation::Sub,
                    Mul => NumericOperation::Mul,
                    Div => NumericOperation::Div,
                    Mod => NumericOperation::Mod,
                    Pow => NumericOperation::Pow,
                    _ => unreachable!(),
                };
                Operation::Numeric {
                    kind,
                    destination: self.read_register(wide, offset)?,
                    left: self.read_register(wide, offset)?,
                    right: self.read_register(wide, offset)?,
                }
            }
            Eq | Neq | Lt | Lte | Gt | Gte | Aeq | Naeq | Same | Nsame | In | And | Or => {
                let kind = match opcode {
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
                    return Err(MiraError::invalid_bytecode(
                        offset,
                        InvalidBytecodeReason::InvalidUpvalueLevel(level),
                    ));
                }
                let register = self.read_param(wide, offset)?;
                let scope = self.scopes[self.scopes.len() - 1 - level];
                if register > scope {
                    return Err(MiraError::invalid_bytecode(
                        offset,
                        InvalidBytecodeReason::RegisterIndexOutOfRange(register, scope),
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
                    return Err(MiraError::invalid_bytecode(
                        offset,
                        InvalidBytecodeReason::InvalidConstantType,
                    ));
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
                let count = self.read_param(wide, offset)?;
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
                let count = self.read_param(wide, offset)?;
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
                let arg_count = self.read_param(wide, offset)?;
                let mut arguments = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    arguments.push(self.read_register(wide, offset)?);
                }
                let spread_count = self.read_param(wide, offset)?;
                let mut spreads = Vec::with_capacity(spread_count);
                for _ in 0..spread_count {
                    let spread = self.read_param(wide, offset)?;
                    if spread >= arg_count {
                        return Err(MiraError::invalid_bytecode(
                            offset,
                            InvalidBytecodeReason::SpreadArgumentIndexOutOfRange(spread, arg_count),
                        ));
                    }
                    spreads.push(spread);
                }
                match (&target, arguments.as_slice(), spreads.is_empty()) {
                    (CallTarget::Global(slot), [], true) => Operation::CallGlobal0 {
                        destination,
                        slot: *slot,
                    },
                    (CallTarget::Global(slot), [argument], true) => Operation::CallGlobal1 {
                        destination,
                        slot: *slot,
                        argument: *argument,
                    },
                    (CallTarget::Global(slot), [a, b], true) => Operation::CallGlobal2 {
                        destination,
                        slot: *slot,
                        arguments: [*a, *b],
                    },
                    (CallTarget::Global(slot), [a, b, c], true) => Operation::CallGlobal3 {
                        destination,
                        slot: *slot,
                        arguments: [*a, *b, *c],
                    },
                    (CallTarget::Global(slot), [a, b, c, d], true) => Operation::CallGlobal4 {
                        destination,
                        slot: *slot,
                        arguments: [*a, *b, *c, *d],
                    },
                    _ => Operation::Call {
                        destination,
                        target,
                        arguments: arguments.into_boxed_slice(),
                        spreads: spreads.into_boxed_slice(),
                    },
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
            _ => {
                return Err(MiraError::invalid_bytecode(
                    offset,
                    InvalidBytecodeReason::UnexpectedOpCode(opcode),
                ));
            }
        };
        Ok(InstructionKind::Op(operation))
    }
}

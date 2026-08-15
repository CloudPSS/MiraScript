use super::*;

impl Decoder<'_> {
    pub(super) fn read_if(
        &mut self,
        opcode: OpCode,
        wide: bool,
        offset: usize,
    ) -> Result<InstructionKind> {
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

    pub(super) fn read_loop(
        &mut self,
        opcode: OpCode,
        wide: bool,
        offset: usize,
    ) -> Result<InstructionKind> {
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

    pub(super) fn read_record(&mut self, wide: bool, offset: usize) -> Result<InstructionKind> {
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

    pub(super) fn read_array(&mut self, wide: bool, offset: usize) -> Result<InstructionKind> {
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

    pub(super) fn read_module(&mut self, wide: bool, offset: usize) -> Result<InstructionKind> {
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
}

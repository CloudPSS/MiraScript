use std::cmp::Ordering;

use super::*;

impl Runtime {
    #[inline]
    fn execute_numeric(
        &mut self,
        frame: FrameId,
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
        operation: impl Fn(f64, f64) -> f64,
    ) -> Result<()> {
        let registers = &mut self.frames.get_mut(frame).registers;
        let left = registers.read(left);
        let right = registers.read(right);

        if let (Some(left), Some(right)) = (left.unwrap().as_number(), right.unwrap().as_number()) {
            let result = MiraValue::number(operation(left, right));
            if destination.is_nil() {
                return Ok(());
            }
            registers.get_mut(destination).replace(result);
            return Ok(());
        }

        // Convert left first to retain the existing error precedence.
        let left = operations::to_number(self, left.check()?)?;
        let right = operations::to_number(self, right.check()?)?;
        let result = MiraValue::number(operation(left, right));
        if destination.is_nil() {
            return Ok(());
        }
        self.write_register(frame, destination, result);
        Ok(())
    }

    pub(super) fn execute_op(&mut self, operation: &Operation, frame: FrameId) -> Result<Flow> {
        match operation {
            Operation::Noop => {}
            Operation::Break => return Ok(Flow::Break),
            Operation::Continue => return Ok(Flow::LoopContinue),
            Operation::Return { value } => {
                return Ok(Flow::Return(self.read_register(frame, *value)?));
            }
            Operation::Constant {
                destination,
                constant,
            } => {
                let value = self.materialize_constant(*constant)?;
                self.write_register(frame, *destination, value);
            }
            Operation::Uninit { destination } => self.clear_register(frame, *destination),
            Operation::Unary {
                kind,
                destination,
                value,
            } => {
                let value = self.read_register(frame, *value)?;
                let result = match kind {
                    UnaryOperation::Pos | UnaryOperation::Plus => {
                        MiraValue::number(operations::to_number(self, value)?)
                    }
                    UnaryOperation::Neg => MiraValue::number(-operations::to_number(self, value)?),
                    UnaryOperation::Not => MiraValue::boolean(!operations::to_boolean(value)?),
                    UnaryOperation::Type => self.insert(value.type_name().to_owned())?,
                    UnaryOperation::ToBoolean => MiraValue::boolean(operations::to_boolean(value)?),
                    UnaryOperation::ToNumber => {
                        MiraValue::number(operations::to_number(self, value)?)
                    }
                    UnaryOperation::ToString => {
                        let value = operations::to_string(self, value)?;
                        self.insert(value)?
                    }
                    UnaryOperation::IsBoolean
                    | UnaryOperation::IsNumber
                    | UnaryOperation::IsString
                    | UnaryOperation::IsRecord
                    | UnaryOperation::IsArray => MiraValue::boolean(match kind {
                        UnaryOperation::IsBoolean => value.is_boolean(),
                        UnaryOperation::IsNumber => value.is_number(),
                        UnaryOperation::IsString => value.is_string(),
                        UnaryOperation::IsRecord => value.is_record(),
                        UnaryOperation::IsArray => value.is_array(),
                        _ => unreachable!(),
                    }),
                    UnaryOperation::Assign => value,
                    UnaryOperation::Length => {
                        MiraValue::number(operations::length(self, value)? as f64)
                    }
                };
                self.write_register(frame, *destination, result);
            }
            Operation::Add {
                destination,
                left,
                right,
            } => self.execute_numeric(frame, *destination, *left, *right, |a, b| a + b)?,
            Operation::Sub {
                destination,
                left,
                right,
            } => self.execute_numeric(frame, *destination, *left, *right, |a, b| a - b)?,
            Operation::Mul {
                destination,
                left,
                right,
            } => self.execute_numeric(frame, *destination, *left, *right, |a, b| a * b)?,
            Operation::Div {
                destination,
                left,
                right,
            } => self.execute_numeric(frame, *destination, *left, *right, |a, b| a / b)?,
            Operation::Mod {
                destination,
                left,
                right,
            } => self.execute_numeric(frame, *destination, *left, *right, |a, b| a % b)?,
            Operation::Pow {
                destination,
                left,
                right,
            } => self.execute_numeric(frame, *destination, *left, *right, f64::powf)?,
            Operation::Binary {
                kind,
                destination,
                left,
                right,
            } => {
                let left = self.read_register(frame, *left)?;
                let right = self.read_register(frame, *right)?;
                let result = match kind {
                    BinaryOperation::Eq
                    | BinaryOperation::Neq
                    | BinaryOperation::Same
                    | BinaryOperation::Nsame => {
                        let mut equal =
                            if matches!(kind, BinaryOperation::Eq | BinaryOperation::Neq) {
                                operations::equal(self, left, right)?
                            } else {
                                operations::same_value(self, left, right)?
                            };
                        if matches!(kind, BinaryOperation::Neq | BinaryOperation::Nsame) {
                            equal = !equal;
                        }
                        MiraValue::boolean(equal)
                    }
                    BinaryOperation::Aeq | BinaryOperation::Naeq => {
                        let mut equal = operations::approximately_equal(self, left, right)?;
                        if *kind == BinaryOperation::Naeq {
                            equal = !equal;
                        }
                        MiraValue::boolean(equal)
                    }
                    BinaryOperation::Lt
                    | BinaryOperation::Lte
                    | BinaryOperation::Gt
                    | BinaryOperation::Gte => {
                        let ordering = operations::compare(self, left, right)?;
                        MiraValue::boolean(match (kind, ordering) {
                            (_, None) => false,
                            (BinaryOperation::Lt, Some(value)) => value == Ordering::Less,
                            (BinaryOperation::Lte, Some(value)) => value != Ordering::Greater,
                            (BinaryOperation::Gt, Some(value)) => value == Ordering::Greater,
                            (BinaryOperation::Gte, Some(value)) => value != Ordering::Less,
                            _ => unreachable!(),
                        })
                    }
                    BinaryOperation::In => {
                        MiraValue::boolean(operations::in_value(self, left, right)?)
                    }
                    BinaryOperation::And | BinaryOperation::Or => {
                        let left = operations::to_boolean(left)?;
                        let right = operations::to_boolean(right)?;
                        MiraValue::boolean(if *kind == BinaryOperation::And {
                            left && right
                        } else {
                            left || right
                        })
                    }
                };
                self.write_register(frame, *destination, result);
            }
            Operation::Swap { left, right } => {
                let left_value = self.read_register_raw(frame, *left);
                let right_value = self.read_register_raw(frame, *right);
                self.write_register_raw(frame, *left, right_value);
                self.write_register_raw(frame, *right, left_value);
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
                        let result = self.read_register(owner, *register)?;
                        self.write_register(frame, *value, result);
                    }
                    UpvalueOperation::Set => {
                        let result = self.read_register(frame, *value)?;
                        self.write_register(owner, *register, result);
                    }
                }
            }
            Operation::GetGlobal { destination, slot } => {
                let result = self.get_global_slot(*slot)?;
                self.write_register(frame, *destination, result);
            }
            Operation::GetGlobalDyn { destination, key } => {
                let key = self.read_register(frame, *key)?;
                let key = operations::to_string(self, key)?;
                let result = self.get_global_name(&key)?;
                self.write_register(frame, *destination, result);
            }
            Operation::InGlobal { destination, key } => {
                let key = self.read_register(frame, *key)?;
                let key = operations::to_string(self, key)?;
                self.write_register(
                    frame,
                    *destination,
                    MiraValue::boolean(self.globals.contains_key(&key)),
                );
            }
            Operation::Concat {
                destination,
                values,
            } => {
                let mut result = String::new();
                for value in values {
                    let value = self.read_register(frame, *value)?;
                    result.push_str(&operations::format_value(self, value, None)?);
                }
                let result = self.insert(result)?;
                self.write_register(frame, *destination, result);
            }
            Operation::Format {
                destination,
                value,
                format,
            } => {
                let format = self.constant_string(*format)?.map(str::to_owned);
                let value = self.read_register(frame, *value)?;
                let result = operations::format_value(self, value, format.as_deref())?;
                let result = self.insert(result)?;
                self.write_register(frame, *destination, result);
            }
            Operation::Assert { kind, value } => match kind {
                AssertOperation::Initialized => {
                    self.read_register(frame, *value)?;
                }
                AssertOperation::NonNil => {
                    operations::assert_non_nil(self.read_register(frame, *value)?)?;
                }
            },
            Operation::PickOmit {
                kind,
                destination,
                value,
                keys,
            } => {
                let keys = keys
                    .iter()
                    .map(|index| self.active_program().constants[*index].to_source_string())
                    .collect::<Vec<_>>();
                let source = self.read_register(frame, *value)?;
                let result = match kind {
                    PickOmitOperation::Pick => operations::pick(self, source, &keys)?,
                    PickOmitOperation::Omit => operations::omit(self, source, &keys)?,
                };
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal0 { destination, slot } => {
                let target = self.get_global_slot(*slot)?;
                let result = self.call(target, &[])?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal1 {
                destination,
                slot,
                argument,
            } => {
                let target = self.get_global_slot(*slot)?;
                let argument = self.read_register(frame, *argument)?;
                let result = self.call(target, &[argument])?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal1FromGlobal {
                destination,
                slot,
                argument_slot,
            } => {
                let target = self.get_global_slot(*slot)?;
                let argument = self.get_global_slot(*argument_slot)?;
                let result = self.call(target, &[argument])?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal2 {
                destination,
                slot,
                arguments: [a, b],
            } => {
                let target = self.get_global_slot(*slot)?;
                let arguments = [
                    self.read_register(frame, *a)?,
                    self.read_register(frame, *b)?,
                ];
                let result = self.call(target, &arguments)?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal3 {
                destination,
                slot,
                arguments: [a, b, c],
            } => {
                let target = self.get_global_slot(*slot)?;
                let arguments = [
                    self.read_register(frame, *a)?,
                    self.read_register(frame, *b)?,
                    self.read_register(frame, *c)?,
                ];
                let result = self.call(target, &arguments)?;
                self.write_register(frame, *destination, result);
            }
            Operation::CallGlobal4 {
                destination,
                slot,
                arguments: [a, b, c, d],
            } => {
                let target = self.get_global_slot(*slot)?;
                let arguments = [
                    self.read_register(frame, *a)?,
                    self.read_register(frame, *b)?,
                    self.read_register(frame, *c)?,
                    self.read_register(frame, *d)?,
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
                    CallTarget::Global(slot) => self.get_global_slot(*slot)?,
                    CallTarget::Register(register) => self.read_register(frame, *register)?,
                };
                let result = self.call_registers(target, arguments, spreads, frame)?;
                self.write_register(frame, *destination, result);
            }
            Operation::Access {
                kind,
                destination,
                value,
                key,
            } => {
                let key = match key {
                    AccessKey::Constant(constant) => self.materialize_constant(*constant)?,
                    AccessKey::Register(register) => self.read_register(frame, *register)?,
                    AccessKey::Index(index) => MiraValue::number(*index as f64),
                };
                let source = self.read_register(frame, *value)?;
                match kind {
                    AccessOperation::Has => {
                        let result = MiraValue::boolean(self.has_value(source, key)?);
                        self.write_register(frame, *destination, result);
                    }
                    AccessOperation::Get => {
                        let result = self.get_value(source, key)?;
                        self.write_register(frame, *destination, result);
                    }
                    AccessOperation::Set => {
                        let assigned = self.read_register(frame, *destination)?;
                        operations::set(self, source, key, assigned)?;
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
                let start = match start {
                    Some(SliceBound::Constant(value)) => Some(MiraValue::number(*value as f64)),
                    Some(SliceBound::Register(register)) => {
                        Some(self.read_register(frame, *register)?)
                    }
                    None => None,
                };
                let end = match end {
                    Some(SliceBound::Constant(value)) => Some(MiraValue::number(*value as f64)),
                    Some(SliceBound::Register(register)) => {
                        Some(self.read_register(frame, *register)?)
                    }
                    None => None,
                };
                let value = self.read_register(frame, *value)?;
                let result = operations::slice(self, value, start, end, *exclusive)?;
                self.write_register(frame, *destination, result);
            }
        }
        Ok(Flow::Continue)
    }
}

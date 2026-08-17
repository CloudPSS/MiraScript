use super::*;

impl Runtime<'_> {
    pub(super) fn execute_op(&mut self, operation: &Operation, frame: usize) -> Result<Flow> {
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
                    UnaryOperation::ToString => {
                        MiraAny::String(operations::to_string(&value)?.into())
                    }
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
                self.write_register(frame, *destination, MiraAny::String(result.into()));
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
                self.write_register(frame, *destination, MiraAny::String(result.into()));
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
}

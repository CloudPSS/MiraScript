use std::ops::Deref;

use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic},
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        ArrayElementBase, AstWalker,
        Pattern::{self, *},
        RecordElementBase, TokenRef,
    },
};

use super::{
    Emitter, OpCode, emitter_scope::check_variable_initialized, opcode::OpParam, opcode::Register,
    variable::BindType,
};

impl<'s> Emitter<'s> {
    pub fn declare_pattern(&mut self, pattern: &'s Pattern<'s>, bind_type: Option<BindType>) {
        match pattern {
            Grouping(op, pattern, cp) => {
                if matches!(
                    pattern.as_ref(),
                    Pattern::Grouping(..)
                        | Pattern::Literal(..)
                        | Pattern::Constant(..)
                        | Pattern::Discard(..)
                        | Pattern::Bind(..)
                        | Pattern::Record(..)
                        | Pattern::Array(..)
                ) {
                    self.diagnostics.push(SourceDiagnostic::new(
                        op.range(),
                        DiagnosticCode::UnnecessaryParentheses,
                    ));
                    self.diagnostics.push(SourceDiagnostic::new(
                        cp.range(),
                        DiagnosticCode::UnnecessaryParentheses,
                    ));
                }
                self.declare_pattern(pattern, bind_type)
            }
            Literal(_, _) => (),
            Constant(_) => (),
            Range(_, _, _) | Relation(_, _) => (), // It can only include a constant pattern
            Discard(_) | SpreadDiscard(_) => (),
            Bind(mut_token, id_token) => {
                if let Some(bind_type) = bind_type {
                    self.declare_variable(id_token, mut_token.is_some(), bind_type);
                }
            }
            Record(_, elements, _) => {
                for element in elements {
                    match element.deref() {
                        RecordElementBase::Named(_, _, pattern)
                        | RecordElementBase::InterpolateNamed(_, _, pattern)
                        | RecordElementBase::OmitNamed(_, pattern)
                        | RecordElementBase::Unnamed(pattern)
                        | RecordElementBase::Spread(_, pattern) => {
                            self.declare_pattern(pattern, bind_type);
                        }
                    }
                }
            }
            Array(_, elements, _) => {
                for element in elements {
                    match element.deref() {
                        ArrayElementBase::Element(pattern)
                        | ArrayElementBase::Spread(_, pattern) => {
                            self.declare_pattern(pattern, bind_type)
                        }
                    }
                }
            }
            And(left, _, right) | Or(left, _, right) => {
                self.declare_pattern(left, bind_type);
                self.declare_pattern(right, bind_type);
            }
            Not(_, pattern) => self.declare_pattern(pattern, bind_type),
            Unknown { .. } => (),
        }
    }

    fn emit_failed_pattern(&mut self, pattern: &'s Pattern<'s>, bind_type: Option<BindType>) {
        // This function is called from `emit_pattern`,
        // Do not emit diagnostics, initialization, or set markers
        match pattern {
            Grouping(_, pattern, _) => self.emit_failed_pattern(pattern, bind_type),
            Literal(_, _) => (),
            Constant(_) => (),
            Range(_, _, _) | Relation(_, _) => (), // It can only include a constant pattern
            Discard(_) | SpreadDiscard(_) => (),
            Bind(_, id_token) => {
                let Some(id) = id_token.to_id_name() else {
                    return;
                };
                let var = self.scopes.find_variable(id);
                let Some((level, variable)) = var else {
                    return;
                };
                if !variable.mutable() && bind_type.is_none() {
                } else if level == self.closures.len() {
                    let register = variable.register();
                    self.op_unary(register, OpCode::Assign, Register::EMPTY);
                } else {
                    let register = variable.register();
                    let level = self.closures.len() - level;
                    self.op_set_upvalue(Register::EMPTY, level, register);
                }
            }
            Record(_, elements, _) => {
                for element in elements {
                    match element.deref() {
                        RecordElementBase::Named(_, _, pattern)
                        | RecordElementBase::InterpolateNamed(_, _, pattern)
                        | RecordElementBase::OmitNamed(_, pattern)
                        | RecordElementBase::Unnamed(pattern)
                        | RecordElementBase::Spread(_, pattern) => {
                            self.emit_failed_pattern(pattern, bind_type);
                        }
                    }
                }
            }
            Array(_, elements, _) => {
                for element in elements {
                    match element.deref() {
                        ArrayElementBase::Element(pattern)
                        | ArrayElementBase::Spread(_, pattern) => {
                            self.emit_failed_pattern(pattern, bind_type)
                        }
                    }
                }
            }
            And(left, _, right) | Or(left, _, right) => {
                self.emit_failed_pattern(left, bind_type);
                self.emit_failed_pattern(right, bind_type);
            }
            Not(_, pattern) => self.emit_failed_pattern(pattern, bind_type),
            Unknown { .. } => (),
        }
    }

    fn emit_literal_constant(&mut self, pattern_constant: &'s Pattern<'s>, value: Register) {
        match pattern_constant {
            Literal(prefix, lit) => {
                if let Some(lit_num) = match &lit.kind {
                    TokenKind::Number(n, _) => Some(*n),
                    TokenKind::Ordinal(o) => Some(*o as f64),
                    TokenKind::Keyword(Keyword::Nan) => Some(f64::NAN),
                    TokenKind::Keyword(Keyword::Inf) => Some(f64::INFINITY),
                    _ => None,
                } {
                    let inv = prefix
                        .as_ref()
                        .is_some_and(|f| *f.as_ref() == Operator::Minus);
                    let lit_num = if inv { -lit_num } else { lit_num };
                    self.op_number(value, lit_num);
                } else {
                    match &lit.kind {
                        TokenKind::Keyword(Keyword::Nil) => {
                            self.op_nil(value);
                        }
                        TokenKind::Keyword(Keyword::True) => {
                            self.op_bool(value, true);
                        }
                        TokenKind::Keyword(Keyword::False) => {
                            self.op_bool(value, false);
                        }
                        TokenKind::String(s, _) => {
                            self.op_string(value, s.as_ref());
                        }
                        _ => self.unreachable(prefix, lit, file!(), line!()),
                    }
                }
            }
            Constant(var) => {
                self.emit_var_read(var, value);
            }
            _ => unreachable!(),
        }
    }

    fn emit_constant_pattern(
        &mut self,
        op: OpCode,
        success: Register,
        pattern: &'s Pattern<'s>,
        value: Register,
    ) -> bool {
        match pattern {
            Literal(_, lit) => {
                if success.is_empty() {
                    self.diagnostics.push(SourceDiagnostic::new(
                        pattern.range(),
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                    ));
                    return true;
                }
                if lit.kind == Keyword::Nil {
                    self.op_3(op, success, Register::EMPTY, value);
                } else {
                    let reg = self.closures.add_reg();
                    self.emit_literal_constant(pattern, reg);
                    self.op_3(op, success, reg, value);
                }
                true
            }
            Constant(_) => {
                if success.is_empty() {
                    self.diagnostics.push(SourceDiagnostic::new(
                        pattern.range(),
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                    ));
                    return true;
                }
                let reg = self.closures.add_reg();
                self.emit_literal_constant(pattern, reg);
                self.op_3(op, success, reg, value);
                true
            }
            Grouping(_, pattern, _) => self.emit_constant_pattern(op, success, pattern, value),
            _ => false,
        }
    }

    pub fn emit_pattern(
        &mut self,
        success: Register,
        pattern: &'s Pattern<'s>,
        value: Register,
        bind_type: Option<BindType>,
    ) {
        if self.emit_constant_pattern(OpCode::Same, success, pattern, value) {
            return;
        }
        match pattern {
            Grouping(_, pattern, _) => self.emit_pattern(success, pattern, value, bind_type),
            Relation(op, constant) => {
                if success.is_empty() {
                    self.diagnostics.push(SourceDiagnostic::new(
                        pattern.range(),
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                    ));
                    return;
                }
                let Some(op) = (match op.kind {
                    TokenKind::Operator(op) if op.is_relation() => op.to_infix_op(),
                    _ => None,
                }) else {
                    self.unreachable(op, constant, file!(), line!());
                    return;
                };
                let reg = self.closures.add_reg();
                self.emit_literal_constant(constant, reg);
                self.op_3(op, success, value, reg);
            }
            Range(l, token, r) => {
                if success.is_empty() {
                    self.diagnostics.push(SourceDiagnostic::new(
                        pattern.range(),
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                    ));
                    return;
                }
                let start = self.closures.add_reg();
                let end = self.closures.add_reg();
                self.emit_literal_constant(l, start);
                self.emit_literal_constant(r, end);
                self.op_3(OpCode::Lte, start, start, value);
                self.op_3(
                    if token.kind == Operator::HalfOpenRange {
                        OpCode::Gt
                    } else {
                        OpCode::Gte
                    },
                    end,
                    end,
                    value,
                );
                self.op_3(OpCode::And, success, start, end);
            }
            Discard(_) => {
                if success.is_empty() {
                    // TODO: useless irrefutable pattern except in array patterns
                    // or use directly in `let _ = <value>`
                    return;
                }
                self.op_bool(success, true);
            }
            SpreadDiscard(_) => {
                if success.is_empty() {
                    return;
                }
                self.op_bool(success, true);
            }
            Bind(_, id_token) => {
                let Some(id) = id_token.to_id_name() else {
                    return;
                };
                if !success.is_empty() {
                    // binding always succeeds
                    self.op_bool(success, true);
                }
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    let bind = bind_type.is_some();
                    if bind {
                        self.closures.initialize_variable(variable);
                    } else {
                        variable.mark_write(id_token);
                        if !check_variable_initialized(
                            self.diagnostics,
                            &self.closures,
                            id_token,
                            variable,
                            level,
                        ) {
                            return;
                        }
                    }
                    if !variable.mutable() && !bind {
                        self.diagnostics.push(SourceDiagnostic::new(
                            id_token.range(),
                            DiagnosticCode::ImmutableVariableAssignment,
                        ));
                        variable.put_decl_ref(self.diagnostics);
                    } else if level == self.closures.len() {
                        let register = variable.register();
                        self.op_unary(register, OpCode::Assign, value);
                    } else {
                        let register = variable.register();
                        let level = self.closures.len() - level;
                        self.op_set_upvalue(value, level, register);
                    }
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        id_token.range.clone(),
                        DiagnosticCode::UndefinedVariableAssignment,
                    ));
                }
            }
            Record(_, elements, _) => {
                let is_empty = elements.is_empty();
                if success.is_empty() {
                    if is_empty {
                        self.diagnostics.push(SourceDiagnostic::new(
                            pattern.range(),
                            DiagnosticCode::UnnecessaryIrrefutablePattern,
                        ));
                        return;
                    }
                } else {
                    self.op_2(OpCode::IsRecord, success, value);
                }

                if !is_empty {
                    // 不论是否成功匹配，都进行子模式匹配
                    // 以便使用 let (:e) = mod; 语法对非记录进行解构

                    let sub_flag = if success.is_empty() {
                        // 此时匹配成功与否并不重要，而且也要把这是非条件匹配的信息传到子级
                        Register::EMPTY
                    } else {
                        self.closures.add_reg()
                    };

                    let has_omitted = elements
                        .last()
                        .is_some_and(|e| matches!(e.deref(), RecordElementBase::Spread(_, _)));
                    let mut omitted = if has_omitted { Some(vec![]) } else { None };

                    fn is_optional(token: &TokenRef<'_>) -> bool {
                        *token.as_ref() == Operator::QuestionColon
                    }
                    for (i, element) in elements.iter().enumerate() {
                        match element.deref() {
                            RecordElementBase::Named(token, colon, pattern) => {
                                let Some((id_type, id)) = token.to_field_name() else {
                                    unreachable!("Expected identifier token");
                                };
                                self.diagnostics
                                    .push(SourceDiagnostic::new(token.range(), id_type));
                                let const_id = self.add_const_string(id);
                                let required = !sub_flag.is_empty() && !is_optional(colon);
                                if required {
                                    self.op_3(OpCode::Has, sub_flag, value, const_id);
                                    self.op_if(OpCode::If, sub_flag);
                                }
                                let ret = self.closures.add_reg();
                                self.op_3(OpCode::Get, ret, value, const_id);
                                self.emit_pattern(sub_flag, pattern, ret, bind_type);
                                if let Some(omitted) = omitted.as_mut() {
                                    omitted.push(const_id);
                                }
                                if required {
                                    self.op_else();
                                    self.emit_failed_pattern(pattern, bind_type);
                                    self.op_if_end();
                                }
                            }
                            RecordElementBase::InterpolateNamed(..) => {}
                            RecordElementBase::OmitNamed(colon, pattern) => {
                                let Pattern::Bind(_, id_token) = pattern.as_ref() else {
                                    continue;
                                };
                                let Some(id) = id_token.to_id_name() else {
                                    continue;
                                };
                                self.diagnostics.push(SourceDiagnostic::new(
                                    colon.range(),
                                    DiagnosticCode::OmitNamedRecordField,
                                ));
                                self.diagnostics.push(SourceDiagnostic::new(
                                    id_token.range(),
                                    DiagnosticCode::OmitNamedRecordFieldName,
                                ));
                                let const_id = self.add_const_string(id);
                                let required = !sub_flag.is_empty() && !is_optional(colon);
                                if required {
                                    self.op_3(OpCode::Has, sub_flag, value, const_id);
                                    self.op_if(OpCode::If, sub_flag);
                                }
                                let ret = self.closures.add_reg();
                                self.op_3(OpCode::Get, ret, value, const_id);
                                self.emit_pattern(sub_flag, pattern, ret, bind_type);
                                if let Some(omitted) = omitted.as_mut() {
                                    omitted.push(const_id);
                                }
                                if required {
                                    self.op_else();
                                    self.emit_failed_pattern(pattern, bind_type);
                                    self.op_if_end();
                                }
                            }
                            RecordElementBase::Unnamed(pattern) => {
                                let ret = self.closures.add_reg();
                                let code = match i {
                                    0 => DiagnosticCode::UnnamedRecordField0,
                                    1 => DiagnosticCode::UnnamedRecordField1,
                                    2 => DiagnosticCode::UnnamedRecordField2,
                                    3 => DiagnosticCode::UnnamedRecordField3,
                                    4 => DiagnosticCode::UnnamedRecordField4,
                                    5 => DiagnosticCode::UnnamedRecordField5,
                                    6 => DiagnosticCode::UnnamedRecordField6,
                                    7 => DiagnosticCode::UnnamedRecordField7,
                                    8 => DiagnosticCode::UnnamedRecordField8,
                                    9 => DiagnosticCode::UnnamedRecordField9,
                                    _ => DiagnosticCode::UnnamedRecordFieldN,
                                };
                                let start = pattern.range().start;
                                self.diagnostics
                                    .push(SourceDiagnostic::new(start..start, code));
                                if !sub_flag.is_empty() {
                                    self.op_3(OpCode::HasIndex, sub_flag, value, OpParam::from(i));
                                    self.op_if(OpCode::If, sub_flag);
                                }
                                self.op_get_index(ret, value, i as i32);
                                self.emit_pattern(sub_flag, pattern, ret, bind_type);
                                if let Some(omitted) = omitted.as_mut() {
                                    let const_id = self.add_const_ordinal(i as i32);
                                    omitted.push(const_id);
                                }
                                if !sub_flag.is_empty() {
                                    self.op_else();
                                    self.emit_failed_pattern(pattern, bind_type);
                                    self.op_if_end();
                                }
                            }
                            RecordElementBase::Spread(_, pattern) => {
                                let ret = self.closures.add_reg();
                                let Some(omitted) = std::mem::take(&mut omitted) else {
                                    continue;
                                };
                                self.op_variadic_1(ret, OpCode::Omit, value, omitted);
                                self.emit_pattern(sub_flag, pattern, ret, bind_type);
                            }
                        }

                        if !sub_flag.is_empty() {
                            self.op_3(OpCode::And, success, success, sub_flag);
                        }
                    }
                }
            }
            Array(_, items, _) => {
                let spread = items
                    .iter()
                    .enumerate()
                    .find(|&(_, p)| matches!(p.deref(), ArrayElementBase::Spread(_, _)));
                let (len, before, spread, after) = if let Some((i, spread)) = spread {
                    let len = items.len() - 1;
                    let before = &items[..i];
                    let spread = spread.deref();
                    let after = &items[i + 1..];
                    (len, before, Some(spread), after)
                } else {
                    (items.len(), &items[..], None, &[] as &[_])
                };

                // 因为数组模式匹配失败后不匹配子模式，需要一个 flag 记录匹配情况
                let flag = if success.is_empty() {
                    self.closures.add_reg()
                } else {
                    success
                };
                self.op_2(OpCode::IsArray, flag, value);

                self.op_if(OpCode::If, flag);

                let sub_flag = if success.is_empty() {
                    // 此时匹配成功与否并不重要，而且也要把这是非条件匹配的信息传到子级
                    Register::EMPTY
                } else {
                    self.closures.add_reg()
                };
                for (i, item) in before.iter().enumerate() {
                    let ArrayElementBase::Element(pattern) = item.deref() else {
                        unreachable!();
                    };

                    let ret = self.closures.add_reg();
                    self.op_get_index(ret, value, i as i32);
                    self.emit_pattern(sub_flag, pattern, ret, bind_type);

                    if !sub_flag.is_empty() {
                        self.op_3(OpCode::And, flag, flag, sub_flag);
                    }
                }
                if let Some(ArrayElementBase::Spread(_, spread)) = spread {
                    for (i, item) in after.iter().rev().enumerate() {
                        let ArrayElementBase::Element(pattern) = item.deref() else {
                            unreachable!();
                        };

                        let ret = self.closures.add_reg();
                        self.op_get_index(ret, value, -1 - (i as i32));
                        self.emit_pattern(sub_flag, pattern, ret, bind_type);

                        if !sub_flag.is_empty() {
                            self.op_3(OpCode::And, flag, flag, sub_flag);
                        }
                    }

                    if !matches!(spread.as_ref(), SpreadDiscard(_)) {
                        let ret = self.closures.add_reg();
                        if before.is_empty() && after.is_empty() {
                            // 如果没有前后元素，直接返回整个数组
                            self.op_2(OpCode::Assign, ret, value);
                        } else if after.is_empty() {
                            // 切片前面的元素
                            self.op_3(OpCode::SliceEnd, ret, value, OpParam::from(before.len()));
                        } else if before.is_empty() {
                            // 切片后面的元素
                            self.op_3(
                                OpCode::SliceStart,
                                ret,
                                value,
                                OpParam::from(-(after.len() as i32) - 1),
                            );
                        } else {
                            // 切片前后都有元素
                            self.op_4(
                                OpCode::Slice,
                                ret,
                                value,
                                OpParam::from(before.len()),
                                OpParam::from(-(after.len() as i32) - 1),
                            );
                        }
                        self.emit_pattern(sub_flag, spread, ret, bind_type);

                        if !sub_flag.is_empty() {
                            self.op_3(OpCode::And, flag, flag, sub_flag);
                        }
                    }
                }

                self.op_else();
                self.emit_failed_pattern(pattern, bind_type);

                self.op_if_end();

                // 最后进行长度测试，避免 [1] 匹配 [x, y] 时 x 也为 nil
                if !success.is_empty() && (len > 0 || spread.is_none()) {
                    self.op_if(OpCode::If, flag);
                    let len_reg = self.closures.add_reg();
                    self.op_2(OpCode::Length, len_reg, value);
                    let expected_len_reg = self.closures.add_reg();
                    self.op_number(expected_len_reg, len as f64);
                    self.op_3(
                        if spread.is_some() {
                            OpCode::Gte
                        } else {
                            OpCode::Eq
                        },
                        flag,
                        len_reg,
                        expected_len_reg,
                    );
                    self.op_if_end();
                }
            }
            And(left, op, right) | Or(left, op, right) => {
                // No short-circuiting in pattern matching
                if success.is_empty() {
                    if pattern.is_or() {
                        self.diagnostics.push(SourceDiagnostic::new(
                            op.range(),
                            DiagnosticCode::MisleadingOrInIrrefutablePattern,
                        ));
                    }
                    self.emit_pattern(Register::EMPTY, left, value, bind_type);
                    self.emit_pattern(Register::EMPTY, right, value, bind_type);
                } else {
                    let op = if *op.as_ref() == Keyword::And {
                        OpCode::And
                    } else {
                        OpCode::Or
                    };
                    let right_success = self.closures.add_reg();
                    self.emit_pattern(success, left, value, bind_type);
                    self.emit_pattern(right_success, right, value, bind_type);
                    self.op_3(op, success, right_success, success);
                }
            }
            Not(kw, p) => {
                if success.is_empty() {
                    self.diagnostics.push(SourceDiagnostic::new(
                        kw.range(),
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                    ));
                    self.emit_pattern(Register::EMPTY, p, value, bind_type);
                } else if self.emit_constant_pattern(OpCode::Nsame, success, p, value) {
                    return;
                } else {
                    self.emit_pattern(success, p, value, bind_type);
                    self.op_2(OpCode::Not, success, success);
                }
            }
            Unknown { .. } => (),
            Literal(..) => unreachable!(),
            Constant(_) => unreachable!(),
        }
    }
}

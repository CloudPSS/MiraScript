use std::ops::Deref;

use crate::{
    Keyword, Operator,
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    emitter::emitter_scope::check_variable_initialized,
    lexer::TokenKind,
    parser::{
        ArrayElementBase, AstWalker,
        Pattern::{self, *},
        RecordElementBase,
    },
};

use super::{Emitter, OpCode, opcode::Register, variable::BindType};

impl<'s> Emitter<'s> {
    pub fn declare_pattern(&mut self, pattern: &'s Pattern<'s>, bind_type: Option<BindType>) {
        match pattern {
            Grouping(op, pattern, cp) => {
                if matches!(
                    pattern.as_ref(),
                    Pattern::Grouping(..)
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
            Constant(_, _) => (),
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
                        ArrayElementBase::Range(_) => unreachable!(),
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
        match pattern {
            Grouping(_, pattern, _) => self.emit_failed_pattern(pattern, bind_type),
            Constant(_, _) => (),
            Range(_, _, _) | Relation(_, _) => (), // It can only include a constant pattern
            Discard(_) | SpreadDiscard(_) => (),
            Bind(_, id_token) => {
                let Some(id) = id_token.to_id_name() else {
                    return;
                };
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    let bind = bind_type.is_some();
                    if bind {
                        self.closures.initialize_variable(variable);
                    } else {
                        variable.mark_write(id_token);
                    }
                    if !check_variable_initialized(
                        &mut self.diagnostics,
                        &self.closures,
                        id_token,
                        variable,
                        level,
                    ) {
                        return;
                    }
                    if !variable.mutable() && !bind {
                        self.diagnostics.push(SourceDiagnostic::new(
                            id_token.range(),
                            DiagnosticCode::ImmutableVariableAssignment,
                        ));
                        variable.put_decl_ref(&mut self.diagnostics);
                    } else if level == self.closures.len() {
                        let register = variable.register();
                        self.op_unary(register, OpCode::Assign, Register::EMPTY);
                    } else {
                        let register = variable.register();
                        let level = self.closures.len() - level;
                        self.op_set_upvalue(Register::EMPTY, level, register);
                    }
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        id_token.range.clone(),
                        DiagnosticCode::UndefinedVariableAssignment,
                    ));
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
                        ArrayElementBase::Range(_) => unreachable!(),
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

    pub fn emit_pattern(
        &mut self,
        success: Register,
        pattern: &'s Pattern<'s>,
        value: Register,
        bind_type: Option<BindType>,
    ) {
        match pattern {
            Grouping(_, pattern, _) => self.emit_pattern(success, pattern, value, bind_type),
            Constant(prefix, lit) => {
                if success.is_empty() {
                    self.diagnostics.push(SourceDiagnostic::new(
                        pattern.range(),
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                    ));
                    return;
                }
                if let Some(lit_num) = match &lit.kind {
                    TokenKind::Number(n) => Some(*n),
                    TokenKind::Ordinal(o) => Some(*o as f64),
                    TokenKind::Keyword(Keyword::Nan) => Some(f64::NAN),
                    TokenKind::Keyword(Keyword::Inf) => Some(f64::INFINITY),
                    _ => None,
                } {
                    let inv = prefix
                        .as_ref()
                        .is_some_and(|f| *f.as_ref() == Operator::Minus);
                    let lit_num = if inv { -lit_num } else { lit_num };
                    self.op_number(success, lit_num);
                    self.op_3(OpCode::Same, success, success, value);
                } else {
                    match &lit.kind {
                        TokenKind::Keyword(Keyword::Nil) => {
                            self.op_3(OpCode::Same, success, Register::EMPTY, value);
                        }
                        TokenKind::Keyword(Keyword::True) => {
                            self.op_bool(success, true);
                            self.op_3(OpCode::Same, success, success, value);
                        }
                        TokenKind::Keyword(Keyword::False) => {
                            self.op_bool(success, false);
                            self.op_3(OpCode::Same, success, success, value);
                        }
                        _ => self.unreachable(prefix, lit, file!(), line!()),
                    }
                }
            }
            Relation(token, pattern) => self.unimplemented(token, pattern),
            Range(l, token, r) => self.unimplemented(l, r),
            Discard(_) => {
                if success.is_empty() {
                    // TODO: useless irrefutable pattern except in array patterns
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
                    }
                    if !check_variable_initialized(
                        &mut self.diagnostics,
                        &self.closures,
                        id_token,
                        variable,
                        level,
                    ) {
                        return;
                    }
                    if !variable.mutable() && !bind {
                        self.diagnostics.push(SourceDiagnostic::new(
                            id_token.range(),
                            DiagnosticCode::ImmutableVariableAssignment,
                        ));
                        variable.put_decl_ref(&mut self.diagnostics);
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
                let flag = if success.is_empty() {
                    if is_empty {
                        self.diagnostics.push(SourceDiagnostic::new(
                            pattern.range(),
                            DiagnosticCode::UnnecessaryIrrefutablePattern,
                        ));
                        return;
                    }
                    self.closures.add_reg()
                } else {
                    success
                };
                self.op_2(OpCode::IsRecord, flag, value);
                self.op_if(OpCode::If, flag);

                for (i, element) in elements.iter().enumerate() {
                    match element.deref() {
                        RecordElementBase::Named(token, _, pattern) => {
                            let Some((id_type, id)) = token.to_field_name() else {
                                unreachable!("Expected identifier token");
                            };
                            self.diagnostics
                                .push(SourceDiagnostic::new(token.range(), id_type));
                            let ret = self.closures.add_reg();
                            self.op_get(ret, value, id);
                            self.emit_pattern(success, pattern, ret, bind_type);
                        }
                        RecordElementBase::InterpolateNamed(..) => {}
                        RecordElementBase::OmitNamed(colon, pattern) => {
                            let Pattern::Bind(_, id_token) = pattern.as_ref() else {
                                continue;
                            };
                            let TokenKind::Identifier(id) = &id_token.kind else {
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
                            let ret = self.closures.add_reg();
                            self.op_get(ret, value, id.as_ref());
                            self.emit_pattern(success, pattern, ret, bind_type);
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
                            self.diagnostics.push(SourceDiagnostic::new(
                                SourceRange { start, end: start },
                                code,
                            ));
                            self.op_get_index(ret, value, i);
                            self.emit_pattern(success, pattern, ret, bind_type);
                        }
                        RecordElementBase::Spread(dots, pattern) => {
                            // let ret = self.closures.add_reg();
                            // self.op_get_spread(ret, value);
                            // self.emit_pattern(success, pattern, ret, bind_type);
                            self.unimplemented(dots, pattern)
                        }
                    }
                }

                if !is_empty {
                    self.op_else();
                    self.emit_failed_pattern(pattern, bind_type);
                }

                self.op_if_end();
            }
            Array(ob, array_element_bases, cb) => self.unimplemented(ob, cb),
            And(left, op, right) | Or(left, op, right) => {
                // No short-circuiting in pattern matching
                if success.is_empty() {
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
            Not(_, p) => {
                if success.is_empty() {
                    self.emit_pattern(Register::EMPTY, p, value, bind_type);
                } else {
                    self.emit_pattern(success, p, value, bind_type);
                    self.op_2(OpCode::Not, success, success);
                }
            }
            Unknown { .. } => (),
        }
    }
}

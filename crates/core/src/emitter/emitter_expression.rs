use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    emitter::{emitter_scope::check_variable_initialized, variable::VariableUsage},
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        self, ArrayElement, AstWalker, Callable,
        Expression::{self, *},
        Iterable, Range, RecordElement,
    },
};

use super::{
    Emitter, OpCode,
    opcode::{OpParam, OpParamTrait, Register},
    variable::BindType,
};

fn is_global_expression(expr: &Expression) -> bool {
    matches!(expr, Variable(token) if token.kind == Keyword::Global)
}

impl<'s> Emitter<'s> {
    pub fn declare_expression(&mut self, expr: &'s Expression<'s>) {
        match expr {
            Literal(_) => (),
            InterpolatedString(_, expressions) => expressions
                .iter()
                .for_each(|expression| self.declare_expression(expression)),
            Variable(_) => (),
            Grouping(_, expression, _) => self.declare_expression(expression),
            Record(_, elements, _) => elements.iter().for_each(|element| match element {
                RecordElement::Named(_, _, exp, _)
                | RecordElement::OmitNamed(_, exp, _)
                | RecordElement::Unnamed(exp, _)
                | RecordElement::Spread(_, exp, _) => {
                    self.declare_expression(exp);
                }
                RecordElement::InterpolateNamed(name_exp, _, exp, _) => {
                    self.declare_expression(name_exp);
                    self.declare_expression(exp);
                }
            }),
            Array(_, elements, _) => elements.iter().for_each(|element| match element {
                ArrayElement::Spread(_, exp, _) | ArrayElement::Element(exp, _) => {
                    self.declare_expression(exp);
                }
                ArrayElement::Range(range, _) => {
                    self.declare_expression(&range.0);
                    self.declare_expression(&range.2);
                }
            }),
            Call(callable, _, expressions, _) => {
                if let Callable::Expression(callable) = callable.as_ref() {
                    self.declare_expression(callable);
                }
                expressions
                    .iter()
                    .for_each(|expression| self.declare_expression(expression));
            }
            Extension(expression, _, callable, _, expressions, _) => {
                self.declare_expression(expression);
                if let Callable::Expression(callable) = callable.as_ref() {
                    self.declare_expression(callable);
                }
                expressions
                    .iter()
                    .for_each(|expression| self.declare_expression(expression));
            }
            Access(expression, _, _) => self.declare_expression(expression),
            Index(expression, _, prop, _) => {
                self.declare_expression(expression);
                self.declare_expression(prop);
            }
            NonNil(expression, _) => self.declare_expression(expression),
            Prefix(_, expression) => self.declare_expression(expression),
            Infix(left, _, right) => {
                self.declare_expression(left);
                self.declare_expression(right);
            }
            Is(expression, _, pattern) => {
                self.declare_expression(expression);
                self.declare_pattern(pattern, Some(BindType::Init));
            }
            Block(_, _, _, _) => (),
            Loop(_, _) => (),
            While(_, _, _, _) => (),
            ForIn(_, _, _, _, _, _) => (),
            If(_, _, _, _) => (),
            Match(_, _, _, _, _) => (),
            Function(_, _, _) => (),
            Unknown { .. } => (),
        }
    }
    pub fn emit_expression_reg(
        &mut self,
        expr: &'s Expression<'s>,
        brk: Option<Register>,
    ) -> Register {
        let reg = self.add_reg();
        self.emit_expression(expr, reg, brk);
        reg
    }
    pub fn emit_expression(
        &mut self,
        expr: &'s Expression<'s>,
        ret: Register,
        brk: Option<Register>,
    ) {
        match expr {
            Literal(token) => match &token.kind {
                TokenKind::String(s) => self.op_string(ret, s.as_ref()),
                TokenKind::Number(n) => self.op_number(ret, *n),
                TokenKind::Ordinal(o) => self.op_number(ret, *o as f64),
                TokenKind::Keyword(Keyword::Nil) => self.op_nil(ret),
                TokenKind::Keyword(Keyword::True) => self.op_bool(ret, true),
                TokenKind::Keyword(Keyword::False) => self.op_bool(ret, false),
                TokenKind::Keyword(Keyword::Nan) => self.op_number(ret, f64::NAN),
                TokenKind::Keyword(Keyword::Inf) => self.op_number(ret, f64::INFINITY),
                _ => self.unreachable(token, token, file!(), line!()),
            },
            InterpolatedString(token, expressions) => {
                let TokenKind::InterpolatedString(strs, _) = &token.kind else {
                    self.unreachable(token, expressions, file!(), line!());
                    return;
                };
                let mut args_reg = vec![];
                let mut s_iter = strs.iter();
                let mut e_iter = expressions.iter();
                loop {
                    let Some(str) = s_iter.next() else {
                        break;
                    };
                    if !str.is_empty() {
                        let reg = self.add_reg();
                        self.op_string(reg, str.as_ref());
                        args_reg.push(reg);
                    }
                    if let Some(expression) = e_iter.next() {
                        let reg = self.add_reg();
                        self.emit_expression(expression, reg, brk);
                        args_reg.push(reg);
                    }
                }
                if args_reg.is_empty() {
                    self.op_string(ret, "");
                } else if args_reg.len() == 1 {
                    self.op_unary(ret, OpCode::ToString, args_reg[0]);
                } else {
                    self.op_variadic(ret, OpCode::Concat, args_reg);
                }
            }
            Variable(token) => {
                let TokenKind::Identifier(id) = &token.kind else {
                    if token.kind == Keyword::Global {
                        self.diagnostics.push(SourceDiagnostic::new(
                            token.range.clone(),
                            DiagnosticCode::MisuseOfGlobalKeyword,
                        ));
                    }
                    return;
                };
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    let register = variable.register();
                    variable.usage(token, VariableUsage::Read, &mut self.diagnostics);
                    if !check_variable_initialized(
                        &mut self.diagnostics,
                        &self.closures,
                        token,
                        variable,
                        level,
                    ) {
                        return;
                    }
                    if level == self.closures.len() {
                        self.op_unary(ret, OpCode::Assign, register);
                    } else {
                        let up_reg = variable.register();
                        let level = self.closures.len() - level;
                        self.op_get_upvalue(ret, level, up_reg);
                    }
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        token.range(),
                        DiagnosticCode::GlobalVariable,
                    ));
                    self.op_global(ret, id.as_ref());
                }
            }
            Grouping(_, expression, _) => self.emit_expression(expression, ret, brk),
            Record(_, elements, _) => {
                let mut elements_regs = vec![];
                for element in elements {
                    match element {
                        RecordElement::Named(_, _, expression, _) => {
                            let reg = self.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                        RecordElement::InterpolateNamed(name_expression, _, expression, _) => {
                            let name_reg = self.add_reg();
                            self.emit_expression(name_expression, name_reg, brk);
                            let reg = self.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(name_reg);
                            elements_regs.push(reg);
                        }
                        RecordElement::OmitNamed(_, expression, _) => {
                            let reg = self.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                        RecordElement::Unnamed(expression, _) => {
                            let reg = self.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                        RecordElement::Spread(_, expression, _) => {
                            let reg = self.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                    }
                }
                self.op_1(OpCode::Record, ret);
                let mut reg_index = 0;
                for element in elements.iter() {
                    let opt = element
                        .colon()
                        .is_some_and(|c| c.kind == Operator::QuestionColon);
                    match element {
                        RecordElement::Named(token, _, _, _) => {
                            let reg = elements_regs[reg_index];
                            if let TokenKind::Ordinal(id) = &token.kind {
                                self.diagnostics.push(SourceDiagnostic::new(
                                    token.range(),
                                    DiagnosticCode::RecordFieldOrdinalName,
                                ));
                                self.op_2(
                                    if opt {
                                        OpCode::FieldOptIndex
                                    } else {
                                        OpCode::FieldIndex
                                    },
                                    OpParam::new(*id as usize),
                                    reg,
                                );
                            } else {
                                let Some((id_type, id)) = token.to_field_name() else {
                                    self.unreachable(token, token, file!(), line!());
                                    return;
                                };
                                self.diagnostics
                                    .push(SourceDiagnostic::new(token.range(), id_type));
                                let const_id = self.add_const_string(id);
                                self.op_2(
                                    if opt { OpCode::FieldOpt } else { OpCode::Field },
                                    const_id,
                                    reg,
                                );
                            }
                            reg_index += 1;
                        }
                        RecordElement::InterpolateNamed(_, _, _, _) => {
                            let name_reg = elements_regs[reg_index];
                            let reg = elements_regs[reg_index + 1];
                            self.op_2(
                                if opt {
                                    OpCode::FieldOptDyn
                                } else {
                                    OpCode::FieldDyn
                                },
                                name_reg,
                                reg,
                            );
                            reg_index += 2;
                        }
                        RecordElement::OmitNamed(colon, expression, _) => {
                            let id_token = if let Expression::Variable(id)
                            | Expression::Access(_, _, id) = expression.as_ref()
                            {
                                Some(id)
                            } else if let Expression::Index(_, _, id, _) = expression.as_ref() {
                                if let Expression::Literal(literal) = id.as_ref() {
                                    Some(literal)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            let Some((_, id)) = id_token.and_then(|id| id.to_field_name()) else {
                                self.diagnostics.push(SourceDiagnostic::new(
                                    expression.range(),
                                    DiagnosticCode::BadOmitKeyRecordExpression,
                                ));
                                return;
                            };
                            self.diagnostics.push(SourceDiagnostic::new(
                                colon.range(),
                                DiagnosticCode::OmitNamedRecordField,
                            ));
                            self.diagnostics.push(SourceDiagnostic::new(
                                id_token.unwrap().range(),
                                DiagnosticCode::OmitNamedRecordFieldName,
                            ));
                            let const_id = self.add_const_string(id);
                            let reg = elements_regs[reg_index];
                            self.op_2(
                                if opt { OpCode::FieldOpt } else { OpCode::Field },
                                const_id,
                                reg,
                            );
                            reg_index += 1;
                        }
                        RecordElement::Unnamed(exp, _) => {
                            let reg = elements_regs[reg_index];
                            self.op_2(
                                if opt {
                                    OpCode::FieldOptIndex
                                } else {
                                    OpCode::FieldIndex
                                },
                                OpParam::new(reg_index),
                                reg,
                            );
                            let code = match reg_index {
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
                            let start = exp.range().start;
                            self.diagnostics.push(SourceDiagnostic::new(
                                SourceRange { start, end: start },
                                code,
                            ));
                            reg_index += 1;
                        }
                        RecordElement::Spread(_, _, _) => {
                            let reg = elements_regs[reg_index];
                            self.op_1(OpCode::Spread, reg);
                            reg_index += 1;
                        }
                    }
                }
                self.op(OpCode::Freeze);
            }
            Array(_, items, _) => {
                let mut items_regs = vec![];
                for item in items {
                    match item {
                        ArrayElement::Element(expression, _) => {
                            let reg = self.emit_expression_reg(expression, brk);
                            items_regs.push(reg);
                        }
                        ArrayElement::Spread(_, expression, _) => {
                            let reg = self.emit_expression_reg(expression, brk);
                            items_regs.push(reg);
                        }
                        ArrayElement::Range(range, _) => {
                            let Range(start, _, end) = range.as_ref();
                            let start = self.emit_expression_reg(start, brk);
                            let end = self.emit_expression_reg(end, brk);
                            items_regs.push(start);
                            items_regs.push(end);
                        }
                    }
                }
                self.op_1(OpCode::Array, ret);
                let mut reg_index = 0;
                for item in items.iter() {
                    match item {
                        ArrayElement::Element(_, _) => {
                            self.op_1(OpCode::Item, items_regs[reg_index]);
                            reg_index += 1;
                        }
                        ArrayElement::Spread(_, _, _) => {
                            self.op_1(OpCode::Spread, items_regs[reg_index]);
                            reg_index += 1;
                        }
                        ArrayElement::Range(range, _) => {
                            let start = items_regs[reg_index];
                            let end = items_regs[reg_index + 1];
                            self.op_2(
                                if range.exclusive() {
                                    OpCode::ItemRangeExclusiveDyn
                                } else {
                                    OpCode::ItemRangeDyn
                                },
                                start,
                                end,
                            );
                            reg_index += 2;
                        }
                    }
                }
                self.op(OpCode::Freeze);
            }
            Call(callable, _, args, _) => {
                let mut args_reg = vec![];
                for expression in args {
                    let reg = self.add_reg();
                    self.emit_expression(expression, reg, brk);
                    args_reg.push(reg);
                }
                match callable.as_ref() {
                    Callable::Expression(expression) => {
                        let reg = self.add_reg();
                        self.emit_expression(expression, reg, brk);
                        self.op_call_dyn(ret, reg, args_reg);
                    }
                    Callable::Type(_) => {
                        self.op_unary(ret, OpCode::Type, args_reg.into_iter().next().unwrap());
                    }
                }
            }
            Extension(expression, _, callable, _, args, _) => {
                let self_reg = self.add_reg();
                self.emit_expression(expression, self_reg, brk);
                let mut args_reg = vec![];
                for expression in args {
                    let reg = self.add_reg();
                    self.emit_expression(expression, reg, brk);
                    args_reg.push(reg);
                }
                match callable.as_ref() {
                    Callable::Expression(expression) => {
                        let reg = self.add_reg();
                        args_reg.insert(0, self_reg);
                        self.emit_expression(expression, reg, brk);
                        self.op_call_dyn(ret, reg, args_reg);
                    }
                    Callable::Type(_) => {
                        self.op_unary(ret, OpCode::Type, self_reg);
                    }
                }
            }
            Access(expression, _, id) => {
                if !is_global_expression(expression) {
                    self.emit_expression(expression, ret, brk);
                    match &id.kind {
                        TokenKind::Identifier(id) => self.op_get(ret, ret, id.as_ref()),
                        TokenKind::Ordinal(ord) => self.op_get_num(ret, ret, *ord as f64),
                        _ => unreachable!("Expected identifier token"),
                    };
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        id.range(),
                        DiagnosticCode::GlobalVariable,
                    ));
                    match &id.kind {
                        TokenKind::Identifier(id) => self.op_global(ret, id.as_ref()),
                        TokenKind::Ordinal(ord) => self.op_global_num(ret, *ord as f64),
                        _ => unreachable!("Expected identifier token"),
                    };
                }
            }
            Index(expression, _, index, _) => {
                if !is_global_expression(expression) {
                    self.emit_expression(expression, ret, brk);
                    let index_reg = self.add_reg();
                    self.emit_expression(index, index_reg, brk);
                    self.op_get_dyn(ret, ret, index_reg);
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        index.range(),
                        DiagnosticCode::GlobalDynamicAccess,
                    ));
                    let index_reg = self.add_reg();
                    self.emit_expression(index, index_reg, brk);
                    self.op_global_dyn(ret, index_reg);
                }
            }
            NonNil(expression, _) => {
                self.emit_expression(expression, ret, brk);
                self.op_non_nil(ret);
            }
            Prefix(token, expression) => {
                let reg = self.add_reg();
                self.emit_expression(expression, reg, brk);
                let op = match token.kind {
                    TokenKind::Operator(Operator::Plus) => OpCode::Pos,
                    TokenKind::Operator(Operator::Minus) => OpCode::Neg,
                    TokenKind::Operator(Operator::Exclamation) => OpCode::Not,
                    _ => unreachable!(),
                };
                self.op_unary(ret, op, reg);
            }
            Infix(left, token, right) => {
                if **token == Operator::LogicalAnd {
                    // ret is used in the immediate if, so we need to ensure it is not empty
                    let ret = if !ret.is_empty() { ret } else { self.add_reg() };
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::If, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Operator::LogicalOr {
                    let ret = if !ret.is_empty() { ret } else { self.add_reg() };
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::IfNot, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Operator::NullCoalescing {
                    let ret = if !ret.is_empty() { ret } else { self.add_reg() };
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::IfNil, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Keyword::In && is_global_expression(right) {
                    let left_reg = self.add_reg();
                    self.emit_expression(left, left_reg, brk);
                    self.op_unary(ret, OpCode::InGlobal, left_reg);
                } else {
                    let op = match token.kind {
                        TokenKind::Operator(Operator::Caret) => OpCode::Pow,

                        TokenKind::Operator(Operator::Asterisk) => OpCode::Mul,
                        TokenKind::Operator(Operator::Slash) => OpCode::Div,
                        TokenKind::Operator(Operator::Percent) => OpCode::Mod,

                        TokenKind::Operator(Operator::Plus) => OpCode::Add,
                        TokenKind::Operator(Operator::Minus) => OpCode::Sub,

                        TokenKind::Operator(Operator::Greater) => OpCode::Gt,
                        TokenKind::Operator(Operator::GreaterEqual) => OpCode::Geq,
                        TokenKind::Operator(Operator::Less) => OpCode::Lt,
                        TokenKind::Operator(Operator::LessEqual) => OpCode::Leq,

                        TokenKind::Operator(Operator::EqualEqual) => OpCode::Eq,
                        TokenKind::Operator(Operator::NotEqual) => OpCode::Neq,
                        TokenKind::Operator(Operator::TildeEqual) => OpCode::Aeq,
                        TokenKind::Operator(Operator::NotTildeEqual) => OpCode::Naeq,
                        TokenKind::Keyword(Keyword::In) => OpCode::In,

                        _ => unreachable!("Unexpected infix operator"),
                    };
                    let left_reg = self.add_reg();
                    let right_reg = self.add_reg();
                    self.emit_expression(left, left_reg, brk);
                    self.emit_expression(right, right_reg, brk);
                    self.op_binary(ret, op, left_reg, right_reg);
                }
            }
            Is(expression, token, pattern) => self.unimplemented(expression, pattern),
            Block(_, stmts, ret_expr, _) => {
                self.enter_scope(expr.range());
                self.declare_block(stmts, ret_expr);
                self.emit_block(stmts, ret_expr, ret, brk);
                self.exit_scope();
            }
            Loop(_, expression) => {
                self.enter_closure(false);
                self.enter_scope(expression.range());
                let pos = self.chunk.code.len();
                self.op(OpCode::Loop);
                let Expression::Block(_, stmts, expr, _) = expression.as_ref() else {
                    // unreachable!("Expected block expression");
                    return;
                };
                self.emit_block(stmts, expr, Register::EMPTY, Some(ret));
                self.op(OpCode::LoopEnd);

                let nreg: OpParam = self.current_closure().reg_len().into();
                if nreg.is_wide() {
                    self.chunk.code[pos] = OpCode::Loop.wide_code();
                    self.chunk.code.splice(pos + 1..pos + 1, nreg.wide_code());
                } else {
                    self.chunk.code.splice(pos + 1..pos + 1, [nreg.code()]);
                }

                self.exit_scope();
                self.exit_closure();
            }
            While(kw, cond, body, else_part) => {
                let Expression::Block(_, stmts, expr, _) = body.as_ref() else {
                    // unreachable!("Expected block expression");
                    return;
                };

                let ret = if !ret.is_empty() {
                    self.op_uninit(ret);
                    ret
                } else if else_part.is_some() {
                    self.add_reg()
                } else {
                    Register::EMPTY
                };

                self.enter_closure(false);
                self.enter_scope(kw.range.end..body.range().end);

                let cond_reg = self.add_reg();
                self.declare_expression(cond);
                self.declare_block(stmts, expr);

                let pos = self.chunk.code.len();
                self.op(OpCode::Loop);
                self.emit_expression(cond, cond_reg, Some(ret));
                self.op_if(OpCode::IfNot, cond_reg);
                self.op(OpCode::Break);
                self.op_if_end();

                self.emit_block(stmts, expr, Register::EMPTY, Some(ret));
                self.op(OpCode::LoopEnd);

                let nreg: OpParam = self.current_closure().reg_len().into();
                if nreg.is_wide() {
                    self.chunk.code[pos] = OpCode::Loop.wide_code();
                    self.chunk.code.splice(pos + 1..pos + 1, nreg.wide_code());
                } else {
                    self.chunk.code.splice(pos + 1..pos + 1, [nreg.code()]);
                }

                self.exit_scope();
                self.exit_closure();

                if !ret.is_empty() {
                    self.op_if(OpCode::IfNotInit, ret);
                    if let Some((_, else_expr)) = else_part {
                        self.emit_expression(else_expr, ret, None);
                    } else {
                        self.op_nil(ret);
                    }
                    self.op_if_end();
                }
            }
            ForIn(kw, pattern, _, iterable, expression, else_part) => {
                let Expression::Block(_, stmts, expr, _) = expression.as_ref() else {
                    // unreachable!("Expected block expression");
                    return;
                };

                // 先进入 scope 再进入 closure
                self.enter_leveled_scope(
                    kw.range.end..expression.range().end,
                    self.closures.len() + 1,
                );

                self.declare_pattern(pattern, Some(BindType::Init));
                match iterable.as_ref() {
                    Iterable::Value(value) => {
                        self.declare_expression(value);
                    }
                    Iterable::Range(range) => {
                        self.declare_expression(&range.0);
                        self.declare_expression(&range.2);
                    }
                };

                let iterable_reg = match iterable.as_ref() {
                    Iterable::Value(value) => self.emit_expression_reg(value, None),
                    Iterable::Range(range) => {
                        let start = self.emit_expression_reg(&range.0, None);
                        let end = self.emit_expression_reg(&range.2, None);
                        let ret = self.add_reg();
                        self.op_1(OpCode::Array, ret);
                        self.op_2(
                            if range.exclusive() {
                                OpCode::ItemRangeExclusiveDyn
                            } else {
                                OpCode::ItemRangeDyn
                            },
                            start,
                            end,
                        );
                        self.op(OpCode::Freeze);
                        ret
                    }
                };

                self.enter_closure(false);
                // 根据虚拟机定义，iterator 寄存器为闭包内第一个寄存器
                // 进入 closure 后立即分配
                let iterator = self.add_reg();
                self.declare_block(stmts, expr);

                let ret = if !ret.is_empty() {
                    self.op_uninit(ret);
                    ret
                } else if else_part.is_some() {
                    self.add_reg()
                } else {
                    Register::EMPTY
                };

                let pos = self.chunk.code.len();
                self.op(OpCode::LoopFor);
                self.emit_pattern(pattern, iterator, Some(BindType::Init));
                self.emit_block(stmts, expr, Register::EMPTY, Some(ret));
                self.op(OpCode::LoopEnd);
                let nreg: OpParam = self.current_closure().reg_len().into();
                if nreg.is_wide() || iterable_reg.is_wide() {
                    self.chunk.code[pos] = OpCode::Loop.wide_code();
                    self.chunk.code.splice(
                        pos + 1..pos + 1,
                        [nreg.wide_code(), iterable_reg.wide_code()].concat(),
                    );
                } else {
                    self.chunk
                        .code
                        .splice(pos + 1..pos + 1, [nreg.code(), iterable_reg.code()]);
                }
                self.exit_scope();
                self.exit_closure();

                if !ret.is_empty() {
                    self.op_if(OpCode::IfNotInit, ret);
                    if let Some((_, else_expr)) = else_part {
                        self.emit_expression(else_expr, ret, None);
                    } else {
                        self.op_nil(ret);
                    }
                    self.op_if_end();
                }
            }
            If(kw, cond, then_expr, else_part) => {
                self.enter_scope(kw.range.end..expr.range().end);
                self.declare_expression(cond);

                let cond_reg = self.add_reg();
                self.emit_expression(cond, cond_reg, brk);
                self.op_if(OpCode::If, cond_reg);
                self.emit_expression(then_expr, ret, brk);
                if let Some((_, else_expr)) = else_part {
                    self.op_else();
                    self.emit_expression(else_expr, ret, brk);
                } else if !ret.is_empty() {
                    self.op_else();
                    self.op_nil(ret);
                }
                self.op_if_end();

                self.exit_scope();
            }
            Match(token, expression, op, items, cp) => self.unimplemented(token, cp),
            Function(kw, args, expression) => {
                let parser::Expression::Block(_, stmts, expr, _) = &**expression else {
                    // unreachable!("Expected block expression");
                    return;
                };
                let body_range = expr.range();
                self.emit_fn(
                    ret,
                    kw.range.end..body_range.start,
                    args,
                    body_range,
                    stmts,
                    expr,
                );
            }
            Unknown { .. } => {
                // Load nil as result of unknown expression
                self.op_nil(ret);
            }
        }
    }
}

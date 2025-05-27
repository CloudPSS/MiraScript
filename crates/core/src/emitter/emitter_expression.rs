use crate::{
    error::{ErrorCode, SourceError},
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        self, ArrayElement, AstWalker, Callable,
        Expression::{self, *},
        Iterable, Range, RecordElement, Statement,
    },
};

use super::{
    Emitter, OpCode,
    opcode::{OpParam, Register},
    variable::{BindType, Variable},
};

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
                RecordElement::CalculatedNamed(_, name_exp, _, _, exp, _) => {
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
    pub fn emit_expression_reg(&mut self, expr: &'s Expression<'s>, brk: Register) -> Register {
        let reg = self.add_reg();
        self.emit_expression(expr, reg, brk);
        reg
    }
    pub fn emit_expression(&mut self, expr: &'s Expression<'s>, ret: Register, brk: Register) {
        match expr {
            Literal(token) => match &token.kind {
                TokenKind::String(s) => self.op_string(ret, s),
                TokenKind::Number(n) => self.op_number(ret, *n),
                TokenKind::Ordinal(o) => self.op_number(ret, *o as f64),
                TokenKind::Keyword(Keyword::Nil) => self.op_nil(ret),
                TokenKind::Keyword(Keyword::True) => self.op_bool(ret, true),
                TokenKind::Keyword(Keyword::False) => self.op_bool(ret, false),
                TokenKind::Keyword(Keyword::Nan) => self.op_number(ret, f64::NAN),
                TokenKind::Keyword(Keyword::Inf) => self.op_number(ret, f64::INFINITY),
                _ => unreachable!(),
            },
            InterpolatedString(token, expressions) => {
                let TokenKind::InterpolatedString(strs, _) = &token.kind else {
                    unreachable!();
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
                        self.op_string(reg, str);
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
                    unreachable!("Expected identifier token");
                };
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    if level == self.closures.len() {
                        if !variable.initialized() {
                            self.errors.push(SourceError::new(
                                token.range.clone(),
                                ErrorCode::UninitializedVariable,
                            ));
                        }
                        let register = variable.register();
                        self.op_unary(ret, OpCode::Assign, register);
                    } else {
                        let up_reg = variable.register();
                        let level = self.closures.len() - level;
                        self.op_get_upvalue(ret, level, up_reg);
                    }
                } else {
                    self.op_global(ret, id);
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
                        RecordElement::CalculatedNamed(_, name_expression, _, _, expression, _) => {
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
                    match element {
                        RecordElement::Named(token, _, _, _) => {
                            let TokenKind::Identifier(id) = &token.kind else {
                                unreachable!("Expected identifier token");
                            };
                            let const_id = self.add_const_string(id);
                            let reg = elements_regs[reg_index];
                            self.op_2(OpCode::Field, const_id, reg);
                            reg_index += 1;
                        }
                        RecordElement::CalculatedNamed(_, _, _, _, _, _) => {
                            let name_reg = elements_regs[reg_index];
                            let reg = elements_regs[reg_index + 1];
                            self.op_2(OpCode::FieldDyn, name_reg, reg);
                            reg_index += 2;
                        }
                        RecordElement::OmitNamed(_, expression, _) => {
                            let Expression::Variable(id) = expression.as_ref() else {
                                unreachable!("Expected identifier token");
                            };
                            let TokenKind::Identifier(id) = &id.kind else {
                                unreachable!("Expected identifier token");
                            };
                            let const_id = self.add_const_string(id);
                            let reg = elements_regs[reg_index];
                            self.op_2(OpCode::Field, const_id, reg);
                            reg_index += 1;
                        }
                        RecordElement::Unnamed(_, _) => {
                            let reg = elements_regs[reg_index];
                            self.op_2(OpCode::FieldIndex, OpParam::new(reg_index), reg);
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
                self.emit_expression(expression, ret, brk);
                match &id.kind {
                    TokenKind::Identifier(id) => self.op_get(ret, ret, id),
                    TokenKind::Ordinal(ord) => self.op_get_num(ret, ret, *ord as f64),
                    _ => unreachable!("Expected identifier token"),
                };
            }
            Index(expression, _, index, _) => {
                self.emit_expression(expression, ret, brk);
                let index_reg = self.add_reg();
                self.emit_expression(index, index_reg, brk);
                self.op_get_dyn(ret, ret, index_reg);
            }
            NonNil(expression, token) => {
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
                    _ => todo!(),
                };
                self.op_unary(ret, op, reg);
            }
            Infix(left, token, right) => {
                if **token == Operator::LogicalAnd {
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::If, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Operator::LogicalOr {
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::IfNot, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
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

                        _ => unreachable!("Unexpected infix operator"),
                    };
                    let left_reg = self.add_reg();
                    let right_reg = self.add_reg();
                    self.emit_expression(left, left_reg, brk);
                    self.emit_expression(right, right_reg, brk);
                    self.op_binary(ret, op, left_reg, right_reg);
                }
            }
            Is(expression, token, pattern) => todo!(),
            Block(_, stmts, expr, _) => {
                self.enter_scope();
                self.emit_block(stmts, expr, ret, brk);
                self.exit_scope();
            }
            Loop(_, expression) => {
                self.enter_scope();
                self.op(OpCode::Loop);
                let Expression::Block(_, stmts, expr, _) = expression.as_ref() else {
                    unreachable!("Expected block expression");
                };
                self.emit_block(stmts, expr, Register::EMPTY, ret);
                self.op(OpCode::LoopEnd);
                self.exit_scope();
            }
            While(_, cond, body, else_part) => {
                self.enter_scope();
                let cond_reg = self.add_reg();
                self.declare_expression(cond);
                self.op_uninit(ret);
                self.op(OpCode::Loop);
                self.emit_expression(cond, cond_reg, ret);
                self.op_if(OpCode::IfNot, cond_reg);
                self.op(OpCode::Break);
                self.op_if_end();

                let Expression::Block(_, stmts, expr, _) = body.as_ref() else {
                    unreachable!("Expected block expression");
                };
                self.emit_block(stmts, expr, Register::EMPTY, ret);

                self.op(OpCode::LoopEnd);
                self.exit_scope();

                self.op_if(OpCode::IfNotInit, ret);
                if let Some((_, else_expr)) = else_part {
                    let Expression::Block(_, stmts, expr, _) = else_expr.as_ref() else {
                        unreachable!("Expected block expression");
                    };
                    self.emit_block(stmts, expr, ret, Register::EMPTY);
                } else {
                    self.op_nil(ret);
                }
                self.op_if_end();
            }
            ForIn(_, pattern, _, iterable, expression, else_part) => {
                self.enter_scope();
                self.declare_pattern(pattern, Some(BindType::Init));
                let iterable_reg = match iterable.as_ref() {
                    Iterable::Value(expr) => self.emit_expression_reg(expr, Register::EMPTY),
                    Iterable::Range(range) => {
                        let start = self.emit_expression_reg(range.0.as_ref(), Register::EMPTY);
                        let end = self.emit_expression_reg(range.2.as_ref(), Register::EMPTY);
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
                let iterator = self.add_reg();
                self.op_uninit(ret);
                self.op_2(OpCode::LoopFor, iterator, iterable_reg);
                self.emit_pattern(pattern, iterator, Some(BindType::Init));

                let Expression::Block(_, stmts, expr, _) = expression.as_ref() else {
                    unreachable!("Expected block expression");
                };
                self.emit_block(stmts, expr, Register::EMPTY, ret);
                self.op(OpCode::LoopEnd);
                self.exit_scope();

                self.op_if(OpCode::IfNotInit, ret);
                if let Some((_, else_expr)) = else_part {
                    let Expression::Block(_, stmts, expr, _) = else_expr.as_ref() else {
                        unreachable!("Expected block expression");
                    };
                    self.emit_block(stmts, expr, ret, Register::EMPTY);
                } else {
                    self.op_nil(ret);
                }
                self.op_if_end();
            }
            If(_, cond, then_expr, else_part) => {
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
            }
            Match(token, expression, token1, items, token2) => todo!(),
            Function(_, args, expression) => {
                let parser::Expression::Block(_, stmts, expr, _) = &**expression else {
                    unreachable!("Expected block expression");
                };
                self.emit_closure(ret, args, stmts, expr);
            }
            Unknown { .. } => {
                // Load nil as result of unknown expression
                self.op_nil(ret);
            }
        }
    }
}

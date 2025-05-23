use crate::{
    error::{ErrorCode, SourceError},
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        ArrayElement, Callable,
        Expression::{self, *},
        RecordElement, Statement,
    },
};

use super::{
    Emitter, OpCode,
    opcode::Register,
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
                RecordElement::Named(_, _, exp, _) | RecordElement::Spread(_, exp, _) => {
                    self.declare_expression(exp);
                }
                _ => (),
            }),
            Array(_, elements, _) => elements.iter().for_each(|element| match element {
                ArrayElement::Spread(_, exp, _) | ArrayElement::Element(exp, _) => {
                    self.declare_expression(exp);
                }
                _ => (),
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
    pub fn emit_expression(&mut self, expr: &'s Expression<'s>, ret: Register) {
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
                        self.emit_expression(expression, reg);
                        args_reg.push(reg);
                    }
                }
                self.op_variadic(ret, OpCode::Concat, args_reg);
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
            Grouping(_, expression, _) => self.emit_expression(expression, ret),
            Record(token, record_element_bases, token1) => todo!(),
            Array(token, array_element_bases, token1) => todo!(),
            Call(callable, _, expressions, _) => {
                let mut args_reg = vec![];
                for expression in expressions {
                    let reg = self.add_reg();
                    self.emit_expression(expression, reg);
                    args_reg.push(reg);
                }
                match callable.as_ref() {
                    Callable::Expression(expression) => {
                        let reg = self.add_reg();
                        self.emit_expression(expression, reg);
                        self.op_call_dyn(ret, reg, args_reg);
                    }
                    Callable::Type(_) => {
                        self.op_unary(ret, OpCode::Type, args_reg.into_iter().next().unwrap());
                    }
                }
            }
            Extension(expression, token, callable, token1, expressions, token2) => todo!(),
            Access(expression, token, token1) => todo!(),
            Index(expression, token, expression1, token1) => todo!(),
            NonNil(expression, token) => todo!(),
            Prefix(token, expression) => {
                let reg = self.add_reg();
                self.emit_expression(expression, reg);
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
                    self.emit_expression(left, ret);
                    self.op_if(ret);
                    self.emit_expression(right, ret);
                    self.op_if_end();
                } else if **token == Operator::LogicalOr {
                    self.emit_expression(left, ret);
                    self.op_if(ret);
                    self.op_else();
                    self.emit_expression(right, ret);
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

                        TokenKind::Operator(Operator::Equal) => OpCode::Eq,
                        TokenKind::Operator(Operator::NotEqual) => OpCode::Neq,
                        TokenKind::Operator(Operator::TildeEqual) => OpCode::Aeq,
                        TokenKind::Operator(Operator::NotTildeEqual) => OpCode::Naeq,

                        _ => unreachable!("Unexpected infix operator"),
                    };
                    let left_reg = self.add_reg();
                    let right_reg = self.add_reg();
                    self.emit_expression(left, left_reg);
                    self.emit_expression(right, right_reg);
                    self.op_binary(ret, op, left_reg, right_reg);
                }
            }
            Is(expression, token, pattern) => todo!(),
            Block(_, stmts, expr, _) => {
                self.enter_scope();
                self.emit_block(ret, stmts, expr);
                self.exit_scope();
            }
            Loop(token, expression) => todo!(),
            While(token, expression, expression1, _) => todo!(),
            ForIn(token, pattern, token1, iterable, expression, _) => todo!(),
            If(token, expression, expression1, _) => todo!(),
            Match(token, expression, token1, items, token2) => todo!(),
            Function(token, tokens, expression) => todo!(),
            Unknown { .. } => {
                // Load nil as result of unknown expression
                self.op_nil(ret);
            }
        }
    }
}

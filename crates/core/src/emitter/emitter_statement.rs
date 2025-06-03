use crate::{
    error::{ErrorCode, SourceError, SourceRange},
    lexer::{Operator, TokenKind},
    parser::{
        self, Expression,
        Statement::{self, *},
    },
};

use super::{Emitter, OpCode, opcode::Register, variable::BindType};

impl<'s> Emitter<'s> {
    pub fn declare_statement(&mut self, stmt: &'s Statement<'s>) {
        match stmt {
            Expression(_, _) | BlockExpression(_) => (),
            Bind(_, pattern, _, _, _) => {
                self.declare_pattern(pattern, Some(BindType::Let));
            }
            Rebind(pattern, _, _, _) => {
                self.declare_pattern(pattern, None);
            }
            Assign(_, _, _, _) => (),
            Function(_, name, _, _) => {
                self.declare_variable(name, false, BindType::Func);
            }
            Return(_, _, _) => (),
            Break(_, _, _) => (),
            Continue(_, _) => (),
            Empty(_) | Unknown { .. } => (),
        }
    }
    pub fn emit_statement(&mut self, stmt: &'s Statement<'s>, brk: Option<Register>) -> bool {
        match stmt {
            Expression(expression, _) | BlockExpression(expression) => {
                self.enter_scope();
                self.declare_expression(expression);
                self.emit_expression(expression, Register::EMPTY, brk);
                self.exit_scope();
                false
            }
            Bind(_, pattern, _, expression, _) => {
                let value_reg = self.add_reg();
                self.enter_scope();
                self.declare_expression(expression);
                self.emit_expression(expression, value_reg, brk);
                self.exit_scope();
                self.emit_pattern(pattern, value_reg, Some(BindType::Let));
                false
            }
            Rebind(pattern, _, expression, _) => {
                let value_reg = self.add_reg();
                self.enter_scope();
                self.declare_expression(expression);
                self.emit_expression(expression, value_reg, brk);
                self.exit_scope();
                self.emit_pattern(pattern, value_reg, None);
                false
            }
            Assign(assignee, op, expression, _) => {
                let mut final_op: Option<Box<dyn FnOnce(&mut Self)>> = None;
                let assignee_reg = match &**assignee {
                    Expression::Variable(id_token) => {
                        let TokenKind::Identifier(id) = &id_token.kind else {
                            unreachable!();
                        };
                        let var = self.scopes.find_variable(id);
                        if let Some((level, variable)) = var {
                            if !variable.mutable() {
                                self.errors.push(SourceError::new(
                                    id_token.range.clone(),
                                    ErrorCode::ImmutableVariableAssignment,
                                ));
                                Register::EMPTY
                            } else if level == self.closures.len() {
                                variable.register()
                            } else {
                                let up_reg = variable.register();
                                let level = self.closures.len() - level;
                                let ret = self.add_reg();
                                self.op_get_upvalue(ret, level, up_reg);
                                final_op = Some(Box::new(move |s| {
                                    s.op_set_upvalue(ret, level, up_reg);
                                }));

                                ret
                            }
                        } else {
                            self.errors.push(SourceError::new(
                                id_token.range.clone(),
                                ErrorCode::UndefinedVariableAssignment,
                            ));
                            Register::EMPTY
                        }
                    }
                    Expression::Access(obj, _, prop) => {
                        todo!()
                    }
                    Expression::Index(obj, _, prop_expr, _) => {
                        todo!()
                    }
                    _ => unreachable!(),
                };
                if **op == Operator::Equal {
                    self.enter_scope();
                    self.declare_expression(expression);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.exit_scope();
                } else if **op == Operator::LogicalAndEqual {
                    self.op_if(OpCode::If, assignee_reg);
                    self.enter_scope();
                    self.declare_expression(expression);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.exit_scope();
                    self.op_if_end();
                } else if **op == Operator::LogicalOrEqual {
                    self.op_if(OpCode::IfNot, assignee_reg);
                    self.enter_scope();
                    self.declare_expression(expression);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.exit_scope();
                    self.op_if_end();
                } else if **op == Operator::NullCoalescingEqual {
                    self.op_if(OpCode::IfNil, assignee_reg);
                    self.enter_scope();
                    self.declare_expression(expression);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.exit_scope();
                    self.op_if_end();
                } else {
                    let op = match op.kind {
                        TokenKind::Operator(Operator::PlusEqual) => OpCode::Add,
                        TokenKind::Operator(Operator::MinusEqual) => OpCode::Sub,
                        TokenKind::Operator(Operator::AsteriskEqual) => OpCode::Mul,
                        TokenKind::Operator(Operator::SlashEqual) => OpCode::Div,
                        TokenKind::Operator(Operator::PercentEqual) => OpCode::Mod,
                        TokenKind::Operator(Operator::CaretEqual) => OpCode::Pow,
                        _ => unreachable!("Unexpected assign operator"),
                    };
                    let right_reg = self.add_reg();
                    self.enter_scope();
                    self.declare_expression(expression);
                    self.emit_expression(expression, right_reg, brk);
                    self.exit_scope();
                    self.op_binary(assignee_reg, op, assignee_reg, right_reg);
                }
                if let Some(final_op) = final_op {
                    final_op(self);
                }
                false
            }
            Function(_, name, args, expression) => {
                let TokenKind::Identifier(name) = &name.kind else {
                    unreachable!("Expected identifier token");
                };
                let func_var = self.scopes.find_local_variable(name).unwrap();
                func_var.initialize();
                let func_reg = func_var.register();
                let parser::Expression::Block(_, stmts, expr, _) = &**expression else {
                    unreachable!("Expected block expression");
                };
                self.emit_fn(func_reg, args, stmts, expr);
                false
            }
            Return(_, expression, _) => {
                if let Some(expression) = expression {
                    let ret_reg = self.add_reg();
                    self.enter_scope();
                    self.declare_expression(expression);
                    self.emit_expression(expression, ret_reg, brk);
                    self.exit_scope();
                    self.op_return(ret_reg);
                } else {
                    self.op_return(Register::EMPTY);
                }
                true
            }
            Break(kw, expression, comma) => {
                let Some(brk) = brk else {
                    self.errors.push(SourceError::new(
                        SourceRange {
                            start: kw.range.start,
                            end: comma.range.end,
                        },
                        ErrorCode::UnexpectedBreakOutsideLoop,
                    ));
                    return false;
                };
                if let Some(expression) = expression {
                    self.enter_scope();
                    self.declare_expression(expression);
                    let brk_ret = self.add_reg();
                    self.emit_expression(expression, brk_ret, Some(brk));
                    self.op_set_upvalue(brk_ret, 1, brk);
                    self.exit_scope();
                } else if !brk.is_empty() {
                    self.op_set_upvalue(Register::EMPTY, 1, brk);
                }
                self.op(OpCode::Break);
                true
            }
            Continue(kw, comma) => {
                if brk.is_none() {
                    self.errors.push(SourceError::new(
                        SourceRange {
                            start: kw.range.start,
                            end: comma.range.end,
                        },
                        ErrorCode::UnexpectedContinueOutsideLoop,
                    ));
                    return false;
                }
                self.op(OpCode::Continue);
                true
            }
            Empty(_) | Unknown { .. } => false,
        }
    }
}

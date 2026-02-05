use crate::{
    diagnostic::DiagnosticCode,
    emitter::{emitter_scope::check_variable_initialized, opcode::OpParam},
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        AstWalker, Expression, Pattern,
        Statement::{self, *},
    },
};

use super::{
    Emitter, OpCode,
    emitter_pub::{ModuleExports, ModuleExportsData},
    opcode::Register,
    utils::is_global_expression,
    variable::BindType,
};

impl<'s, 'c> Emitter<'s, 'c> {
    pub fn declare_statement(
        &mut self,
        stmt: &'s Statement<'s>,
        exports: &mut ModuleExports<'s, 'c>,
    ) {
        match stmt {
            Expression(expr, _) | BlockExpression(expr) => self.declare_expression(expr),
            Module(kw_pub, _, id, _) => {
                let Some(var) = self.declare_variable(id, false, BindType::Module) else {
                    return;
                };
                let name = var.name();
                self.declare_pub(exports, kw_pub, name);
            }
            Bind(kw_pub, _, pattern, _, expr, _) => {
                if matches!(pattern.as_ref(), Pattern::Bind(None, _))
                    && matches!(expr.as_ref(), Expression::Function(..))
                {
                    self.declare_pattern(pattern, Some(BindType::LetFunc), kw_pub, exports);
                } else {
                    self.declare_pattern(pattern, Some(BindType::Let), kw_pub, exports);
                }
                self.declare_expression(expr);
            }
            Rebind(pattern, _, expr, _) => {
                self.declare_pattern(pattern, None, &None, &mut None);
                self.declare_expression(expr);
            }
            Const(kw_pub, _, id_token, _, expr, _) => {
                let Some(var) = self.declare_variable(id_token, false, BindType::Const) else {
                    return;
                };
                let name = var.name();
                self.declare_pub(exports, kw_pub, name);
                self.declare_expression(expr);
            }
            Assign(assignee, _, expr, _) => {
                self.declare_expression(assignee);
                self.declare_expression(expr);
            }
            Function(kw_pub, _, name, _, _) => {
                let Some(var) = self.declare_variable(name, false, BindType::Func) else {
                    return;
                };
                let name = var.name();
                self.declare_pub(exports, kw_pub, name);
            }
            Return(_, expr, _) | Break(_, expr, _) => {
                if let Some(expr) = expr {
                    self.declare_expression(expr);
                }
            }
            Continue(_, _) => (),
            Empty(_) | Unknown { .. } => (),
        }
    }

    pub fn emit_statement(&mut self, stmt: &'s Statement<'s>, brk: Option<Register>) -> bool {
        match stmt {
            Expression(expression, _) | BlockExpression(expression) => {
                self.emit_expression(expression, Register::EMPTY, brk);
                false
            }
            Module(_, _, id_token, expr) => {
                let Some(id) = id_token.to_id_name() else {
                    return false;
                };
                let Some((_, variable)) = self.scopes.find_variable(id) else {
                    return false;
                };
                let Expression::Block(_, stmts, None, _) = expr.as_ref() else {
                    return false;
                };
                let reg = self.closures.initialize_variable(variable);

                self.enter_scope(expr.range());

                let mut mod_exports = ModuleExportsData::new(id_token);
                self.declare_block(stmts, &None, &mut mod_exports);
                mod_exports.unwrap().commit(self, id, reg);

                self.emit_block(stmts, &None, expr.range(), Register::EMPTY, brk);
                self.exit_scope();

                false
            }
            Bind(_, _, pattern, _, expression, _) => {
                let value_reg = self.emit_expression_reg(expression, brk);
                self.emit_pattern(Register::EMPTY, pattern, value_reg, Some(BindType::Let));
                false
            }
            Rebind(pattern, _, expression, _) => {
                let value_reg = self.emit_expression_reg(expression, brk);
                self.emit_pattern(Register::EMPTY, pattern, value_reg, None);
                false
            }
            Const(_, _, id_token, _, expression, _) => {
                let Some(id) = id_token.to_id_name() else {
                    return false;
                };
                let Some((_, variable)) = self.scopes.find_variable(id) else {
                    return false;
                };
                self.closures.initialize_variable(variable);
                let reg = variable.register();
                self.emit_expression(expression, reg, brk);
                false
            }
            Assign(assignee, op, expression, _) => {
                let op = op.as_ref();
                let is_compound = *op != Operator::Assign;
                let mut final_op: Box<dyn FnOnce(&mut Self)> = Box::new(|_| ());
                let assignee_reg = match &**assignee {
                    Expression::Variable(id_token) => {
                        if **id_token == Keyword::Global {
                            self.diagnostics
                                .push(DiagnosticCode::MisuseOfGlobalKeyword, id_token.range());
                            return false;
                        }
                        let Some(id) = id_token.to_id_name() else {
                            return false;
                        };
                        let var = self.scopes.find_variable(id);
                        if let Some((level, variable)) = var {
                            if *op == Operator::Assign {
                                variable.mark_write(id_token);
                            } else {
                                variable.mark_read_write(id_token);
                            }
                            check_variable_initialized(
                                &mut self.diagnostics,
                                &self.closures,
                                id_token,
                                variable,
                                level,
                            );
                            if !variable.mutable() {
                                self.diagnostics.push(
                                    DiagnosticCode::ImmutableVariableAssignment,
                                    id_token.range(),
                                );
                                variable.put_decl_ref(&mut self.diagnostics);
                                Register::EMPTY
                            } else if level == self.closures.len() {
                                variable.register()
                            } else {
                                let up_reg = variable.register();
                                let level = self.closures.len() - level;
                                let ret = self.closures.add_reg();
                                self.op_get_upvalue(stmt.range(), ret, level, up_reg);
                                final_op = Box::new(move |s| {
                                    s.op_set_upvalue(stmt.range(), ret, level, up_reg);
                                });

                                ret
                            }
                        } else {
                            self.diagnostics.push(
                                DiagnosticCode::UndefinedVariableAssignment,
                                id_token.range.clone(),
                            );
                            Register::EMPTY
                        }
                    }

                    Expression::Access(obj, _, id) if !is_global_expression(obj) => {
                        let obj_reg = self.emit_expression_reg(obj, brk);
                        let field_reg = self.closures.add_reg();
                        match id.kind {
                            TokenKind::Identifier(id) => {
                                let field_name = self.add_const_string(id);
                                if is_compound {
                                    self.op_3(
                                        stmt.range(),
                                        OpCode::Get,
                                        field_reg,
                                        obj_reg,
                                        field_name,
                                    );
                                }
                                final_op = Box::new(move |s| {
                                    s.op_3(
                                        stmt.range(),
                                        OpCode::Set,
                                        field_reg,
                                        obj_reg,
                                        field_name,
                                    );
                                });
                            }
                            TokenKind::Ordinal(ord) => {
                                let field_ord = OpParam::new(ord);
                                if is_compound {
                                    self.op_3(
                                        stmt.range(),
                                        OpCode::GetIndex,
                                        field_reg,
                                        obj_reg,
                                        field_ord,
                                    );
                                }
                                final_op = Box::new(move |s| {
                                    s.op_3(
                                        stmt.range(),
                                        OpCode::SetIndex,
                                        field_reg,
                                        obj_reg,
                                        field_ord,
                                    );
                                });
                            }
                            _ => unreachable!("Expected identifier token"),
                        }
                        field_reg
                    }
                    Expression::Index(obj, _, prop_expr, _) if !is_global_expression(obj) => {
                        let obj_reg = self.emit_expression_reg(obj, brk);
                        let field_name = self.emit_expression_reg(prop_expr, brk);
                        let field_reg = self.closures.add_reg();
                        if is_compound {
                            self.op_3(stmt.range(), OpCode::GetDyn, field_reg, obj_reg, field_name);
                        }
                        final_op = Box::new(move |s| {
                            s.op_3(stmt.range(), OpCode::SetDyn, field_reg, obj_reg, field_name);
                        });
                        field_reg
                    }
                    _ => {
                        self.diagnostics
                            .push(DiagnosticCode::UnassignableExpression, assignee.range());
                        return false;
                    }
                };
                if !is_compound {
                    self.emit_expression(expression, assignee_reg, brk);
                } else if *op == Operator::LogicalAndAssign {
                    self.op_if(stmt.range(), OpCode::If, assignee_reg);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.op_if_end(stmt.range());
                } else if *op == Operator::LogicalOrAssign {
                    self.op_if(stmt.range(), OpCode::IfNot, assignee_reg);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.op_if_end(stmt.range());
                } else if *op == Operator::NullCoalescingAssign {
                    self.op_if(stmt.range(), OpCode::IfNil, assignee_reg);
                    self.emit_expression(expression, assignee_reg, brk);
                    self.op_if_end(stmt.range());
                } else {
                    let Some(op) = (match op.kind {
                        TokenKind::Operator(o) => o.to_compound_op(),
                        _ => None,
                    }) else {
                        // Unexpected assign operator
                        self.unreachable(op, op, file!(), line!());
                        return false;
                    };
                    let right_reg = self.emit_expression_reg(expression, brk);
                    self.op_binary(stmt.range(), assignee_reg, op, assignee_reg, right_reg);
                }
                final_op(self);
                false
            }
            Function(_, _, name_token, args, body) => {
                let TokenKind::Identifier(name) = name_token.kind else {
                    unreachable!("Expected identifier token");
                };
                let func_var = self.scopes.find_local_variable(name).unwrap();
                let func_reg = self.closures.initialize_variable(func_var);
                self.emit_fn(
                    func_reg,
                    name_token.range(),
                    name_token.range.end,
                    args,
                    body,
                );
                false
            }
            Return(_, expression, _) => {
                if let Some(expression) = expression {
                    let ret_reg = self.emit_expression_reg(expression, brk);
                    self.op_return(stmt.range(), ret_reg);
                } else {
                    self.op_return(stmt.range(), Register::EMPTY);
                }
                true
            }
            Break(kw, expression, comma) => {
                let Some(brk) = brk else {
                    self.diagnostics.push(
                        DiagnosticCode::UnexpectedBreak,
                        kw.range.start..comma.range.end,
                    );
                    return false;
                };
                if let Some(expression) = expression {
                    let brk_ret = self.emit_expression_reg(expression, Some(brk));
                    self.op_set_upvalue(stmt.range(), brk_ret, 1, brk);
                } else if !brk.is_empty() {
                    self.op_set_upvalue(stmt.range(), Register::EMPTY, 1, brk);
                }
                self.op(kw.range(), OpCode::Break);
                true
            }
            Continue(kw, comma) => {
                if brk.is_none() {
                    self.diagnostics.push(
                        DiagnosticCode::UnexpectedContinue,
                        kw.range.start..comma.range.end,
                    );
                    return false;
                }
                self.op(kw.range(), OpCode::Continue);
                true
            }
            Empty(_) | Unknown { .. } => false,
        }
    }
}

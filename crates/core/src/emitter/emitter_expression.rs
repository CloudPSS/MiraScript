use std::{borrow::Cow, ops::Deref};

use crate::{
    SourceRange,
    diagnostic::{DiagnosticCode, SourceDiagnostic},
    emitter::{emitter_scope::check_variable_initialized, utils::is_global_expression},
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        ArrayElement, ArrayElementBase, AstWalker, Callable, ElseBlock,
        Expression::{self, *},
        Iterable, Range, RecordElementBase,
    },
};

use super::{
    Emitter, OpCode,
    opcode::{OpParam, OpParamTrait, Register},
    variable::BindType,
};

impl<'s> Emitter<'s> {
    fn declare_callable(&mut self, callable: &'s Callable<'s>) {
        match callable {
            Callable::Expression(callable) => {
                // 此时的 Grouping 用于标记 callable 为复杂表达式以启用空安全，跳过 declare_expression 的 Grouping 处理
                if let Expression::Grouping(_, inner, _) = callable.as_ref() {
                    if inner.is_variable() || inner.is_grouping() {
                        self.declare_expression(inner);
                        return;
                    }
                }
                self.declare_expression(callable);
            }
            Callable::Type(_) => (),
        }
    }
    fn declare_argument(&mut self, arg: &'s ArrayElement<'s>) {
        match arg.deref() {
            ArrayElementBase::Element(expression) => self.declare_expression(expression),
            ArrayElementBase::Spread(_, expression) => self.declare_expression(expression),
            ArrayElementBase::Range(_) => unreachable!(),
        }
    }
    pub fn declare_expression(&mut self, outer: &'s Expression<'s>) {
        match outer {
            Literal(_) => (),
            // InterpolatedString 内的表达式只有 Variable 和 Block，均不需要声明
            InterpolatedString(_, _) => (),
            Variable(_) => (),
            Grouping(op, expression, cp) => {
                if matches!(
                    expression.as_ref(),
                    Expression::Variable(..)
                        | Expression::Literal(..)
                        | Expression::Grouping(..)
                        | Expression::InterpolatedString(..)
                        | Expression::Record(..)
                        | Expression::Array(..)
                        | Expression::NonNil(..)
                        | Expression::Call(..)
                        | Expression::Extension(..)
                        | Expression::Access(..)
                        | Expression::Index(..)
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
                self.declare_expression(expression)
            }
            Record(_, elements, _) => elements.iter().for_each(|element| match element.deref() {
                RecordElementBase::Named(_, _, exp)
                | RecordElementBase::OmitNamed(_, exp)
                | RecordElementBase::Unnamed(exp)
                | RecordElementBase::Spread(_, exp) => {
                    self.declare_expression(exp);
                }
                RecordElementBase::InterpolateNamed(name_exp, _, exp) => {
                    self.declare_expression(name_exp);
                    self.declare_expression(exp);
                }
            }),
            Array(_, elements, _) => elements.iter().for_each(|element| match element.deref() {
                ArrayElementBase::Spread(_, exp) | ArrayElementBase::Element(exp) => {
                    self.declare_expression(exp);
                }
                ArrayElementBase::Range(range) => {
                    self.declare_expression(&range.0);
                    self.declare_expression(&range.2);
                }
            }),
            Call(callable, _, expressions, _) => {
                self.declare_callable(callable);
                expressions
                    .iter()
                    .for_each(|arg| self.declare_argument(arg));
            }
            Extension(expression, _, callable, _, expressions, _) => {
                self.declare_expression(expression);
                self.declare_callable(callable);
                expressions
                    .iter()
                    .for_each(|arg| self.declare_argument(arg));
            }
            Access(expression, _, _) => self.declare_expression(expression),
            Index(expression, _, prop, _) => {
                self.declare_expression(expression);
                self.declare_expression(prop);
            }
            Slice(expression, _, start, _, end, _) => {
                self.declare_expression(expression);
                if let Some(start) = start {
                    self.declare_expression(start);
                }
                if let Some(end) = end {
                    self.declare_expression(end);
                }
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
            Loop(kw_loop, body) => {
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_loop.range.start..body.range().end,
                    DiagnosticCode::LoopExpression,
                ));
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_loop.range(),
                    DiagnosticCode::KeywordLoop,
                ));
            }
            While(kw_while, _, body, else_part) => {
                let else_part = else_part.as_ref();
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_while.range.start
                        ..else_part
                            .map(|e| e.1.range())
                            .unwrap_or_else(|| body.range())
                            .end,
                    DiagnosticCode::WhileExpression,
                ));
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_while.range(),
                    DiagnosticCode::KeywordWhile,
                ));
                else_part.iter().for_each(|ElseBlock(kw_else, _)| {
                    self.diagnostics.push(SourceDiagnostic::new(
                        kw_else.range(),
                        DiagnosticCode::KeywordElse,
                    ));
                });
            }
            ForIn(kw_for, _, kw_in, _, body, else_part) => {
                let (kw_else, else_body) = if let Some(ElseBlock(kw_else, else_body)) = else_part {
                    (Some(kw_else), Some(else_body))
                } else {
                    (None, None)
                };
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_for.range.start..else_body.unwrap_or(body).range().end,
                    DiagnosticCode::ForExpression,
                ));
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_for.range(),
                    DiagnosticCode::KeywordFor,
                ));
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_in.range(),
                    DiagnosticCode::KeywordIn,
                ));
                kw_else.iter().for_each(|kw_else| {
                    self.diagnostics.push(SourceDiagnostic::new(
                        kw_else.range(),
                        DiagnosticCode::KeywordElse,
                    ));
                });
            }
            If(kw_if, _, then_block, else_part) => {
                let mut else_part = else_part.as_ref();
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_if.range.start
                        ..else_part
                            .map(|e| e.1.range())
                            .unwrap_or_else(|| then_block.range())
                            .end,
                    DiagnosticCode::IfExpression,
                ));
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_if.range(),
                    DiagnosticCode::KeywordIf,
                ));

                while let Some(ElseBlock(kw_else, else_block)) = else_part {
                    self.diagnostics.push(SourceDiagnostic::new(
                        kw_else.range(),
                        DiagnosticCode::KeywordElse,
                    ));
                    if let If(kw_if, _, _, next_else_part) = else_block.as_ref() {
                        self.diagnostics.push(SourceDiagnostic::new(
                            kw_if.range(),
                            DiagnosticCode::KeywordIf,
                        ));
                        else_part = next_else_part.as_ref();
                    } else {
                        else_part = None;
                    }
                }
            }
            Match(kw_match, _, _, branches, cp) => {
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_match.range.start..cp.range.end,
                    DiagnosticCode::MatchExpression,
                ));
                self.diagnostics.push(SourceDiagnostic::new(
                    kw_match.range(),
                    DiagnosticCode::KeywordMatch,
                ));
                self.diagnostics
                    .extend(branches.iter().map(|(kw_case, _, _)| {
                        SourceDiagnostic::new(kw_case.range(), DiagnosticCode::KeywordCase)
                    }));
            }
            Function(_, _, _) => (),
            Unknown { .. } => (),
        }
    }
    pub fn emit_expression_reg(
        &mut self,
        expr: &'s Expression<'s>,
        brk: Option<Register>,
    ) -> Register {
        let reg = self.closures.add_reg();
        self.emit_expression(expr, reg, brk);
        reg
    }
    fn hint_mislead_nil(&mut self, id: &str, range: SourceRange) {
        if id == "null"
            || id == "Null"
            || id == "NULL"
            || id == "undefined"
            || id == "None"
            || id == "Nothing"
        {
            self.diagnostics.push(SourceDiagnostic::new(
                range,
                DiagnosticCode::MisleadingNilVariable,
            ));
        }
    }
    fn emit_global_access(&mut self, expr: &'s Expression<'s>) -> Option<Cow<'s, str>> {
        let id = if let Variable(id_token) = expr {
            let id = id_token.to_id_name()?;
            if self.scopes.find_variable(id).is_some() {
                // 变量已在当前作用域中声明
                return None;
            }
            self.diagnostics.push(SourceDiagnostic::new(
                id_token.range(),
                DiagnosticCode::GlobalVariable,
            ));
            self.hint_mislead_nil(id, id_token.range());
            Cow::Borrowed(id)
        } else if let Access(parent, _, id_token) = expr {
            let Variable(parent) = parent.as_ref() else {
                return None;
            };
            if parent.kind != Keyword::Global {
                return None;
            }
            let (_, id) = id_token.to_field_name()?;
            self.diagnostics.push(SourceDiagnostic::new(
                id_token.range(),
                DiagnosticCode::GlobalVariable,
            ));
            id
        } else {
            // 不处理 global[expr]
            return None;
        };
        Some(id)
    }
    fn emit_call(
        &mut self,
        callable: &'s Callable<'s>,
        arg0: Option<&'s Expression<'s>>,
        args: &'s [ArrayElement<'s>],
        ret: Register,
        brk: Option<Register>,
    ) {
        match callable {
            Callable::Expression(callable) => {
                let args_reg = |s: &mut Self| {
                    let mut args_reg = vec![];
                    let mut spread: Vec<OpParam> = vec![];
                    if let Some(arg0) = arg0 {
                        args_reg.push(s.emit_expression_reg(arg0, brk));
                    }
                    for arg in args {
                        let reg = s.closures.add_reg();
                        match arg.deref() {
                            ArrayElementBase::Element(exp) => s.emit_expression(exp, reg, brk),
                            ArrayElementBase::Spread(_, exp) => {
                                spread.push(args_reg.len().into());
                                s.emit_expression(exp, reg, brk);
                            }
                            ArrayElementBase::Range(_) => unreachable!(),
                        }
                        args_reg.push(reg);
                    }
                    (args_reg, spread)
                };

                if let Some(id) = self.emit_global_access(callable) {
                    // Global function call
                    let (args, spreads) = args_reg(self);
                    self.op_call(ret, id, args, spreads);
                    return;
                }

                // Local function call
                let callable_reg = self.emit_expression_reg(callable, brk);
                let complex = !callable.is_variable();
                if complex {
                    self.op_if(OpCode::IfNotNil, callable_reg);
                }
                let (args, spreads) = args_reg(self);
                self.op_call_dyn(ret, callable_reg, args, spreads);
                if complex {
                    self.op_else();
                    self.op_nil(ret);
                    self.op_if_end();
                }
            }
            Callable::Type(_) => {
                let arg0 = arg0.map_or(Register::EMPTY, |f| self.emit_expression_reg(f, brk));
                self.op_unary(ret, OpCode::Type, arg0);
            }
        }
    }
    pub fn emit_expression(
        &mut self,
        expr: &'s Expression<'s>,
        ret: Register,
        brk: Option<Register>,
    ) {
        match expr {
            Literal(token) => match &token.kind {
                TokenKind::String(s, _) => self.op_string(ret, s.as_ref()),
                TokenKind::Number(n, _) => self.op_number(ret, *n),
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
                    self.unreachable(*token, expressions, file!(), line!());
                    return;
                };
                let mut args_reg = vec![];
                let mut s_iter = strs.iter();
                let mut e_iter = expressions.iter();
                loop {
                    let Some((str, _)) = s_iter.next() else {
                        break;
                    };
                    if !str.is_empty() {
                        let reg = self.closures.add_reg();
                        self.op_string(reg, str.as_ref());
                        args_reg.push(reg);
                    }
                    if let Some(expression) = e_iter.next() {
                        let reg = self.closures.add_reg();
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
                let Some(id) = token.to_id_name() else {
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
                    variable.mark_read(token);
                    if !check_variable_initialized(
                        self.diagnostics,
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
                    self.op_global(ret, id);
                    self.hint_mislead_nil(id, token.range());
                }
            }
            Grouping(_, expression, _) => self.emit_expression(expression, ret, brk),
            Record(_, elements, _) => {
                let mut elements_regs = vec![];
                for element in elements {
                    match element.deref() {
                        RecordElementBase::Named(_, _, expression) => {
                            let reg = self.closures.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                        RecordElementBase::InterpolateNamed(name_expression, _, expression) => {
                            let name_reg = self.closures.add_reg();
                            self.emit_expression(name_expression, name_reg, brk);
                            let reg = self.closures.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(name_reg);
                            elements_regs.push(reg);
                        }
                        RecordElementBase::OmitNamed(_, expression) => {
                            let reg = self.closures.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                        RecordElementBase::Unnamed(expression) => {
                            let reg = self.closures.add_reg();
                            self.emit_expression(expression, reg, brk);
                            elements_regs.push(reg);
                        }
                        RecordElementBase::Spread(_, expression) => {
                            let reg = self.closures.add_reg();
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
                    match element.deref() {
                        RecordElementBase::Named(token, _, _) => {
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
                                    OpParam::new(*id),
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
                        RecordElementBase::InterpolateNamed(_, _, _) => {
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
                        RecordElementBase::OmitNamed(colon, expression) => {
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
                        RecordElementBase::Unnamed(exp) => {
                            let reg = elements_regs[reg_index];
                            self.op_2(
                                if opt {
                                    OpCode::FieldOptIndex
                                } else {
                                    OpCode::FieldIndex
                                },
                                OpParam::from(reg_index),
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
                            self.diagnostics
                                .push(SourceDiagnostic::new(start..start, code));
                            reg_index += 1;
                        }
                        RecordElementBase::Spread(_, _) => {
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
                    match item.deref() {
                        ArrayElementBase::Element(expression) => {
                            let reg = self.emit_expression_reg(expression, brk);
                            items_regs.push(reg);
                        }
                        ArrayElementBase::Spread(_, expression) => {
                            let reg = self.emit_expression_reg(expression, brk);
                            items_regs.push(reg);
                        }
                        ArrayElementBase::Range(range) => {
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
                    match item.deref() {
                        ArrayElementBase::Element(_) => {
                            self.op_1(OpCode::Item, items_regs[reg_index]);
                            reg_index += 1;
                        }
                        ArrayElementBase::Spread(_, _) => {
                            self.op_1(OpCode::Spread, items_regs[reg_index]);
                            reg_index += 1;
                        }
                        ArrayElementBase::Range(range) => {
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
                if let Some((first, rest)) = args.split_first() {
                    if let ArrayElementBase::Element(first) = first.deref() {
                        self.emit_call(callable, Some(first), rest, ret, brk);
                        return;
                    }
                }
                self.emit_call(callable, None, args, ret, brk);
            }
            Extension(expression, _, callable, _, args, _) => {
                self.emit_call(callable, Some(expression), args, ret, brk);
            }
            Access(expression, _, id) => {
                if !is_global_expression(expression) {
                    self.emit_expression(expression, ret, brk);
                    match id.kind {
                        TokenKind::Identifier(id) => self.op_get(ret, ret, id),
                        TokenKind::Ordinal(ord) => self.op_get_index(ret, ret, ord),
                        _ => unreachable!("Expected identifier token"),
                    };
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        id.range(),
                        DiagnosticCode::GlobalVariable,
                    ));
                    match id.kind {
                        TokenKind::Identifier(id) => self.op_global(ret, id),
                        TokenKind::Ordinal(ord) => self.op_global_num(ret, ord as f64),
                        _ => unreachable!("Expected identifier token"),
                    };
                }
            }
            Index(expression, _, index, _) => {
                if !is_global_expression(expression) {
                    self.emit_expression(expression, ret, brk);
                    let index_reg = self.closures.add_reg();
                    self.emit_expression(index, index_reg, brk);
                    self.op_get_dyn(ret, ret, index_reg);
                } else {
                    self.diagnostics.push(SourceDiagnostic::new(
                        index.range(),
                        DiagnosticCode::GlobalDynamicAccess,
                    ));
                    let index_reg = self.closures.add_reg();
                    self.emit_expression(index, index_reg, brk);
                    self.op_global_dyn(ret, index_reg);
                }
            }
            Slice(expression, _, start, op, end, _) => {
                // slice 不能用于 global 关键字，Variable 表达式将处理此错误
                let arr_reg = if ret.is_empty() {
                    self.closures.add_reg()
                } else {
                    ret
                };
                self.emit_expression(expression, arr_reg, brk);
                let start_reg = if let Some(start) = start {
                    self.emit_expression_reg(start, brk)
                } else {
                    Register::EMPTY
                };
                let end_reg = if let Some(end) = end {
                    self.emit_expression_reg(end, brk)
                } else {
                    Register::EMPTY
                };
                let op = if *op.as_ref() == Operator::HalfOpenRange {
                    OpCode::SliceExclusiveDyn
                } else {
                    OpCode::SliceDyn
                };
                self.op_4(op, ret, arr_reg, start_reg, end_reg);
            }
            NonNil(expression, _) => {
                self.emit_expression(expression, ret, brk);
                self.op_non_nil(ret);
            }
            Prefix(token, expression) => {
                let reg = self.closures.add_reg();
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
                    let ret = if !ret.is_empty() {
                        ret
                    } else {
                        self.closures.add_reg()
                    };
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::If, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Operator::LogicalOr {
                    let ret = if !ret.is_empty() {
                        ret
                    } else {
                        self.closures.add_reg()
                    };
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::IfNot, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Operator::NullCoalescing {
                    let ret = if !ret.is_empty() {
                        ret
                    } else {
                        self.closures.add_reg()
                    };
                    self.emit_expression(left, ret, brk);
                    self.op_if(OpCode::IfNil, ret);
                    self.emit_expression(right, ret, brk);
                    self.op_if_end();
                } else if **token == Keyword::In && is_global_expression(right) {
                    let left_reg = self.closures.add_reg();
                    self.emit_expression(left, left_reg, brk);
                    self.op_unary(ret, OpCode::InGlobal, left_reg);
                } else {
                    let Some(op) = (match token.kind {
                        TokenKind::Operator(o) => o.to_infix_op(),
                        TokenKind::Keyword(Keyword::In) => Some(OpCode::In),
                        _ => None,
                    }) else {
                        // Unexpected infix operator
                        return self.unreachable(token, token, file!(), line!());
                    };
                    let left_reg = self.closures.add_reg();
                    let right_reg = self.closures.add_reg();
                    self.emit_expression(left, left_reg, brk);
                    self.emit_expression(right, right_reg, brk);
                    self.op_binary(ret, op, left_reg, right_reg);
                }
            }
            Is(expression, _, pattern) => {
                let reg_exp = self.closures.add_reg();
                self.emit_expression(expression, reg_exp, brk);
                self.emit_pattern(ret, pattern, reg_exp, Some(BindType::Init));
            }
            Block(_, stmts, ret_expr, _) => {
                self.enter_scope(expr.range());
                self.declare_block(stmts, ret_expr);
                self.emit_block(stmts, ret_expr, ret, brk);
                self.exit_scope();
            }
            Loop(_, expression) => {
                self.closures.enter(false, 0);
                self.enter_scope(expression.range());
                let pos = self.chunk.code.len();
                self.op(OpCode::Loop);
                let Expression::Block(_, stmts, expr, _) = expression.as_ref() else {
                    // unreachable!("Expected block expression");
                    return;
                };
                self.emit_block(stmts, expr, Register::EMPTY, Some(ret));
                self.op(OpCode::LoopEnd);

                let nreg: OpParam = self.closures.current().reg_len().into();
                if nreg.is_wide() {
                    self.chunk.code[pos] = OpCode::Loop.wide_code();
                    self.chunk.code.splice(pos + 1..pos + 1, nreg.wide_code());
                } else {
                    self.chunk.code.splice(pos + 1..pos + 1, [nreg.code()]);
                }

                self.exit_scope();
                self.closures.exit();
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
                    self.closures.add_reg()
                } else {
                    Register::EMPTY
                };

                self.closures.enter(false, 0);
                self.enter_scope(kw.range.end..body.range().end);

                let cond_reg = self.closures.add_reg();
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

                let nreg: OpParam = self.closures.current().reg_len().into();
                if nreg.is_wide() {
                    self.chunk.code[pos] = OpCode::Loop.wide_code();
                    self.chunk.code.splice(pos + 1..pos + 1, nreg.wide_code());
                } else {
                    self.chunk.code.splice(pos + 1..pos + 1, [nreg.code()]);
                }
                self.exit_scope();
                self.closures.exit();

                if !ret.is_empty() {
                    self.op_if(OpCode::IfNotInit, ret);
                    if let Some(ElseBlock(_, else_expr)) = else_part {
                        self.emit_expression(else_expr, ret, None);
                    } else {
                        self.op_nil(ret);
                    }
                    self.op_if_end();
                }
            }
            ForIn(kw, pattern, _, iterable, expression, else_part) => {
                let Expression::Block(_, stmts, expr, _) = expression.as_ref() else {
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

                let (loop_op, loop_iterable_reg1, loop_iterable_reg2) = match iterable.as_ref() {
                    Iterable::Value(value) => (
                        OpCode::LoopFor,
                        self.emit_expression_reg(value, None),
                        // 按命令解释为 NOOP
                        Register::EMPTY,
                    ),
                    Iterable::Range(range) => {
                        let start = self.emit_expression_reg(&range.0, None);
                        let end = self.emit_expression_reg(&range.2, None);
                        (
                            if range.exclusive() {
                                OpCode::LoopRangeExclusive
                            } else {
                                OpCode::LoopRange
                            },
                            start,
                            end,
                        )
                    }
                };

                self.closures.enter(false, 0);
                // 根据虚拟机定义，iterator 寄存器为闭包内第一个寄存器
                // 进入 closure 后立即分配
                let iterator = self.closures.add_reg();
                self.declare_block(stmts, expr);

                let ret = if !ret.is_empty() {
                    self.op_uninit(ret);
                    ret
                } else if else_part.is_some() {
                    self.closures.add_reg()
                } else {
                    Register::EMPTY
                };

                let pos = self.chunk.code.len();
                self.op(loop_op);
                self.emit_pattern(Register::EMPTY, pattern, iterator, Some(BindType::Init));
                self.emit_block(stmts, expr, Register::EMPTY, Some(ret));
                self.op(OpCode::LoopEnd);
                let nreg: OpParam = self.closures.current().reg_len().into();
                if nreg.is_wide() || loop_iterable_reg1.is_wide() || loop_iterable_reg2.is_wide() {
                    self.chunk.code[pos] = loop_op.wide_code();
                    self.chunk.code.splice(
                        pos + 1..pos + 1,
                        [
                            nreg.wide_code(),
                            loop_iterable_reg1.wide_code(),
                            loop_iterable_reg2.wide_code(),
                        ]
                        .concat(),
                    );
                } else {
                    self.chunk.code.splice(
                        pos + 1..pos + 1,
                        [
                            nreg.code(),
                            loop_iterable_reg1.code(),
                            loop_iterable_reg2.code(),
                        ],
                    );
                }
                self.exit_scope();
                self.closures.exit();

                if !ret.is_empty() {
                    self.op_if(OpCode::IfNotInit, ret);
                    if let Some(ElseBlock(_, else_expr)) = else_part {
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

                let cond_reg = self.closures.add_reg();
                self.emit_expression(cond, cond_reg, brk);
                self.op_if(OpCode::If, cond_reg);
                self.emit_expression(then_expr, ret, brk);
                if let Some(ElseBlock(_, else_expr)) = else_part {
                    self.op_else();
                    self.emit_expression(else_expr, ret, brk);
                } else if !ret.is_empty() {
                    self.op_else();
                    self.op_nil(ret);
                }
                self.op_if_end();

                self.exit_scope();
            }
            Match(_, expression, _, items, _) => {
                let matcher = self.closures.add_reg();
                self.emit_expression(expression, matcher, brk);

                let matched = self.closures.add_reg();
                self.op_bool(matched, false);

                if !ret.is_empty() {
                    self.op_nil(ret);
                }

                for (kw_case, pattern, body) in items {
                    let Expression::Block(_, stmts, expr, end) = body else {
                        return;
                    };

                    self.enter_scope(kw_case.range.end..end.range.end);

                    self.declare_pattern(pattern, Some(BindType::Init));
                    self.declare_block(stmts, expr);

                    self.op_if(OpCode::IfNot, matched);
                    {
                        self.emit_pattern(matched, pattern, matcher, Some(BindType::Init));
                        self.op_if(OpCode::If, matched);
                        {
                            self.emit_block(stmts, expr, ret, brk);
                        }
                        self.op_if_end();
                    }
                    self.op_if_end();
                    self.exit_scope();
                }
            }
            Function(kw, args, body) => {
                self.emit_fn(ret, kw.range.end, args, body);
            }
            Unknown { .. } => {
                // Load nil as result of unknown expression
                self.op_nil(ret);
            }
        }
    }
}

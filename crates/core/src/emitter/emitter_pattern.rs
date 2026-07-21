use std::ops::Deref;

use crate::{
    diagnostic::DiagnosticCode,
    lexer::{Keyword, Operator, TokenKind},
    parser::{
        ArrayElementBase, AstWalker,
        Pattern::{self, *},
        RecordElementBase, TokenRef,
    },
};

use super::{
    Emitter, OpCode,
    constant::Constant,
    emitter_pub::ModuleExports,
    emitter_scope::check_variable_initialized,
    opcode::{OpParam, Register},
    variable::BindType,
};

impl<'s, 'c> Emitter<'s, 'c> {
    pub fn declare_pattern<'a>(
        &mut self,
        pattern: &'s Pattern<'s, 'a>,
        bind_type: Option<BindType>,
        kw_pub: &Option<TokenRef<'s>>,
        exports: &mut ModuleExports<'s, 'c>,
    ) {
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
                    self.diagnostics.push(
                        DiagnosticCode::UnnecessaryParentheses,
                        op.range().start..cp.range().end,
                    );
                }
                self.declare_pattern(pattern, bind_type, kw_pub, exports);
            }
            Literal(_, _) => (),
            Constant(_) => (),
            Range(_, _, _) | Relation(_, _) => (), // It can only include a constant pattern
            Discard(_) | SpreadDiscard(_) => (),
            Bind(mut_token, id_token) => {
                let Some(bind_type) = bind_type else {
                    return;
                };
                let Some(variable) =
                    self.declare_variable(id_token, mut_token.is_some(), bind_type)
                else {
                    return;
                };
                let name = variable.name();
                self.declare_pub(exports, kw_pub, name);
            }
            Record(_, elements, _) => {
                for element in elements {
                    match element.deref() {
                        RecordElementBase::Named(_, _, pattern)
                        | RecordElementBase::InterpolateNamed(_, _, pattern)
                        | RecordElementBase::OmitNamed(_, pattern)
                        | RecordElementBase::Unnamed(pattern)
                        | RecordElementBase::Spread(_, pattern) => {
                            self.declare_pattern(pattern, bind_type, kw_pub, exports);
                        }
                    }
                }
            }
            Array(_, elements, _) => {
                for element in elements {
                    match element.deref() {
                        ArrayElementBase::Element(pattern)
                        | ArrayElementBase::Spread(_, pattern) => {
                            self.declare_pattern(pattern, bind_type, kw_pub, exports);
                        }
                    }
                }
            }
            And(left, _, right) | Or(left, _, right) => {
                self.declare_pattern(left, bind_type, kw_pub, exports);
                self.declare_pattern(right, bind_type, kw_pub, exports);
            }
            Not(_, pattern) => self.declare_pattern(pattern, bind_type, kw_pub, exports),
            Unknown { .. } => (),
        }
    }

    fn emit_failed_pattern<'a>(
        &mut self,
        pattern: &'s Pattern<'s, 'a>,
        bind_type: Option<BindType>,
    ) {
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
                    self.op_unary(pattern.range(), register, OpCode::Assign, Register::EMPTY);
                } else {
                    let register = variable.register();
                    let level = self.closures.len() - level;
                    self.op_set_upvalue(pattern.range(), Register::EMPTY, level, register);
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

    fn emit_literal_constant<'a>(
        &mut self,
        pattern_constant: &'s Pattern<'s, 'a>,
        value: Register,
    ) -> Option<Constant<'s>> {
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
                    self.op_number(pattern_constant.range(), value, lit_num);
                    Some(Constant::Number(lit_num))
                } else {
                    match &lit.kind {
                        TokenKind::Keyword(Keyword::Nil) => {
                            self.op_nil(pattern_constant.range(), value);
                            Some(Constant::Nil)
                        }
                        TokenKind::Keyword(Keyword::True) => {
                            self.op_bool(pattern_constant.range(), value, true);
                            Some(Constant::True)
                        }
                        TokenKind::Keyword(Keyword::False) => {
                            self.op_bool(pattern_constant.range(), value, false);
                            Some(Constant::False)
                        }
                        TokenKind::String(s, _) => {
                            self.op_string(pattern_constant.range(), value, s.as_ref());
                            Some(Constant::String(s.as_ref().into()))
                        }
                        _ => {
                            self.unreachable(prefix, lit, file!(), line!());
                            None
                        }
                    }
                }
            }
            Constant(var) => {
                self.emit_var_read(var, value);
                None
            }
            _ => None,
        }
    }

    fn emit_constant_pattern<'a, const SAME: bool>(
        &mut self,
        success: Register,
        pattern: &'s Pattern<'s, 'a>,
        value: Register,
    ) -> bool {
        let op: OpCode = if SAME { OpCode::Same } else { OpCode::Nsame };
        match pattern {
            Literal(_, lit) => {
                if success.is_empty() {
                    self.diagnostics.push(
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                        pattern.range(),
                    );
                    return true;
                }
                if lit.kind == Keyword::Nil {
                    self.op_3(pattern.range(), op, success, Register::EMPTY, value);
                } else {
                    let reg = self.closures.add_reg();
                    self.emit_literal_constant(pattern, reg);
                    self.op_3(pattern.range(), op, success, reg, value);
                }
                true
            }
            Constant(_) => {
                if success.is_empty() {
                    self.diagnostics.push(
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                        pattern.range(),
                    );
                    return true;
                }
                let reg = self.closures.add_reg();
                self.emit_literal_constant(pattern, reg);
                self.op_3(pattern.range(), op, success, reg, value);
                true
            }
            Grouping(_, pattern, _) => self.emit_constant_pattern::<SAME>(success, pattern, value),
            _ => false,
        }
    }

    fn emit_literal_guard<'a>(
        &mut self,
        success: Register,
        pattern: &'s Pattern<'s, 'a>,
        value: Register,
        literal: Constant<'s>,
    ) {
        match literal {
            Constant::Nil => self.op_binary(
                pattern.range(),
                success,
                OpCode::Same,
                Register::EMPTY,
                value,
            ),
            Constant::True | Constant::False => {
                self.op_unary(pattern.range(), success, OpCode::IsBoolean, value)
            }
            Constant::Ordinal(_) | Constant::Number(_) => {
                self.op_unary(pattern.range(), success, OpCode::IsNumber, value)
            }
            Constant::String(_) => self.op_unary(pattern.range(), success, OpCode::IsString, value),
        }
    }
    fn emit_constant_guard<'a>(
        &mut self,
        success: Register,
        pattern: &'s Pattern<'s, 'a>,
        value: Register,
        constant: Register,
    ) {
        let constant_type = self.closures.add_reg();
        self.op_unary(pattern.range(), constant_type, OpCode::Type, constant);
        let value_type = self.closures.add_reg();
        self.op_unary(pattern.range(), value_type, OpCode::Type, value);
        self.op_binary(
            pattern.range(),
            success,
            OpCode::Same,
            constant_type,
            value_type,
        );
    }

    pub fn emit_pattern<'a>(
        &mut self,
        success: Register,
        pattern: &'s Pattern<'s, 'a>,
        value: Register,
        bind_type: Option<BindType>,
    ) {
        if self.emit_constant_pattern::<true>(success, pattern, value) {
            return;
        }
        match pattern {
            Grouping(_, pattern, _) => self.emit_pattern(success, pattern, value, bind_type),
            Relation(op, constant) => {
                if success.is_empty() {
                    self.diagnostics.push(
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                        pattern.range(),
                    );
                    return;
                }
                let Some((op, opcode)) = (match op.kind {
                    TokenKind::Operator(op) if op.is_relation() => {
                        op.to_infix_op().map(|c| (op, c))
                    }
                    _ => None,
                }) else {
                    self.unreachable(op, constant, file!(), line!());
                    return;
                };
                let const_reg = self.closures.add_reg();

                if let Some(lit) = self.emit_literal_constant(constant, const_reg) {
                    if op.is_comparison() {
                        // 近似比较运算和关系运算，仅支持数字和字符串
                        if !matches!(
                            lit,
                            Constant::Number(_) | Constant::Ordinal(_) | Constant::String(_)
                        ) {
                            self.diagnostics.push(
                                DiagnosticCode::NonNumberOrStringInComparison,
                                constant.range(),
                            );
                        }
                    } else {
                        // 相等运算，本身不进行类型转换，字面量支持所有常量类型
                    }
                    self.emit_literal_guard(success, pattern, value, lit);
                } else {
                    self.emit_constant_guard(success, pattern, value, const_reg);
                }
                self.op_if(pattern.range(), OpCode::If, success);
                self.op_binary(pattern.range(), success, opcode, value, const_reg);
                self.op_if_end(pattern.range());
            }
            Range(l, token, r) => {
                if success.is_empty() {
                    self.diagnostics.push(
                        DiagnosticCode::UnnecessaryIrrefutablePattern,
                        pattern.range(),
                    );
                    return;
                }
                let start = self.closures.add_reg();
                let end = self.closures.add_reg();
                self.emit_literal_guard(success, pattern, value, Constant::Ordinal(0));
                self.op_if(pattern.range(), OpCode::If, success);
                self.emit_literal_constant(l, start);
                self.emit_literal_constant(r, end);
                self.op_3(pattern.range(), OpCode::Lte, start, start, value);
                self.op_3(
                    pattern.range(),
                    if token.kind == Operator::HalfOpenRange {
                        OpCode::Gt
                    } else {
                        OpCode::Gte
                    },
                    end,
                    end,
                    value,
                );
                self.op_3(pattern.range(), OpCode::And, success, start, end);
                self.op_if_end(pattern.range());
            }
            Discard(_) => {
                if success.is_empty() {
                    return;
                }
                self.op_bool(pattern.range(), success, true);
            }
            SpreadDiscard(_) => {
                if success.is_empty() {
                    return;
                }
                self.op_bool(pattern.range(), success, true);
            }
            Bind(_, id_token) => {
                let Some(id) = id_token.to_id_name() else {
                    return;
                };
                if !success.is_empty() {
                    // binding always succeeds
                    self.op_bool(pattern.range(), success, true);
                }
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    let bind = bind_type.is_some();
                    if bind {
                        self.closures.initialize_variable(variable);
                    } else {
                        variable.mark_write(id_token);
                        if !check_variable_initialized(
                            &mut self.diagnostics,
                            &self.closures,
                            id_token,
                            variable,
                            level,
                        ) {
                            return;
                        }
                    }
                    if !variable.mutable() && !bind {
                        self.diagnostics.push(
                            DiagnosticCode::ImmutableVariableAssignment,
                            id_token.range(),
                        );
                        variable.put_decl_ref(&mut self.diagnostics);
                    } else if level == self.closures.len() {
                        let register = variable.register();
                        self.op_unary(pattern.range(), register, OpCode::Assign, value);
                    } else {
                        let register = variable.register();
                        let level = self.closures.len() - level;
                        self.op_set_upvalue(pattern.range(), value, level, register);
                    }
                } else {
                    self.diagnostics.push(
                        DiagnosticCode::UndefinedVariableAssignment,
                        id_token.range.clone(),
                    );
                }
            }
            Record(_, elements, _) => {
                let is_empty = elements.is_empty();
                let (has_rest, has_discard_rest) = elements.last().map_or((false, false), |e| {
                    let RecordElementBase::Spread(_, pattern) = e.deref() else {
                        return (false, false);
                    };
                    (true, matches!(pattern.as_ref(), SpreadDiscard(_)))
                });
                if success.is_empty() {
                    if is_empty {
                        self.diagnostics.push(
                            DiagnosticCode::UnnecessaryIrrefutablePattern,
                            pattern.range(),
                        );
                        return;
                    }
                } else {
                    self.op_2(pattern.range(), OpCode::IsRecord, success, value);
                }

                // 不论是否成功匹配，都进行子模式匹配
                // 以便使用 let (:e) = mod; 语法对非记录进行解构

                let sub_flag = if success.is_empty() {
                    // 此时匹配成功与否并不重要，而且也要把这是非条件匹配的信息传到子级
                    Register::EMPTY
                } else {
                    self.closures.add_reg()
                };

                let test_len = !success.is_empty() && !has_rest;
                let expected_len_reg = if test_len {
                    let reg = self.closures.add_reg();
                    self.op_number(pattern.range(), reg, elements.len() as f64);
                    reg
                } else {
                    Register::EMPTY
                };

                let mut omitted = if has_rest && !has_discard_rest {
                    Some(vec![])
                } else {
                    None
                };

                fn is_optional(token: &TokenRef<'_>) -> bool {
                    *token.as_ref() == Operator::QuestionColon
                }
                for (i, element) in elements.iter().enumerate() {
                    match element.deref() {
                        RecordElementBase::Named(token, colon, pattern) => {
                            let Some((id_type, id)) = token.to_field_name() else {
                                unreachable!("Expected identifier token");
                            };
                            self.diagnostics.push(id_type, token.range());
                            let const_id = self.add_const_string(id);
                            if test_len && is_optional(colon) {
                                let has_field = self.closures.add_reg();
                                self.op_3(element.range(), OpCode::Has, has_field, value, const_id);
                                self.op_2(element.range(), OpCode::Not, has_field, has_field);
                                self.op_3(
                                    element.range(),
                                    OpCode::Sub,
                                    expected_len_reg,
                                    expected_len_reg,
                                    has_field,
                                );
                            }
                            let required = !sub_flag.is_empty() && !is_optional(colon);
                            if required {
                                self.op_3(element.range(), OpCode::Has, sub_flag, value, const_id);
                                self.op_if(element.range(), OpCode::If, sub_flag);
                            }
                            let ret = self.closures.add_reg();
                            self.op_3(element.range(), OpCode::Get, ret, value, const_id);
                            self.emit_pattern(sub_flag, pattern, ret, bind_type);
                            if let Some(omitted) = omitted.as_mut() {
                                omitted.push(const_id);
                            }
                            if required {
                                self.op_else(element.range());
                                self.emit_failed_pattern(pattern, bind_type);
                                self.op_if_end(element.range());
                            }
                            if !sub_flag.is_empty() {
                                self.op_3(element.range(), OpCode::And, success, success, sub_flag);
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
                            self.diagnostics
                                .push(DiagnosticCode::OmitNamedRecordField, colon.range());
                            self.diagnostics
                                .push(DiagnosticCode::OmitNamedRecordFieldName, id_token.range());
                            let const_id = self.add_const_string(id);
                            if test_len && is_optional(colon) {
                                let has_field = self.closures.add_reg();
                                self.op_3(element.range(), OpCode::Has, has_field, value, const_id);
                                self.op_2(element.range(), OpCode::Not, has_field, has_field);
                                self.op_3(
                                    element.range(),
                                    OpCode::Sub,
                                    expected_len_reg,
                                    expected_len_reg,
                                    has_field,
                                );
                            }
                            let required = !sub_flag.is_empty() && !is_optional(colon);
                            if required {
                                self.op_3(element.range(), OpCode::Has, sub_flag, value, const_id);
                                self.op_if(element.range(), OpCode::If, sub_flag);
                            }
                            let ret = self.closures.add_reg();
                            self.op_3(element.range(), OpCode::Get, ret, value, const_id);
                            self.emit_pattern(sub_flag, pattern, ret, bind_type);
                            if let Some(omitted) = omitted.as_mut() {
                                omitted.push(const_id);
                            }
                            if required {
                                self.op_else(element.range());
                                self.emit_failed_pattern(pattern, bind_type);
                                self.op_if_end(element.range());
                            }
                            if !sub_flag.is_empty() {
                                self.op_3(element.range(), OpCode::And, success, success, sub_flag);
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
                            self.diagnostics.push(code, start..start);
                            if !sub_flag.is_empty() {
                                self.op_3(
                                    element.range(),
                                    OpCode::HasIndex,
                                    sub_flag,
                                    value,
                                    OpParam::from(i),
                                );
                                self.op_if(element.range(), OpCode::If, sub_flag);
                            }
                            self.op_get_index(element.range(), ret, value, i as i32);
                            self.emit_pattern(sub_flag, pattern, ret, bind_type);
                            if let Some(omitted) = omitted.as_mut() {
                                let const_id = self.add_const_ordinal(i as i32);
                                omitted.push(const_id);
                            }
                            if !sub_flag.is_empty() {
                                self.op_else(element.range());
                                self.emit_failed_pattern(pattern, bind_type);
                                self.op_if_end(element.range());
                                self.op_3(element.range(), OpCode::And, success, success, sub_flag);
                            }
                        }
                        RecordElementBase::Spread(_, pattern) => {
                            if let Some(omitted) = std::mem::take(&mut omitted) {
                                let ret = self.closures.add_reg();
                                self.op_variadic_1(
                                    element.range(),
                                    ret,
                                    OpCode::Omit,
                                    value,
                                    omitted,
                                );
                                self.emit_pattern(sub_flag, pattern, ret, bind_type);
                                if !sub_flag.is_empty() {
                                    self.op_3(
                                        element.range(),
                                        OpCode::And,
                                        success,
                                        success,
                                        sub_flag,
                                    );
                                }
                            }
                        }
                    }
                }

                // 最后进行长度测试
                if !success.is_empty() && !has_rest {
                    self.op_if(pattern.range(), OpCode::If, success);
                    let len_reg = self.closures.add_reg();
                    self.op_2(pattern.range(), OpCode::Length, len_reg, value);
                    self.op_3(
                        pattern.range(),
                        OpCode::Eq,
                        success,
                        len_reg,
                        expected_len_reg,
                    );
                    self.op_if_end(pattern.range());
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
                self.op_2(pattern.range(), OpCode::IsArray, flag, value);

                self.op_if(pattern.range(), OpCode::If, flag);

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
                    self.op_get_index(item.range(), ret, value, i as i32);
                    self.emit_pattern(sub_flag, pattern, ret, bind_type);

                    if !sub_flag.is_empty() {
                        self.op_3(item.range(), OpCode::And, flag, flag, sub_flag);
                    }
                }
                if let Some(spread) = spread
                    && let ArrayElementBase::Spread(_, spread_inner) = spread
                {
                    for (i, item) in after.iter().rev().enumerate() {
                        let ArrayElementBase::Element(pattern) = item.deref() else {
                            unreachable!();
                        };

                        let ret = self.closures.add_reg();
                        self.op_get_index(item.range(), ret, value, -1 - (i as i32));
                        self.emit_pattern(sub_flag, pattern, ret, bind_type);

                        if !sub_flag.is_empty() {
                            self.op_3(item.range(), OpCode::And, flag, flag, sub_flag);
                        }
                    }

                    if !matches!(spread_inner.as_ref(), SpreadDiscard(_)) {
                        let ret = self.closures.add_reg();
                        if before.is_empty() && after.is_empty() {
                            // 如果没有前后元素，直接返回整个数组
                            self.op_2(spread.range(), OpCode::Assign, ret, value);
                        } else if after.is_empty() {
                            // 切片前面的元素
                            self.op_3(
                                spread.range(),
                                OpCode::SliceEnd,
                                ret,
                                value,
                                OpParam::from(before.len()),
                            );
                        } else if before.is_empty() {
                            // 切片后面的元素
                            self.op_3(
                                spread.range(),
                                OpCode::SliceStart,
                                ret,
                                value,
                                OpParam::from(-(after.len() as i32) - 1),
                            );
                        } else {
                            // 切片前后都有元素
                            self.op_4(
                                spread.range(),
                                OpCode::Slice,
                                ret,
                                value,
                                OpParam::from(before.len()),
                                OpParam::from(-(after.len() as i32) - 1),
                            );
                        }
                        self.emit_pattern(sub_flag, spread_inner, ret, bind_type);

                        if !sub_flag.is_empty() {
                            self.op_3(spread.range(), OpCode::And, flag, flag, sub_flag);
                        }
                    }
                }

                self.op_else(pattern.range());
                self.emit_failed_pattern(pattern, bind_type);

                self.op_if_end(pattern.range());

                // 最后进行长度测试，避免 [1] 匹配 [x, y] 时 x 也为 nil
                if !success.is_empty() && (len > 0 || spread.is_none()) {
                    self.op_if(pattern.range(), OpCode::If, flag);
                    let len_reg = self.closures.add_reg();
                    self.op_2(pattern.range(), OpCode::Length, len_reg, value);
                    let expected_len_reg = self.closures.add_reg();
                    self.op_number(pattern.range(), expected_len_reg, len as f64);
                    self.op_3(
                        pattern.range(),
                        if spread.is_some() {
                            OpCode::Gte
                        } else {
                            OpCode::Eq
                        },
                        flag,
                        len_reg,
                        expected_len_reg,
                    );
                    self.op_if_end(pattern.range());
                }
            }
            And(left, op, right) | Or(left, op, right) => {
                // No short-circuiting in pattern matching
                if success.is_empty() {
                    self.emit_pattern(Register::EMPTY, left, value, bind_type);
                    self.emit_pattern(Register::EMPTY, right, value, bind_type);
                } else {
                    let opcode = if *op.as_ref() == Keyword::And {
                        OpCode::And
                    } else {
                        OpCode::Or
                    };
                    let right_success = self.closures.add_reg();
                    self.emit_pattern(success, left, value, bind_type);
                    self.emit_pattern(right_success, right, value, bind_type);
                    self.op_3(op.range(), opcode, success, right_success, success);
                }
            }
            Not(kw, p) => {
                if success.is_empty() {
                    self.diagnostics
                        .push(DiagnosticCode::UnnecessaryIrrefutablePattern, kw.range());
                    self.emit_pattern(Register::EMPTY, p, value, bind_type);
                } else if self.emit_constant_pattern::<false>(success, p, value) {
                } else {
                    self.emit_pattern(success, p, value, bind_type);
                    self.op_2(kw.range(), OpCode::Not, success, success);
                }
            }
            Unknown { .. } => (),
            Literal(..) => unreachable!(),
            Constant(_) => unreachable!(),
        }
    }
}

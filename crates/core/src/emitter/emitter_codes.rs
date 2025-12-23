use std::borrow::Cow;

use crate::SourceRange;

use super::{
    Emitter,
    constant::Constant as C,
    opcode::{
        OpCode::{self, *},
        OpParam, OpParamTrait, Register,
    },
};

impl<'s, 'c> Emitter<'s, 'c> {
    pub fn add_const_string(&mut self, value: impl Into<Cow<'s, str>>) -> OpParam {
        self.chunk.add_constant(C::String(value.into()))
    }
    pub fn add_const_number(&mut self, value: f64) -> OpParam {
        self.chunk.add_constant(C::Number(value))
    }
    pub fn add_const_ordinal(&mut self, value: i32) -> OpParam {
        self.chunk.add_constant(C::Ordinal(value))
    }
    pub fn add_const_bool(&mut self, value: bool) -> OpParam {
        self.chunk
            .add_constant(if value { C::True } else { C::False })
    }

    pub fn op(&mut self, mapping: SourceRange, op: OpCode) {
        self.chunk.add_code(op);
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_1(&mut self, mapping: SourceRange, op: OpCode, arg1: impl OpParamTrait) {
        if !arg1.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_2(
        &mut self,
        mapping: SourceRange,
        op: OpCode,
        arg1: impl OpParamTrait,
        arg2: impl OpParamTrait,
    ) {
        if !arg1.is_wide() && !arg2.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
            self.chunk.add_param(arg2);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(arg2);
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_3(
        &mut self,
        mapping: SourceRange,
        op: OpCode,
        arg1: impl OpParamTrait,
        arg2: impl OpParamTrait,
        arg3: impl OpParamTrait,
    ) {
        if !arg1.is_wide() && !arg2.is_wide() && !arg3.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
            self.chunk.add_param(arg2);
            self.chunk.add_param(arg3);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(arg2);
            self.chunk.add_param_wide(arg3);
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_4(
        &mut self,
        mapping: SourceRange,
        op: OpCode,
        arg1: impl OpParamTrait,
        arg2: impl OpParamTrait,
        arg3: impl OpParamTrait,
        arg4: impl OpParamTrait,
    ) {
        if !arg1.is_wide() && !arg2.is_wide() && !arg3.is_wide() && !arg4.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
            self.chunk.add_param(arg2);
            self.chunk.add_param(arg3);
            self.chunk.add_param(arg4);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(arg2);
            self.chunk.add_param_wide(arg3);
            self.chunk.add_param_wide(arg4);
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    fn op_const(&mut self, mapping: SourceRange, reg: Register, const_id: OpParam) {
        self.op_2(mapping, Constant, reg, const_id);
    }
    pub fn op_global(
        &mut self,
        mapping: SourceRange,
        reg: Register,
        name: impl Into<Cow<'s, str>>,
    ) {
        let const_id = self.add_const_string(name);
        self.op_2(mapping, GetGlobal, reg, const_id);
    }
    pub fn op_global_num(&mut self, mapping: SourceRange, reg: Register, name: f64) {
        let const_id = self.add_const_number(name);
        self.op_2(mapping, GetGlobal, reg, const_id);
    }
    pub fn op_global_dyn(&mut self, mapping: SourceRange, reg: Register, name: Register) {
        self.op_2(mapping, GetGlobalDyn, reg, name);
    }
    pub fn op_nil(&mut self, mapping: SourceRange, reg: Register) {
        self.op_unary(mapping, reg, Assign, Register::EMPTY);
    }
    pub fn op_number(&mut self, mapping: SourceRange, reg: Register, value: f64) {
        let const_id = self.add_const_number(value);
        self.op_const(mapping, reg, const_id);
    }
    pub fn op_bool(&mut self, mapping: SourceRange, reg: Register, value: bool) {
        let const_id = self.add_const_bool(value);
        self.op_const(mapping, reg, const_id);
    }
    pub fn op_string(
        &mut self,
        mapping: SourceRange,
        reg: Register,
        value: impl Into<Cow<'s, str>>,
    ) {
        let const_id = self.add_const_string(value);
        self.op_const(mapping, reg, const_id);
    }
    pub fn op_uninit(&mut self, mapping: SourceRange, reg: Register) {
        self.op_1(mapping, OpCode::Uninit, reg);
    }
    pub fn op_format(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        val: Register,
        fmt: impl Into<Cow<'s, str>>,
    ) {
        let fmt = self.add_const_string(fmt);
        self.op_3(mapping, OpCode::Format, ret, val, fmt);
    }
    pub fn op_unary(&mut self, mapping: SourceRange, ret: Register, op: OpCode, reg: Register) {
        self.op_2(mapping, op, ret, reg);
    }
    pub fn op_binary(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        op: OpCode,
        left: Register,
        right: Register,
    ) {
        self.op_3(mapping, op, ret, left, right);
    }
    pub fn op_variadic(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        op: OpCode,
        var_args: Vec<impl OpParamTrait>,
    ) {
        let n: OpParam = var_args.len().into();
        if !n.is_wide() && !ret.is_wide() && !var_args.iter().any(|r| r.is_wide()) {
            self.chunk.add_code(op);
            self.chunk.add_param(ret);
            self.chunk.add_param(n);
            for r in var_args {
                self.chunk.add_param(r);
            }
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(n);
            for r in var_args {
                self.chunk.add_param_wide(r);
            }
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_variadic_1(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        op: OpCode,
        arg1: impl OpParamTrait,
        var_args: Vec<impl OpParamTrait>,
    ) {
        let n: OpParam = var_args.len().into();
        if !n.is_wide()
            && !ret.is_wide()
            && !arg1.is_wide()
            && !var_args.iter().any(|r| r.is_wide())
        {
            self.chunk.add_code(op);
            self.chunk.add_param(ret);
            self.chunk.add_param(arg1);
            self.chunk.add_param(n);
            for r in var_args {
                self.chunk.add_param(r);
            }
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(n);
            for r in var_args {
                self.chunk.add_param_wide(r);
            }
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_variadic_variadic_1(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        op: OpCode,
        arg1: impl OpParamTrait,
        var_args1: Vec<impl OpParamTrait>,
        var_args2: Vec<impl OpParamTrait>,
    ) {
        let n1: OpParam = var_args1.len().into();
        let n2: OpParam = var_args2.len().into();
        if !n1.is_wide()
            && !n2.is_wide()
            && !ret.is_wide()
            && !arg1.is_wide()
            && !var_args1.iter().any(|r| r.is_wide())
            && !var_args2.iter().any(|r| r.is_wide())
        {
            self.chunk.add_code(op);
            self.chunk.add_param(ret);
            self.chunk.add_param(arg1);
            self.chunk.add_param(n1);
            for r in var_args1 {
                self.chunk.add_param(r);
            }
            self.chunk.add_param(n2);
            for r in var_args2 {
                self.chunk.add_param(r);
            }
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(n1);
            for r in var_args1 {
                self.chunk.add_param_wide(r);
            }
            self.chunk.add_param_wide(n2);
            for r in var_args2 {
                self.chunk.add_param_wide(r);
            }
        }
        self.diagnostics
            .push(crate::DiagnosticCode::SourceMap, mapping);
    }
    pub fn op_non_nil(&mut self, mapping: SourceRange, reg: Register) {
        self.op_1(mapping, AssertNonNil, reg);
    }

    pub fn op_if(&mut self, mapping: SourceRange, if_code: OpCode, cond: Register) {
        self.op_1(mapping, if_code, cond);
    }
    pub fn op_else(&mut self, mapping: SourceRange) {
        self.op(mapping, Else);
    }
    pub fn op_if_end(&mut self, mapping: SourceRange) {
        self.op(mapping, IfEnd);
    }

    pub fn op_return(&mut self, mapping: SourceRange, ret: Register) {
        self.op_1(mapping, Return, ret);
    }

    pub fn op_call(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        func: impl Into<Cow<'s, str>>,
        args: Vec<Register>,
        spreads: Vec<OpParam>,
    ) {
        let f = self.add_const_string(func);
        self.op_variadic_variadic_1(mapping, ret, Call, f, args, spreads)
    }

    pub fn op_call_dyn(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        func: Register,
        args: Vec<Register>,
        spreads: Vec<OpParam>,
    ) {
        self.op_variadic_variadic_1(mapping, ret, CallDyn, func, args, spreads)
    }

    pub fn op_get_upvalue(
        &mut self,
        mapping: SourceRange,
        reg: Register,
        level: usize,
        up_reg: Register,
    ) {
        let level: OpParam = level.into();
        self.op_3(mapping, GetUpvalue, reg, level, up_reg);
    }

    pub fn op_set_upvalue(
        &mut self,
        mapping: SourceRange,
        reg: Register,
        level: usize,
        up_reg: Register,
    ) {
        let level: OpParam = level.into();
        self.op_3(mapping, SetUpvalue, reg, level, up_reg);
    }

    pub fn op_get(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        obj: Register,
        name: impl Into<Cow<'s, str>>,
    ) {
        let const_id = self.add_const_string(name);
        self.op_3(mapping, Get, ret, obj, const_id);
    }

    pub fn op_get_index(&mut self, mapping: SourceRange, ret: Register, obj: Register, index: i32) {
        self.op_3(mapping, GetIndex, ret, obj, OpParam::new(index));
    }

    pub fn op_get_dyn(
        &mut self,
        mapping: SourceRange,
        ret: Register,
        obj: Register,
        index: Register,
    ) {
        self.op_3(mapping, GetDyn, ret, obj, index);
    }
}

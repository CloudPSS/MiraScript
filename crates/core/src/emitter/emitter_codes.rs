use std::borrow::Cow;

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

    pub fn op(&mut self, op: OpCode) {
        self.chunk.add_code(op);
    }
    pub fn op_1(&mut self, op: OpCode, arg1: impl OpParamTrait) {
        if !arg1.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
        }
    }
    pub fn op_2(&mut self, op: OpCode, arg1: impl OpParamTrait, arg2: impl OpParamTrait) {
        if !arg1.is_wide() && !arg2.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
            self.chunk.add_param(arg2);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(arg2);
        }
    }
    pub fn op_3(
        &mut self,
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
    }
    pub fn op_4(
        &mut self,
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
    }
    fn op_const(&mut self, reg: Register, const_id: OpParam) {
        self.op_2(Constant, reg, const_id);
    }
    pub fn op_global(&mut self, reg: Register, name: impl Into<Cow<'s, str>>) {
        let const_id = self.add_const_string(name);
        self.op_2(GetGlobal, reg, const_id);
    }
    pub fn op_global_num(&mut self, reg: Register, name: f64) {
        let const_id = self.add_const_number(name);
        self.op_2(GetGlobal, reg, const_id);
    }
    pub fn op_global_dyn(&mut self, reg: Register, name: Register) {
        self.op_2(GetGlobalDyn, reg, name);
    }
    pub fn op_nil(&mut self, reg: Register) {
        self.op_unary(reg, Assign, Register::EMPTY);
    }
    pub fn op_number(&mut self, reg: Register, value: f64) {
        let const_id = self.add_const_number(value);
        self.op_const(reg, const_id);
    }
    pub fn op_bool(&mut self, reg: Register, value: bool) {
        let const_id = self.add_const_bool(value);
        self.op_const(reg, const_id);
    }
    pub fn op_string(&mut self, reg: Register, value: impl Into<Cow<'s, str>>) {
        let const_id = self.add_const_string(value);
        self.op_const(reg, const_id);
    }
    pub fn op_uninit(&mut self, reg: Register) {
        self.op_1(OpCode::Uninit, reg);
    }
    pub fn op_unary(&mut self, ret: Register, op: OpCode, reg: Register) {
        self.op_2(op, ret, reg);
    }
    pub fn op_binary(&mut self, ret: Register, op: OpCode, left: Register, right: Register) {
        self.op_3(op, ret, left, right);
    }
    pub fn op_variadic(&mut self, ret: Register, op: OpCode, var_args: Vec<impl OpParamTrait>) {
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
    }
    pub fn op_variadic_1(
        &mut self,
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
    }
    pub fn op_variadic_variadic_1(
        &mut self,
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
    }
    pub fn op_non_nil(&mut self, reg: Register) {
        self.op_1(AssertNonNil, reg);
    }

    pub fn op_if(&mut self, if_code: OpCode, cond: Register) {
        self.op_1(if_code, cond);
    }
    pub fn op_else(&mut self) {
        self.op(Else);
    }
    pub fn op_if_end(&mut self) {
        self.op(IfEnd);
    }

    pub fn op_return(&mut self, ret: Register) {
        self.op_1(Return, ret);
    }

    pub fn op_call(
        &mut self,
        ret: Register,
        func: impl Into<Cow<'s, str>>,
        args: Vec<Register>,
        spreads: Vec<OpParam>,
    ) {
        let f = self.add_const_string(func);
        self.op_variadic_variadic_1(ret, Call, f, args, spreads)
    }

    pub fn op_call_dyn(
        &mut self,
        ret: Register,
        func: Register,
        args: Vec<Register>,
        spreads: Vec<OpParam>,
    ) {
        self.op_variadic_variadic_1(ret, CallDyn, func, args, spreads)
    }

    pub fn op_get_upvalue(&mut self, reg: Register, level: usize, up_reg: Register) {
        let level: OpParam = level.into();
        self.op_3(GetUpvalue, reg, level, up_reg);
    }

    pub fn op_set_upvalue(&mut self, reg: Register, level: usize, up_reg: Register) {
        let level: OpParam = level.into();
        self.op_3(SetUpvalue, reg, level, up_reg);
    }

    pub fn op_get(&mut self, ret: Register, obj: Register, name: impl Into<Cow<'s, str>>) {
        let const_id = self.add_const_string(name);
        self.op_3(Get, ret, obj, const_id);
    }

    pub fn op_get_index(&mut self, ret: Register, obj: Register, index: i32) {
        self.op_3(GetIndex, ret, obj, OpParam::new(index));
    }

    pub fn op_get_dyn(&mut self, ret: Register, obj: Register, index: Register) {
        self.op_3(GetDyn, ret, obj, index);
    }
}

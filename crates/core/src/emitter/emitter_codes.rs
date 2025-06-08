use std::borrow::Cow;

use super::{
    Emitter,
    constant::Constant as C,
    opcode::{
        OpCode::{self, *},
        OpParam, OpParamTrait, Register,
    },
};

impl<'s> Emitter<'s> {
    pub fn add_const_string(&mut self, value: impl Into<Cow<'s, str>>) -> OpParam {
        self.chunk.add_constant(C::String(value.into()))
    }
    pub fn add_const_number(&mut self, value: f64) -> OpParam {
        self.chunk.add_constant(C::Number(value))
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
    pub fn op_5(
        &mut self,
        op: OpCode,
        arg1: impl OpParamTrait,
        arg2: impl OpParamTrait,
        arg3: impl OpParamTrait,
        arg4: impl OpParamTrait,
        arg5: impl OpParamTrait,
    ) {
        if !arg1.is_wide()
            && !arg2.is_wide()
            && !arg3.is_wide()
            && !arg4.is_wide()
            && !arg5.is_wide()
        {
            self.chunk.add_code(op);
            self.chunk.add_param(arg1);
            self.chunk.add_param(arg2);
            self.chunk.add_param(arg3);
            self.chunk.add_param(arg4);
            self.chunk.add_param(arg5);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(arg1);
            self.chunk.add_param_wide(arg2);
            self.chunk.add_param_wide(arg3);
            self.chunk.add_param_wide(arg4);
            self.chunk.add_param_wide(arg5);
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
    pub fn op_variadic(&mut self, ret: Register, op: OpCode, reg: Vec<Register>) {
        let n: OpParam = reg.len().into();
        if !n.is_wide() && !ret.is_wide() && !reg.iter().any(|r| r.is_wide()) {
            self.chunk.add_code(op);
            self.chunk.add_param(ret);
            self.chunk.add_param(n);
            for r in reg {
                self.chunk.add_param(r);
            }
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(n);
            for r in reg {
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

    pub fn op_call(&mut self, ret: Register, func: impl Into<Cow<'s, str>>, args: Vec<Register>) {
        let narg: OpParam = args.len().into();
        let f = self.add_const_string(func);
        if !narg.is_wide() && !ret.is_wide() && !f.is_wide() && !args.iter().any(|r| r.is_wide()) {
            self.chunk.add_code(Call);
            self.chunk.add_param(ret);
            self.chunk.add_param(f);
            self.chunk.add_param(narg);
            for r in args {
                self.chunk.add_param(r);
            }
        } else {
            self.chunk.add_code_wide(Call);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(f);
            self.chunk.add_param_wide(narg);
            for r in args {
                self.chunk.add_param_wide(r);
            }
        }
    }

    pub fn op_call_dyn(&mut self, ret: Register, func: Register, args: Vec<Register>) {
        let narg: OpParam = args.len().into();
        if !narg.is_wide() && !ret.is_wide() && !func.is_wide() && !args.iter().any(|r| r.is_wide())
        {
            self.chunk.add_code(CallDyn);
            self.chunk.add_param(ret);
            self.chunk.add_param(func);
            self.chunk.add_param(narg);
            for r in args {
                self.chunk.add_param(r);
            }
        } else {
            self.chunk.add_code_wide(CallDyn);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(func);
            self.chunk.add_param_wide(narg);
            for r in args {
                self.chunk.add_param_wide(r);
            }
        }
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

    pub fn op_get_num(&mut self, ret: Register, obj: Register, index: f64) {
        let const_id = self.add_const_number(index);
        self.op_3(Get, ret, obj, const_id);
    }

    pub fn op_get_index(&mut self, ret: Register, obj: Register, index: usize) {
        let index: OpParam = index.into();
        self.op_3(GetIndex, ret, obj, index);
    }

    pub fn op_get_dyn(&mut self, ret: Register, obj: Register, index: Register) {
        self.op_3(GetDyn, ret, obj, index);
    }
}

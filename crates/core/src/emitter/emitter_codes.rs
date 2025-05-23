use crate::{
    error::SourceError,
    parser::{Expression, Script, Statement},
};

use super::{
    Emitter,
    constant::Constant as C,
    opcode::{
        OpCode::{self, *},
        OpParam, OpParamTrait, Register,
    },
};

impl<'s> Emitter<'s> {
    fn op_const(&mut self, reg: Register, const_id: OpParam) {
        if !const_id.is_wide() && !reg.is_wide() {
            self.chunk.add_code(Constant);
            self.chunk.add_reg(reg);
            self.chunk.add_param(const_id);
        } else {
            self.chunk.add_code_wide(Constant);
            self.chunk.add_reg_wide(reg);
            self.chunk.add_param_wide(const_id);
        }
    }
    pub fn op_global(&mut self, reg: Register, name: &'s str) {
        let const_id = self.chunk.add_constant(C::String(name));
        if reg.is_wide() || const_id.is_wide() {
            self.chunk.add_code_wide(GetGlobal);
            self.chunk.add_reg_wide(reg);
            self.chunk.add_param_wide(const_id);
        } else {
            self.chunk.add_code(GetGlobal);
            self.chunk.add_reg(reg);
            self.chunk.add_param(const_id);
        }
    }
    pub fn op_nil(&mut self, reg: Register) {
        self.op_unary(reg, Assign, Register::EMPTY);
    }
    pub fn op_number(&mut self, reg: Register, value: f64) {
        let const_id = self.chunk.add_constant(C::Number(value));
        self.op_const(reg, const_id);
    }
    pub fn op_bool(&mut self, reg: Register, value: bool) {
        let const_id = self
            .chunk
            .add_constant(if value { C::True } else { C::False });
        self.op_const(reg, const_id);
    }
    pub fn op_string(&mut self, reg: Register, value: &'s str) {
        let const_id = self.chunk.add_constant(C::String(value));
        self.op_const(reg, const_id);
    }
    pub fn op(&mut self, op: OpCode) {
        self.chunk.add_code(op);
    }
    pub fn op_unary(&mut self, ret: Register, op: OpCode, reg: Register) {
        if !reg.is_wide() && !ret.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_reg(ret);
            self.chunk.add_reg(reg);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_reg_wide(ret);
            self.chunk.add_reg_wide(reg);
        }
    }
    pub fn op_binary(&mut self, ret: Register, op: OpCode, left: Register, right: Register) {
        if !left.is_wide() && !right.is_wide() && !ret.is_wide() {
            self.chunk.add_code(op);
            self.chunk.add_reg(ret);
            self.chunk.add_reg(left);
            self.chunk.add_reg(right);
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_reg_wide(ret);
            self.chunk.add_reg_wide(left);
            self.chunk.add_reg_wide(right);
        }
    }
    pub fn op_variadic(&mut self, ret: Register, op: OpCode, reg: Vec<Register>) {
        let n: OpParam = reg.len().into();
        if !n.is_wide() && !ret.is_wide() && !reg.iter().any(|r| r.is_wide()) {
            self.chunk.add_code(op);
            self.chunk.add_reg(ret);
            self.chunk.add_param(n);
            for r in reg {
                self.chunk.add_reg(r);
            }
        } else {
            self.chunk.add_code_wide(op);
            self.chunk.add_reg_wide(ret);
            self.chunk.add_param_wide(n);
            for r in reg {
                self.chunk.add_reg_wide(r);
            }
        }
    }

    pub fn op_if(&mut self, cond: Register) {
        if !cond.is_wide() {
            self.chunk.add_code(If);
            self.chunk.add_reg(cond);
        } else {
            self.chunk.add_code_wide(If);
            self.chunk.add_reg_wide(cond);
        }
    }
    pub fn op_else(&mut self) {
        self.chunk.add_code(Else);
    }
    pub fn op_if_end(&mut self) {
        self.chunk.add_code(IfEnd);
    }

    pub fn op_return(&mut self, ret: Register) {
        if !ret.is_wide() {
            self.chunk.add_code(Return);
            self.chunk.add_reg(ret);
        } else {
            self.chunk.add_code_wide(Return);
            self.chunk.add_reg_wide(ret);
        }
    }

    pub fn op_call(&mut self, ret: Register, func: &'s str, args: Vec<Register>) {
        let narg: OpParam = args.len().into();
        let f = self.chunk.add_constant(C::String(func));
        if !narg.is_wide() && !ret.is_wide() && !f.is_wide() && !args.iter().any(|r| r.is_wide()) {
            self.chunk.add_code(Call);
            self.chunk.add_reg(ret);
            self.chunk.add_param(f);
            self.chunk.add_param(narg);
            for r in args {
                self.chunk.add_reg(r);
            }
        } else {
            self.chunk.add_code_wide(Call);
            self.chunk.add_reg_wide(ret);
            self.chunk.add_param_wide(f);
            self.chunk.add_param_wide(narg);
            for r in args {
                self.chunk.add_reg_wide(r);
            }
        }
    }

    pub fn op_call_dyn(&mut self, ret: Register, func: Register, args: Vec<Register>) {
        let narg: OpParam = args.len().into();
        if !narg.is_wide() && !ret.is_wide() && !func.is_wide() && !args.iter().any(|r| r.is_wide())
        {
            self.chunk.add_code(CallDyn);
            self.chunk.add_reg(ret);
            self.chunk.add_reg(func);
            self.chunk.add_param(narg);
            for r in args {
                self.chunk.add_reg(r);
            }
        } else {
            self.chunk.add_code_wide(CallDyn);
            self.chunk.add_reg_wide(ret);
            self.chunk.add_reg_wide(func);
            self.chunk.add_param_wide(narg);
            for r in args {
                self.chunk.add_reg_wide(r);
            }
        }
    }

    pub fn op_get_upvalue(&mut self, reg: Register, level: usize, up_reg: Register) {
        let level: OpParam = level.into();
        if !reg.is_wide() && !up_reg.is_wide() && !level.is_wide() {
            self.chunk.add_code(GetUpvalue);
            self.chunk.add_reg(reg);
            self.chunk.add_param(level);
            self.chunk.add_reg(up_reg);
        } else {
            self.chunk.add_code_wide(GetUpvalue);
            self.chunk.add_reg_wide(reg);
            self.chunk.add_param_wide(level);
            self.chunk.add_reg_wide(up_reg);
        }
    }

    pub fn op_set_upvalue(&mut self, reg: Register, level: usize, up_reg: Register) {
        let level: OpParam = level.into();
        if !reg.is_wide() && !up_reg.is_wide() && !level.is_wide() {
            self.chunk.add_code(SetUpvalue);
            self.chunk.add_reg(reg);
            self.chunk.add_param(level);
            self.chunk.add_reg(up_reg);
        } else {
            self.chunk.add_code_wide(SetUpvalue);
            self.chunk.add_reg_wide(reg);
            self.chunk.add_param_wide(level);
            self.chunk.add_reg_wide(up_reg);
        }
    }
}

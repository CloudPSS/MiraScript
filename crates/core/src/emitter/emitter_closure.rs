use std::ops::{Deref, DerefMut};

use crate::{
    diagnostic::SourceRange,
    emitter::variable::Variable,
    parser::{ArrayElementBase, AstWalker as _, Expression, ParameterList, Pattern, Statement},
};

use super::{
    Emitter, OpCode,
    closure::Closure,
    opcode::{OpParam, OpParamTrait, Register},
    variable::BindType,
};

pub(super) struct Closures(Vec<Closure>);

impl Closures {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn current(&mut self) -> &mut Closure {
        self.last_mut().unwrap()
    }
    pub fn add_reg(&mut self) -> Register {
        self.current().add_reg()
    }
    pub fn enter(&mut self, late_binding: bool, arg_len: usize) {
        self.push(Closure::new(late_binding, arg_len));
    }
    pub fn exit(&mut self) {
        self.pop();
    }
    pub fn initialize_variable(&mut self, variable: &mut Variable) -> Register {
        let reg = if !variable.has_register() {
            let reg = self.current().add_reg();
            variable.set_register(reg);
            reg
        } else {
            variable.register()
        };
        variable.initialize();
        reg
    }
}

impl Deref for Closures {
    type Target = Vec<Closure>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Closures {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Emitter<'s> {
    pub fn declare_block(
        &mut self,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
    ) {
        for stmt in stmts {
            self.declare_statement(stmt);
        }
        if let Some(expr) = expr {
            self.declare_expression(expr);
        }
    }
    pub fn emit_block(
        &mut self,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
        ret: Register,
        brk: Option<Register>,
    ) -> bool {
        // 提升函数实现
        for stmt in stmts {
            if matches!(stmt, Statement::Function(..)) {
                self.emit_statement(stmt, brk);
            }
        }
        let mut has_never = false;
        for stmt in stmts {
            if !matches!(stmt, Statement::Function(..)) {
                let never = self.emit_statement(stmt, brk);
                has_never |= never;
            }
        }
        if let Some(expr) = expr {
            self.emit_expression(expr, ret, brk);
        } else if !has_never && !ret.is_empty() {
            self.op_nil(ret);
        };
        has_never
    }
    pub fn emit_fn(
        &mut self,
        ret: Register,
        args_range: SourceRange,
        args: &'s Option<ParameterList<'s>>,
        body_range: SourceRange,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
    ) {
        let arg_len = args.as_ref().map_or(1, |args| args.len());
        let mut has_var_args = false;
        self.closures.enter(true, arg_len);
        self.enter_scope(args_range.start..body_range.end);

        let pos = self.chunk.code.len();
        self.chunk.code.push(OpCode::Func.code());

        if let Some(args) = args {
            for (i, arg) in args.iter().enumerate() {
                match arg.deref() {
                    ArrayElementBase::Element(arg) => {
                        if let Pattern::Bind(kw_mut, id_token) = arg.deref() {
                            self.declare_parameter(
                                i,
                                Some(id_token),
                                id_token.range(),
                                kw_mut.is_some(),
                                BindType::Parameter,
                            );
                        } else {
                            self.declare_parameter(
                                i,
                                None,
                                arg.range(),
                                false,
                                BindType::PatternParameter,
                            );
                            self.declare_pattern(arg, Some(BindType::ParameterSubPattern));
                        }
                    }
                    ArrayElementBase::Spread(_, arg) => {
                        if let Pattern::Bind(kw_mut, id_token) = arg.deref() {
                            self.declare_parameter(
                                i,
                                Some(id_token),
                                id_token.range(),
                                kw_mut.is_some(),
                                BindType::RestParameter,
                            );
                        } else {
                            self.declare_parameter(
                                i,
                                None,
                                arg.range(),
                                false,
                                BindType::RestPatternParameter,
                            );
                            self.declare_pattern(arg, Some(BindType::ParameterSubPattern));
                        }
                        has_var_args = true;
                    }
                    ArrayElementBase::Range(..) => unreachable!(),
                }
            }
        } else {
            self.declare_parameter(
                0,
                None,
                args_range.start..args_range.start,
                false,
                BindType::ItParameter,
            );
        }
        self.declare_block(stmts, expr);

        if let Some(args) = args {
            for (i, arg) in args.iter().enumerate() {
                let reg = Register::new(i + 1);
                match arg.deref() {
                    ArrayElementBase::Element(arg) => {
                        if !arg.is_bind() {
                            self.emit_pattern(Register::EMPTY, arg, reg, Some(BindType::Parameter));
                        }
                    }
                    ArrayElementBase::Spread(_, arg) => {
                        if !arg.is_bind() {
                            self.emit_pattern(
                                Register::EMPTY,
                                arg,
                                reg,
                                Some(BindType::RestParameter),
                            );
                        }
                    }
                    ArrayElementBase::Range(..) => unreachable!(),
                }
            }
        }

        let ret_reg = self.closures.add_reg();
        let never = self.emit_block(stmts, expr, ret_reg, None);
        if !never {
            self.op_return(ret_reg);
        }
        self.op(OpCode::FuncEnd);

        let closure = self.closures.current();
        let func_op = if has_var_args {
            OpCode::FuncVarg
        } else {
            OpCode::Func
        };
        let nreg: OpParam = closure.reg_len().into();
        let narg: OpParam = arg_len.into();
        if ret.is_wide() || nreg.is_wide() || narg.is_wide() {
            self.chunk.code[pos] = func_op.wide_code();
            self.chunk.code.splice(
                pos + 1..pos + 1,
                [ret.wide_code(), narg.wide_code(), nreg.wide_code()].concat(),
            );
        } else if !nreg.is_wide() {
            self.chunk.code[pos] = func_op.code();
            self.chunk
                .code
                .splice(pos + 1..pos + 1, [ret.code(), narg.code(), nreg.code()]);
        }
        self.exit_scope();
        self.closures.exit();
    }
}

use std::ops::{Deref, DerefMut};

use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    emitter::variable::Variable,
    lexer::Token,
    parser::{AstWalker, Expression, Statement},
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
    pub fn enter(&mut self, late_binding: bool) {
        self.push(Closure::new(late_binding));
    }
    pub fn exit(&mut self) {
        self.pop();
    }
    pub fn initialize_variable(&mut self, variable: &mut Variable) -> Register {
        if variable.initialized() {
            return variable.register();
        }
        let reg = self.current().add_reg();
        variable.initialize(reg);
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
        args: &'s Option<Vec<Token<'s>>>,
        body_range: SourceRange,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
    ) {
        self.closures.enter(true);
        self.enter_scope(args_range.start..body_range.end);

        let narg: OpParam = args.as_ref().map_or(1, |args| args.len()).into();
        let pos = self.chunk.code.len();
        self.chunk.code.push(OpCode::Func.code());

        if let Some(args) = args {
            for arg in args {
                if arg.is_identifier() {
                    self.declare_variable(arg, false, BindType::Parameter);
                } else {
                    continue; // Skip non-identifier tokens
                }
                self.diagnostics.push(SourceDiagnostic::new(
                    arg.range(),
                    DiagnosticCode::ParameterImmutable,
                ));
            }
        } else {
            self.declare_implicit_variable(
                "it",
                args_range.start..args_range.start,
                false,
                BindType::ItParameter,
            );
        }
        self.declare_block(stmts, expr);

        let ret_reg = self.closures.add_reg();
        let never = self.emit_block(stmts, expr, ret_reg, None);
        if !never {
            self.op_return(ret_reg);
        }
        self.op(OpCode::FuncEnd);

        let nreg: OpParam = self.closures.current().reg_len().into();
        if ret.is_wide() || nreg.is_wide() || narg.is_wide() {
            self.chunk.code[pos] = OpCode::Func.wide_code();
            self.chunk.code.splice(
                pos + 1..pos + 1,
                [ret.wide_code(), narg.wide_code(), nreg.wide_code()].concat(),
            );
        } else if !nreg.is_wide() {
            self.chunk
                .code
                .splice(pos + 1..pos + 1, [ret.code(), narg.code(), nreg.code()]);
        }
        self.exit_scope();
        self.closures.exit();
    }
}

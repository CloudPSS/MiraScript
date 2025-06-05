use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic},
    lexer::Token,
    parser::{AstWalker, Expression, Statement},
};

use super::{
    Emitter, OpCode,
    closure::Closure,
    opcode::{OpParam, OpParamTrait, Register},
    variable::BindType,
};

impl<'s> Emitter<'s> {
    pub fn enter_closure(&mut self, late_binding: bool) {
        self.closures.push(Closure::new(late_binding));
    }
    pub fn exit_closure(&mut self) {
        self.closures.pop();
    }
    pub fn current_closure(&mut self) -> &mut Closure {
        self.closures.last_mut().unwrap()
    }
    pub fn add_reg(&mut self) -> Register {
        self.current_closure().add_reg()
    }
    pub fn emit_block(
        &mut self,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
        ret: Register,
        brk: Option<Register>,
    ) -> bool {
        for stmt in stmts {
            self.declare_statement(stmt);
        }
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
            self.enter_scope();
            self.declare_expression(expr);
            self.emit_expression(expr, ret, brk);
            self.exit_scope();
        } else if !has_never && !ret.is_empty() {
            self.op_nil(ret);
        };
        has_never
    }
    pub fn emit_fn(
        &mut self,
        ret: Register,
        id_or_fn: &'s Token<'s>,
        args: &'s Option<Vec<Token<'s>>>,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
    ) {
        self.enter_closure(true);
        self.enter_scope();

        let narg: OpParam = args.as_ref().map_or(1, |args| args.len()).into();
        let pos = self.chunk.code.len();
        self.chunk.code.push(OpCode::Func.code());

        if let Some(args) = args {
            for arg in args {
                if arg.is_identifier() {
                    self.declare_variable(arg, false, BindType::Parameter);
                } else {
                    self.declare_implicit_variable(
                        "<unnamed param>",
                        id_or_fn.range(),
                        false,
                        BindType::Parameter,
                    );
                }
                self.diagnostics.push(SourceDiagnostic::new(
                    arg.range(),
                    DiagnosticCode::ParameterImmutable,
                ));
            }
        } else {
            self.declare_implicit_variable("it", id_or_fn.range(), false, BindType::ItParameter);
        }

        let ret_reg = self.add_reg();
        let never = self.emit_block(stmts, expr, ret_reg, None);
        if !never {
            self.op_return(ret_reg);
        }
        self.op(OpCode::FuncEnd);

        let nreg: OpParam = self.current_closure().reg_len().into();
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
        self.exit_closure();
    }
}

use crate::{
    error::{ErrorCode, SourceError},
    lexer::{Token, TokenKind},
    parser::{AstWalker, Expression, Script, Statement},
};

use super::{
    Emitter, OpCode,
    closure::Closure,
    opcode::{OpParam, OpParamTrait, Register},
    scope::Scope,
    variable::{BindType, Variable},
};

impl<'s> Emitter<'s> {
    pub fn enter_closure(&mut self) {
        self.closures.push(Closure::new());
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
        brk: Register,
    ) -> bool {
        for stmt in stmts {
            if matches!(stmt, Statement::Function(..)) {
                self.declare_statement(stmt);
            }
        }
        for stmt in stmts {
            if !matches!(stmt, Statement::Function(..)) {
                self.declare_statement(stmt);
            }
        }
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
    pub fn emit_closure(
        &mut self,
        ret: Register,
        args: &'s Option<Vec<Token<'s>>>,
        stmts: &'s Vec<Statement<'s>>,
        expr: &'s Option<Box<Expression<'s>>>,
    ) {
        self.enter_closure();
        self.enter_scope();

        let narg: OpParam = args.as_ref().map_or(1, |args| args.len()).into();
        let wide = narg.is_wide() || ret.is_wide();
        let pos = self.chunk.code.len();
        if !wide {
            self.chunk.add_code(OpCode::Func);
            self.chunk.add_param(ret);
            self.chunk.add_param(narg);
            // Placeholder for nreg
            self.chunk.add_param(narg);
        } else {
            self.chunk.add_code_wide(OpCode::Func);
            self.chunk.add_param_wide(ret);
            self.chunk.add_param_wide(narg);
            // Placeholder for nreg
            self.chunk.add_param_wide(narg);
        }

        if let Some(args) = args {
            for arg in args {
                if arg.is_identifier() {
                    self.declare_variable(arg, false, BindType::Parameter);
                } else {
                    self.declare_implicit_variable("<unnamed param>", false, BindType::Parameter);
                }
            }
        } else {
            self.declare_implicit_variable("it", false, BindType::Parameter);
        }

        let ret_reg = self.add_reg();
        let never = self.emit_block(stmts, expr, ret_reg, Register::EMPTY);
        if !never {
            self.op_return(ret_reg);
        }
        self.op(OpCode::FuncEnd);

        let nreg: OpParam = self.current_closure().reg_len().into();
        if wide {
            self.chunk.code[pos + 1 + 4 + 4..pos + 1 + 4 + 4 + 4]
                .copy_from_slice(&nreg.wide_code());
        } else if !nreg.is_wide() {
            self.chunk.code[pos + 1 + 1 + 1] = nreg.code();
        } else {
            self.chunk.code[pos] = OpCode::Func.wide_code();
            self.chunk.code.splice(
                pos + 1..pos + 1 + 1 + 1 + 1,
                [ret.wide_code(), narg.wide_code(), nreg.wide_code()].concat(),
            );
        }

        self.exit_scope();
        self.exit_closure();
    }
}

use std::ops::{Deref, DerefMut};

use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    emitter::{closure::Closure, opcode::Register},
    lexer::{Token, TokenKind},
    parser::AstWalker,
};

use super::{
    Emitter,
    scope::Scope,
    variable::{BindType, Variable},
};

pub(super) struct Scopes<'s>(Vec<Scope<'s>>);

impl<'s> Scopes<'s> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn current(&mut self) -> &mut Scope<'s> {
        self.last_mut().unwrap()
    }

    pub fn find_variable(&mut self, name: &str) -> Option<(usize, &mut Variable<'s>)> {
        for scope in self.0.iter_mut().rev() {
            let level = scope.level;
            if let Some(var) = scope.find_variable(name) {
                return Some((level, var));
            }
        }
        None
    }

    pub fn find_local_variable(&mut self, name: &str) -> Option<&mut Variable<'s>> {
        self.current().find_variable(name)
    }
}

impl<'s> Deref for Scopes<'s> {
    type Target = Vec<Scope<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Scopes<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn check_variable_initialized<'s>(
    diagnostics: &mut Vec<SourceDiagnostic>,
    closures: &[Closure],
    access: &Token<'s>,
    variable: &Variable<'s>,
    level: usize,
) -> bool {
    if variable.initialized() {
        // 变量已初始化
        return true;
    }
    if level < closures.len() && closures[level..].iter().any(|c| c.late_binding()) {
        return true;
    }
    // 变量在前级闭包中定义，且中间没有延迟绑定的闭包
    //  或
    // 变量在当前或后级(见 for-in 的实现)闭包中定义
    diagnostics.push(SourceDiagnostic::new(
        access.range(),
        DiagnosticCode::UninitializedVariable,
    ));
    variable.put_decl_ref(diagnostics);
    false
}

impl<'s> Emitter<'s> {
    pub fn enter_scope(&mut self, range: SourceRange) {
        self.enter_leveled_scope(range, self.closures.len());
    }
    pub fn enter_leveled_scope(&mut self, range: SourceRange, level: usize) {
        self.scopes.push(Scope::new(level));
        self.diagnostics
            .push(SourceDiagnostic::new(range, DiagnosticCode::Scope));
    }
    pub fn exit_scope(&mut self) {
        let Some(scope) = self.scopes.pop() else {
            return;
        };
        for var in scope.variables {
            var.exit(&mut self.diagnostics);
        }
    }

    fn add_variable_reg(&mut self, bind_type: BindType) -> Register {
        match bind_type {
            BindType::Parameter => self.closures.current().add_arg(),
            BindType::RestParameter => self.closures.current().add_var_arg(),
            _ => Register::EMPTY,
        }
    }

    fn check_local_variable(&mut self, access: &Token<'s>, name: &str) -> Option<&Variable<'s>> {
        let var = self.scopes.find_local_variable(name)?;
        self.diagnostics.push(SourceDiagnostic::new(
            access.range(),
            DiagnosticCode::DuplicateVariableDeclaration,
        ));
        var.put_decl_ref(&mut self.diagnostics);
        self.diagnostics
            .push(SourceDiagnostic::new(access.range(), var.hint()));
        var.put_decl_ref(&mut self.diagnostics);
        Some(var)
    }

    pub fn declare_implicit_variable(
        &mut self,
        name: &'s str,
        range: SourceRange,
        mutable: bool,
        bind_type: BindType,
    ) {
        let register = self.add_variable_reg(bind_type);
        let var = Variable::new(name, range, mutable, bind_type, register);
        self.scopes.current().declare_variable(var);
    }
    pub fn declare_variable(
        &mut self,
        id_token: &'s Token<'s>,
        mutable: bool,
        bind_type: BindType,
    ) -> bool {
        let TokenKind::Identifier(id) = &id_token.kind else {
            return false;
        };
        if self.check_local_variable(id_token, id).is_some() {
            return false;
        }
        let register = self.add_variable_reg(bind_type);
        let var = Variable::new(id, id_token.range(), mutable, bind_type, register);
        self.scopes.current().declare_variable(var);
        true
    }
}

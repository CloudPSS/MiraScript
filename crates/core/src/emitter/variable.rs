use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    lexer::Token,
    parser::AstWalker,
};

use super::opcode::Register;

/// Represents the type of binding for a variable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BindType {
    /// The variable is bound by `let` statement.
    Let,
    /// The variable is bound as a pattern in `for`, `while`, `match` or `is` expressions.
    Init,
    /// The variable is bound as a function's normal parameter.
    Parameter,
    /// The variable is bound as a function's auto `it` parameter.
    ItParameter,
    /// The variable is bound as a function's rest parameter.
    RestParameter,
    /// The variable is bound as a function statement.
    Func,
}

pub(crate) struct Variable<'s> {
    name: &'s str,
    declaration: SourceRange,
    mutable: bool,
    bind_type: BindType,
    register: Register,
    initialized: bool,
    used: bool,
}

impl<'s> Variable<'s> {
    pub fn new(
        name: &'s str,
        declaration: SourceRange,
        mutable: bool,
        bind_type: BindType,
        register: Register,
    ) -> Self {
        Self {
            name,
            declaration,
            mutable,
            bind_type,
            register,
            initialized: false,
            used: false,
        }
    }

    pub fn initialized(&self) -> bool {
        self.initialized
    }

    pub fn name(&self) -> &'s str {
        self.name
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn bind_type(&self) -> BindType {
        self.bind_type
    }

    pub fn register(&self) -> Register {
        if self.register.is_empty() {
            panic!("Variable {} has no register assigned", self.name);
        }
        self.register
    }

    pub fn has_register(&self) -> bool {
        !self.register.is_empty()
    }

    pub fn set_register(&mut self, register: Register) {
        self.register = register;
    }

    pub fn hint(&self) -> DiagnosticCode {
        if matches!(self.bind_type, BindType::ItParameter) {
            return if self.used {
                DiagnosticCode::ParameterIt
            } else {
                DiagnosticCode::UnusedParameterIt
            };
        }
        if !self.mutable {
            match self.bind_type {
                BindType::Let => DiagnosticCode::LocalImmutable,
                BindType::Init => DiagnosticCode::LocalImmutable,
                BindType::Func => DiagnosticCode::LocalFunction,
                BindType::Parameter => DiagnosticCode::ParameterImmutable,
                BindType::RestParameter => DiagnosticCode::ParameterImmutableRest,
                BindType::ItParameter => unreachable!(),
            }
        } else {
            match self.bind_type {
                BindType::Let => DiagnosticCode::LocalMutable,
                BindType::Init => DiagnosticCode::LocalMutable,
                BindType::Func => DiagnosticCode::LocalFunction,
                BindType::Parameter => DiagnosticCode::ParameterMutable,
                BindType::RestParameter => DiagnosticCode::ParameterMutableRest,
                BindType::ItParameter => unreachable!(),
            }
        }
    }

    pub fn put_decl_ref(&self, diagnostics: &mut Vec<SourceDiagnostic>) {
        let code = match self.bind_type {
            BindType::Parameter => DiagnosticCode::ParameterDeclaredHere,
            BindType::RestParameter => DiagnosticCode::ParameterRestDeclaredHere,
            BindType::ItParameter => DiagnosticCode::ParameterItDeclaredHere,
            BindType::Func => DiagnosticCode::FunctionDeclaredHere,
            _ => DiagnosticCode::VariableDeclaredHere,
        };
        diagnostics.push(SourceDiagnostic::new(self.declaration.clone(), code));
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn mark_read(&mut self, token: &Token<'_>, diagnostics: &mut Vec<SourceDiagnostic>) {
        self.used = true;
        diagnostics.push(SourceDiagnostic::new(token.range(), self.hint()));
        self.put_decl_ref(diagnostics);
    }

    pub fn mark_write(&mut self, token: &Token<'_>, diagnostics: &mut Vec<SourceDiagnostic>) {
        diagnostics.push(SourceDiagnostic::new(token.range(), self.hint()));
        self.put_decl_ref(diagnostics);
    }

    pub fn exit(self, diagnostics: &mut Vec<SourceDiagnostic>) {
        diagnostics.push(SourceDiagnostic::new(self.declaration.clone(), self.hint()));
        if self.used || matches!(self.bind_type, BindType::ItParameter) {
            return;
        }
        diagnostics.push(SourceDiagnostic::new(
            self.declaration,
            if self.bind_type == BindType::Func {
                DiagnosticCode::UnusedLocalFunction
            } else {
                DiagnosticCode::UnusedLocalVariable
            },
        ));
    }
}

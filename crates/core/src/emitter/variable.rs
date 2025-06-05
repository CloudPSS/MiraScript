use crate::diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};

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
            initialized: matches!(
                bind_type,
                BindType::Parameter | BindType::RestParameter | BindType::ItParameter
            ),
            used: false,
        }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
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
        self.register
    }

    pub fn hint(&self) -> DiagnosticCode {
        if !self.mutable {
            match self.bind_type {
                BindType::Let => DiagnosticCode::LocalImmutable,
                BindType::Init => DiagnosticCode::LocalImmutable,
                BindType::Func => DiagnosticCode::LocalFunction,
                BindType::Parameter => DiagnosticCode::ParameterImmutable,
                BindType::ItParameter => DiagnosticCode::ParameterImmutableIt,
                BindType::RestParameter => DiagnosticCode::ParameterImmutableRest,
            }
        } else {
            match self.bind_type {
                BindType::Let => DiagnosticCode::LocalMutable,
                BindType::Init => DiagnosticCode::LocalMutable,
                BindType::Func => DiagnosticCode::LocalFunction,
                BindType::Parameter => DiagnosticCode::ParameterMutable,
                BindType::ItParameter => DiagnosticCode::ParameterMutableIt,
                BindType::RestParameter => DiagnosticCode::ParameterMutableRest,
            }
        }
    }

    pub fn mark_used(&mut self) {
        self.used = true;
    }

    pub fn put_declaration(&self, diagnostics: &mut Vec<SourceDiagnostic>) {
        diagnostics.push(SourceDiagnostic::new(
            self.declaration.clone(),
            match self.bind_type {
                BindType::Parameter => DiagnosticCode::ParameterDeclaredHere,
                BindType::RestParameter => DiagnosticCode::ParameterRestDeclaredHere,
                BindType::ItParameter => DiagnosticCode::ParameterItDeclaredHere,
                _ => DiagnosticCode::VariableDeclaredHere,
            },
        ));
    }

    pub fn exit(self, diagnostics: &mut Vec<SourceDiagnostic>) {
        if self.used {
            return;
        }
        diagnostics.push(SourceDiagnostic::new(
            self.declaration,
            if self.bind_type == BindType::Func {
                DiagnosticCode::LocalUnusedFunction
            } else {
                DiagnosticCode::LocalUnusedVariable
            },
        ));
    }
}

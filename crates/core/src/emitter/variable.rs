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
        declaration: Option<SourceRange>,
        mutable: bool,
        bind_type: BindType,
        register: Register,
    ) -> Self {
        Self {
            name,
            declaration: declaration.unwrap_or_default(),
            mutable,
            bind_type,
            register,
            initialized: matches!(bind_type, BindType::Parameter | BindType::RestParameter),
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
                BindType::RestParameter => DiagnosticCode::ParameterImmutableRest,
            }
        } else {
            match self.bind_type {
                BindType::Let => DiagnosticCode::LocalMutable,
                BindType::Init => DiagnosticCode::LocalMutable,
                BindType::Func => DiagnosticCode::LocalFunction,
                BindType::Parameter => DiagnosticCode::ParameterMutable,
                BindType::RestParameter => DiagnosticCode::ParameterMutableRest,
            }
        }
    }

    pub fn mark_used(&mut self) {
        self.used = true;
    }

    pub fn declaration(&self) -> Option<SourceRange> {
        if self.declaration == SourceRange::default() {
            None
        } else {
            Some(self.declaration.clone())
        }
    }

    pub fn exit(self) -> Option<SourceDiagnostic> {
        if self.used || self.declaration == SourceRange::default() {
            None
        } else {
            Some(SourceDiagnostic::new(
                self.declaration,
                if self.bind_type == BindType::Func {
                    DiagnosticCode::LocalUnusedFunction
                } else {
                    DiagnosticCode::LocalUnusedVariable
                },
            ))
        }
    }
}

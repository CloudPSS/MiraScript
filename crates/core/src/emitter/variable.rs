#[cfg(feature = "track_references")]
use crate::{config::track_references, parser::AstWalker as _};
use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    lexer::Token,
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
    /// The variable is bound as a function's parameter that matches a pattern.
    PatternParameter,
    /// The variable is bound as a function's rest parameter that matches a pattern.
    RestPatternParameter,
    /// The variable is bound as a variable in a pattern of a function's parameter.
    ParameterSubPattern,
    /// The variable is bound as a function statement.
    Func,
}

pub(crate) struct Variable<'s> {
    name: &'s str,
    declaration: SourceRange,
    mutable: bool,
    bind_type: BindType,
    register: Register,
    #[cfg(feature = "track_references")]
    reference: Vec<SourceDiagnostic>,
    initialized: bool,
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
            #[cfg(feature = "track_references")]
            reference: Vec::new(),
            initialized: matches!(bind_type, |BindType::Parameter| BindType::RestParameter
                | BindType::ItParameter),
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

    pub fn put_decl_ref(&self, diagnostics: &mut Vec<SourceDiagnostic>) {
        let code = match self.bind_type {
            BindType::Parameter | BindType::PatternParameter => {
                DiagnosticCode::ParameterDeclaredHere
            }
            BindType::RestParameter | BindType::RestPatternParameter => {
                DiagnosticCode::ParameterRestDeclaredHere
            }
            BindType::ParameterSubPattern => DiagnosticCode::ParameterSubPatternDeclaredHere,
            BindType::ItParameter => DiagnosticCode::ParameterItDeclaredHere,
            BindType::Func => DiagnosticCode::FunctionDeclaredHere,
            _ => DiagnosticCode::VariableDeclaredHere,
        };
        diagnostics.push(SourceDiagnostic::new(self.declaration.clone(), code));
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn mark_read(&mut self, token: &Token<'_>) {
        #[cfg(feature = "track_references")]
        if track_references() {
            self.reference.push(SourceDiagnostic::new(
                token.range(),
                DiagnosticCode::ReadLocal,
            ));
        }
    }

    pub fn mark_write(&mut self, token: &Token<'_>) {
        #[cfg(feature = "track_references")]
        if track_references() {
            self.reference.push(SourceDiagnostic::new(
                token.range(),
                DiagnosticCode::WriteLocal,
            ));
        }
    }

    pub fn mark_read_write(&mut self, token: &Token<'_>) {
        #[cfg(feature = "track_references")]
        if track_references() {
            self.reference.push(SourceDiagnostic::new(
                token.range(),
                DiagnosticCode::ReadWriteLocal,
            ));
        }
    }

    pub fn mark_redeclare(&mut self, token: &Token<'_>) {
        #[cfg(feature = "track_references")]
        if track_references() {
            self.reference.push(SourceDiagnostic::new(
                token.range(),
                DiagnosticCode::RedeclareLocal,
            ));
        }
    }

    pub fn exit(self, diagnostics: &mut Vec<SourceDiagnostic>) {
        #[cfg(feature = "track_references")]
        if track_references() {
            let hint = if !self.mutable {
                match self.bind_type {
                    BindType::Let => DiagnosticCode::LocalImmutable,
                    BindType::Init => DiagnosticCode::LocalImmutable,
                    BindType::Func => DiagnosticCode::LocalFunction,
                    BindType::Parameter => DiagnosticCode::ParameterImmutable,
                    BindType::RestParameter => DiagnosticCode::ParameterImmutableRest,
                    BindType::ItParameter => DiagnosticCode::ParameterIt,
                    BindType::PatternParameter => DiagnosticCode::ParameterPattern,
                    BindType::RestPatternParameter => DiagnosticCode::ParameterRestPattern,
                    BindType::ParameterSubPattern => DiagnosticCode::ParameterSubPatternImmutable,
                }
            } else {
                match self.bind_type {
                    BindType::Let => DiagnosticCode::LocalMutable,
                    BindType::Init => DiagnosticCode::LocalMutable,
                    BindType::Func => DiagnosticCode::LocalFunction,
                    BindType::Parameter => DiagnosticCode::ParameterMutable,
                    BindType::RestParameter => DiagnosticCode::ParameterMutableRest,
                    BindType::ItParameter => DiagnosticCode::ParameterIt,
                    BindType::PatternParameter => DiagnosticCode::ParameterPattern,
                    BindType::RestPatternParameter => DiagnosticCode::ParameterRestPattern,
                    BindType::ParameterSubPattern => DiagnosticCode::ParameterSubPatternMutable,
                }
            };
            let mut used = false;
            diagnostics.push(SourceDiagnostic::new(self.declaration.clone(), hint));
            for reference in self.reference {
                if *reference == DiagnosticCode::ReadLocal {
                    used = true;
                }
                diagnostics.push(reference);
            }
            if !used
                && !matches!(
                    self.bind_type,
                    BindType::ItParameter
                        | BindType::PatternParameter
                        | BindType::RestPatternParameter
                )
            {
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
    }
}

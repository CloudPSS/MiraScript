use crate::{
    diagnostic::{DiagnosticCode, DiagnosticsCollector, SourceDiagnostic, SourceRange},
    lexer::Token,
    parser::AstWalker as _,
};

use super::opcode::Register;

/// Represents the type of binding for a variable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BindType {
    /// The variable is bound by `let` statement.
    Let,
    /// The variable is bound by `let` statement with an function expression.
    LetFunc,
    /// The variable is bound by `const` statement.
    Const,
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
    /// The variable is bound by `mod` statement.
    Module,
}

pub(crate) struct Variable<'s> {
    name: &'s str,
    declaration: SourceRange,
    mutable: bool,
    bind_type: BindType,
    register: Register,
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

    // pub fn bind_type(&self) -> BindType {
    //     self.bind_type
    // }

    pub fn register(&self) -> Register {
        self.register
    }

    pub fn has_register(&self) -> bool {
        !self.register.is_empty()
    }

    pub fn set_register(&mut self, register: Register) {
        self.register = register;
    }

    pub fn put_decl_ref(&self, diagnostics: &mut DiagnosticsCollector<'_, '_>) {
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
        diagnostics.push(code, self.declaration.clone());
    }

    #[inline]
    pub fn initialize(&mut self) {
        debug_assert!(
            !self.register.is_empty(),
            "Variable {} has no register assigned",
            self.name
        );
        self.initialized = true;
    }

    #[inline]
    pub fn mark_read(&mut self, token: &Token<'_>) {
        self.reference.push(SourceDiagnostic::new(
            token.range(),
            DiagnosticCode::ReadLocal,
        ));
    }

    #[inline]
    pub fn mark_write(&mut self, token: &Token<'_>) {
        self.reference.push(SourceDiagnostic::new(
            token.range(),
            DiagnosticCode::WriteLocal,
        ));
    }

    #[inline]
    pub fn mark_read_write(&mut self, token: &Token<'_>) {
        self.reference.push(SourceDiagnostic::new(
            token.range(),
            DiagnosticCode::ReadWriteLocal,
        ));
    }

    #[inline]
    pub fn mark_redeclare(&mut self, token: &Token<'_>) {
        self.reference.push(SourceDiagnostic::new(
            token.range(),
            DiagnosticCode::RedeclareLocal,
        ));
    }

    #[inline]
    pub fn mark_exported(&mut self, module_id_token: &Token<'_>) {
        self.reference.push(SourceDiagnostic::new(
            module_id_token.range(),
            DiagnosticCode::ExportedLocal,
        ));
    }

    #[inline]
    pub fn exit(self, #[allow(unused)] diagnostics: &mut DiagnosticsCollector<'_, '_>) {
        let hint = if !self.mutable {
            match self.bind_type {
                BindType::Const => DiagnosticCode::LocalConst,
                BindType::Let | BindType::Init => DiagnosticCode::LocalImmutable,
                BindType::Func | BindType::LetFunc => DiagnosticCode::LocalFunction,
                BindType::Module => DiagnosticCode::LocalModule,
                BindType::Parameter => DiagnosticCode::ParameterImmutable,
                BindType::RestParameter => DiagnosticCode::ParameterImmutableRest,
                BindType::ItParameter => DiagnosticCode::ParameterIt,
                BindType::PatternParameter => DiagnosticCode::ParameterPattern,
                BindType::RestPatternParameter => DiagnosticCode::ParameterRestPattern,
                BindType::ParameterSubPattern => DiagnosticCode::ParameterSubPatternImmutable,
            }
        } else {
            match self.bind_type {
                BindType::Const => DiagnosticCode::LocalConst,
                BindType::Let | BindType::Init => DiagnosticCode::LocalMutable,
                BindType::Func | BindType::LetFunc => DiagnosticCode::LocalFunction,
                BindType::Module => DiagnosticCode::LocalModule,
                BindType::Parameter => DiagnosticCode::ParameterMutable,
                BindType::RestParameter => DiagnosticCode::ParameterMutableRest,
                BindType::ItParameter => DiagnosticCode::ParameterIt,
                BindType::PatternParameter => DiagnosticCode::ParameterPattern,
                BindType::RestPatternParameter => DiagnosticCode::ParameterRestPattern,
                BindType::ParameterSubPattern => DiagnosticCode::ParameterSubPatternMutable,
            }
        };
        let mut used = false;
        diagnostics.push(hint, self.declaration.clone());
        for reference in self.reference.iter() {
            if matches!(
                **reference,
                DiagnosticCode::ReadLocal | DiagnosticCode::ExportedLocal
            ) {
                used = true;
            }
        }
        diagnostics.extend(self.reference);
        if !used
            && !matches!(
                self.bind_type,
                BindType::ItParameter | BindType::PatternParameter | BindType::RestPatternParameter
            )
        {
            diagnostics.push(
                if matches!(self.bind_type, BindType::Func | BindType::LetFunc) {
                    DiagnosticCode::UnusedLocalFunction
                } else {
                    DiagnosticCode::UnusedLocalVariable
                },
                self.declaration,
            );
        }
    }
}

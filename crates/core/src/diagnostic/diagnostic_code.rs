use std::fmt::Display;
use strum::{EnumMessage, FromRepr};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumMessage, FromRepr)]
#[repr(u16)]
pub enum DiagnosticCode {
    // Preserved 0~999

    // Error 1000~1999
    ErrorStart = 1000,

    #[strum(message = "Unknown internal error")]
    InternalError,
    #[strum(message = "Unknown lexer error")]
    LexerError,
    #[strum(message = "Unknown parser error")]
    ParserError,
    #[strum(message = "Unknown emitter error")]
    EmitterError,
    #[strum(message = "Unknown optimizer error")]
    OptimizerError,

    #[strum(message = "Unknown token")]
    UnknownToken,
    #[strum(message = "Number literal cannot start or end with underscore")]
    InvalidNumberLiteralUnderscore,
    #[strum(message = "Invalid number literal")]
    InvalidNumberLiteral,
    #[strum(message = "Unterminated string literal")]
    UnterminatedString,
    #[strum(message = "Invalid escape sequence")]
    InvalidEscapeSequence,
    #[strum(message = "Expression expected after `..`")]
    BadArraySpread,
    #[strum(message = "Unterminated interpolation expression")]
    UnterminatedInterpolation,
    #[strum(message = "Bad interpolation expression")]
    BadInterpolation,
    #[strum(message = "Unexpected `_`, it is a reserved keyword for discarding")]
    UnexpectedUnderscore,
    #[strum(message = "Unexpected `global`, it is a reserved keyword for global variable")]
    UnexpectedGlobal,
    #[strum(message = "Missing `,`")]
    MissingComma,
    #[strum(message = "Missing `]`")]
    MissingCloseBracket,
    #[strum(message = "Missing `{`")]
    MissingOpenBrace,
    #[strum(message = "Missing `}`")]
    MissingCloseBrace,
    #[strum(message = "Missing `)`")]
    MissingCloseParen,
    #[strum(message = "Missing `;`")]
    MissingSemicolon,
    #[strum(message = "Operator `=` expected in a bind statement")]
    MissingBindOperator,
    #[strum(message = "Missing name in function declaration")]
    MissingFunctionName,
    #[strum(message = "`type` is a function-like keyword, add `(` here")]
    MissingOpenParenAfterType,
    #[strum(message = "`type` call must have exactly one argument")]
    InvalidTypeCall,
    #[strum(message = "Unexpected record literal, grouping expression is expected")]
    RecordLiteralInExtensionCaller,
    #[strum(message = "Missing `case`")]
    MissingCase,
    #[strum(message = "Unknown expression")]
    UnknownExpression,
    #[strum(message = "Unknown pattern")]
    UnknownPattern,
    #[strum(message = "Unknown statement")]
    UnknownStatement,
    #[strum(message = "Pattern expected")]
    PatternExpected,
    #[strum(message = "Operator `!` is not allowed in pattern")]
    ExclamationInConstantsPattern,
    #[strum(message = "Unexpected operator in pattern, only number is allowed")]
    UnexpectedOperatorInConstantsPattern,
    #[strum(message = "`mut` is not allowed while rebinding")]
    MutInBindPattern,
    #[strum(message = "Cannot use `mut` in discard pattern")]
    MutInDiscardPattern,
    #[strum(message = "Discard pattern should be omitted in spread pattern")]
    DiscardInSpreadPattern,
    #[strum(message = "Interpolated name is not allowed in record pattern")]
    InterpolatedNameRecordPattern,
    #[strum(message = "Must be bind pattern while record field name omitted")]
    BadOmitKeyRecordPattern,
    #[strum(message = "Range pattern in array pattern should be parenthesised")]
    AmbiguousRangePattern,
    #[strum(message = "Cannot assign to an undeclared variable")]
    UndefinedVariableAssignment,
    #[strum(message = "Cannot assign to an immutable variable …")]
    ImmutableVariableAssignment,
    #[strum(message = "Cannot access a variable before it is …")]
    UninitializedVariable,
    #[strum(message = "The variable is already …")]
    DuplicateVariableDeclaration,
    #[strum(message = "Unexpected `break` outside of loop")]
    UnexpectedBreakOutsideLoop,
    #[strum(message = "Unexpected `continue` outside of loop")]
    UnexpectedContinueOutsideLoop,
    #[strum(
        message = "`global` keyword can only be used as `global.<name>`, `global[<name>]` or right-hand side of `in` operator"
    )]
    MisuseOfGlobalKeyword,
    #[strum(message = "Can not infer key from expression")]
    BadOmitKeyRecordExpression,

    ErrorEnd = 1999,
    // Warning 2000~2999
    WarningStart = 2000,

    #[strum(message = "Local variable is unused, consider removing it, or use `_` to ignore it")]
    LocalUnusedVariable,
    #[strum(message = "Local function is unused, consider removing it")]
    LocalUnusedFunction,

    WarningEnd = 2999,
    // Info 3000~3999
    InfoStart = 3000,

    InfoEnd = 3999,
    // Hint 4000~4999
    HintStart = 4000,

    HintEnd = 4999,
    // Reference 5000~5999
    ReferenceStart = 5000,

    #[strum(message = "… declared here")]
    VariableDeclaredHere,
    #[strum(message = "… declared as a parameter here")]
    ParameterDeclaredHere,
    #[strum(message = "… declared as the auto parameter `it` by this function here")]
    ParameterItDeclaredHere,
    #[strum(message = "… declared as a rest parameter here")]
    ParameterRestDeclaredHere,

    ReferenceEnd = 5999,

    // Tags 10000~
    GlobalVariable = 10000,
    GlobalDynamicAccess,

    LocalImmutable,
    LocalMutable,
    LocalFunction,

    ParameterImmutable,
    ParameterMutable,
    ParameterImmutableIt,
    ParameterMutableIt,
    ParameterImmutableRest,
    ParameterMutableRest,

    RecordFieldIdName,
    RecordFieldOrdinalName,
    RecordFieldStringName,
    OmittedFunctionArgument,
    UnnamedRecordField0,
    UnnamedRecordField1,
    UnnamedRecordField2,
    UnnamedRecordField3,
    UnnamedRecordField4,
    UnnamedRecordField5,
    UnnamedRecordField6,
    UnnamedRecordField7,
    UnnamedRecordField8,
    UnnamedRecordField9,
    UnnamedRecordFieldN,
    OmitNamedRecordField,
    OmitNamedRecordFieldName,
}

impl From<DiagnosticCode> for u16 {
    fn from(val: DiagnosticCode) -> Self {
        val.code()
    }
}

impl TryInto<DiagnosticCode> for u16 {
    type Error = ();

    fn try_into(self) -> Result<DiagnosticCode, Self::Error> {
        DiagnosticCode::from_repr(self).ok_or(())
    }
}

impl DiagnosticCode {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn from_code(code: u16) -> Option<Self> {
        DiagnosticCode::from_repr(code)
    }

    pub fn message(&self) -> &'static str {
        self.get_message().unwrap_or("Unknown error")
    }

    pub fn level_str(&self) -> &'static str {
        if self.is_error() {
            "Error"
        } else if self.is_warning() {
            "Warning"
        } else if self.is_info() {
            "Info"
        } else if self.is_hint() {
            "Hint"
        } else if self.is_reference() {
            "Reference"
        } else {
            "Unknown"
        }
    }

    pub fn is_error(&self) -> bool {
        *self > DiagnosticCode::ErrorStart && *self < DiagnosticCode::ErrorEnd
    }

    pub fn is_warning(&self) -> bool {
        *self >= DiagnosticCode::WarningStart && *self < DiagnosticCode::WarningEnd
    }

    pub fn is_info(&self) -> bool {
        *self >= DiagnosticCode::InfoStart && *self < DiagnosticCode::InfoEnd
    }

    pub fn is_hint(&self) -> bool {
        *self >= DiagnosticCode::HintStart && *self < DiagnosticCode::HintEnd
    }

    pub fn is_reference(&self) -> bool {
        *self >= DiagnosticCode::ReferenceStart && *self < DiagnosticCode::ReferenceEnd
    }

    pub fn is_other(&self) -> bool {
        *self > DiagnosticCode::ReferenceEnd
    }
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

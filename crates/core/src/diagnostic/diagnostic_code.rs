use std::fmt::Display;
use strum::{EnumMessage, FromRepr};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumMessage, FromRepr)]
#[repr(u16)]
pub enum DiagnosticCode {
    // Internal error 1 ~ 999
    ErrorStart = 0,
    #[strum(message = "Unknown internal error")]
    InternalError = 1,

    // Lexer error 1000 ~ 1999
    #[strum(message = "Unknown lexer error")]
    LexerError = 1000,
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

    // Parser error 2000 ~ 2999
    #[strum(message = "Unknown parser error")]
    ParserError = 2000,
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

    // Emitter error 3000 ~ 3999
    #[strum(message = "Unknown emitter error")]
    EmitterError = 3000,
    #[strum(message = "Cannot assign to an undeclared variable")]
    UndefinedVariableAssignment,
    #[strum(message = "Cannot assign to an immutable variable")]
    ImmutableVariableAssignment,
    #[strum(message = "Cannot access a variable before it is declared")]
    UninitializedVariable,
    #[strum(message = "The variable is already declared")]
    DuplicateVariableDeclaration,
    #[strum(message = "The variable is already declared as a parameter")]
    DuplicateParameterDeclaration,
    #[strum(message = "Unexpected `break` outside of loop")]
    UnexpectedBreakOutsideLoop,
    #[strum(message = "Unexpected `continue` outside of loop")]
    UnexpectedContinueOutsideLoop,
    #[strum(message = "`global` keyword can only be used as `global.<name>` or `global[<name>]`")]
    MisuseOfGlobalKeyword,
    #[strum(message = "Can not infer key from expression")]
    BadOmitKeyRecordExpression,

    // Optimizer error 4000 ~ 4999
    #[strum(message = "Unknown optimizer error")]
    OptimizerError = 4000,

    ErrorEnd = 9999,
    WarningStart = 10000,

    // Lexer warning 11000 ~ 11999
    #[strum(message = "Unknown lexer warning")]
    LexerWarning = 11000,

    // Parser warning 12000 ~ 12999
    #[strum(message = "Unknown parser warning")]
    ParserWarning = 12000,

    // Emitter warning 13000 ~ 13999
    #[strum(message = "Unknown emitter warning")]
    EmitterWarning = 13000,

    // Optimizer warning 14000 ~ 14999
    #[strum(message = "Unknown optimizer warning")]
    OptimizerWarning = 14000,

    WarningEnd = 19999,
    InfoStart = 20000,

    // Lexer info 21000 ~ 21999
    #[strum(message = "Unknown lexer info")]
    LexerInfo = 21000,

    // Parser info 22000 ~ 22999
    #[strum(message = "Unknown parser info")]
    ParserInfo = 22000,

    // Emitter info 23000 ~ 23999
    #[strum(message = "Unknown emitter info")]
    EmitterInfo = 23000,

    // Optimizer info 24000 ~ 24999
    #[strum(message = "Unknown optimizer info")]
    OptimizerInfo = 24000,

    InfoEnd = 29999,
    HintStart = 30000,

    // Lexer hint 31000 ~ 31999
    #[strum(message = "Unknown lexer hint")]
    LexerHint = 31000,

    // Parser hint 32000 ~ 32999
    #[strum(message = "Unknown parser hint")]
    ParserHint = 32000,

    // Emitter hint 33000 ~ 33999
    #[strum(message = "Unknown emitter hint")]
    EmitterHint = 33000,

    // Optimizer hint 34000 ~ 34999
    #[strum(message = "Unknown optimizer hint")]
    OptimizerHint = 34000,

    HintEnd = 39999,
    ReferenceStart = 40000,

    // Lexer reference 41000 ~ 41999
    #[strum(message = "Unknown lexer reference")]
    LexerReference = 41000,

    // Parser reference 42000 ~ 42999
    #[strum(message = "Unknown parser reference")]
    ParserReference = 42000,

    // Emitter reference 43000 ~ 43999
    #[strum(message = "Unknown emitter reference")]
    EmitterReference = 43000,

    // Optimizer reference 44000 ~ 44999
    #[strum(message = "Unknown optimizer reference")]
    OptimizerReference = 44000,

    ReferenceEnd = 49999,

    // Tags 50000 ~ 50999
    GlobalVariable = 50000,
    LocalImmutable,
    LocalVariable,
    LocalFunction,
    OmittedFunctionArgument,
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
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

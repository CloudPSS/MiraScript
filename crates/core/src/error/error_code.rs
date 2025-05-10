use std::fmt::Display;

use strum::{EnumMessage, FromRepr};

#[derive(Debug, Clone, Copy, PartialEq, EnumMessage, FromRepr)]
#[repr(u16)]
pub enum ErrorCode {
    // Internal error 1 ~ 999
    #[strum(message = "Unknown internal error")]
    InternalError = 0,

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
    #[strum(message = "Can not infer key from expression")]
    BadOmitKeyRecordExpression,
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
    #[strum(message = "Must be bind pattern while record field name omitted")]
    BadOmitKeyRecordPattern,
    #[strum(message = "Range pattern in array pattern should be parenthesised")]
    AmbiguousRangePattern,

    // Analyzer error 3000 ~ 3999
    #[strum(message = "Unknown analyzer error")]
    AnalyzerError = 3000,

    // Emitter error 4000 ~ 4999
    #[strum(message = "Unknown emitter error")]
    EmitterError = 4000,

    // Optimizer error 5000 ~ 5999
    #[strum(message = "Unknown optimizer error")]
    OptimizerError = 5000,

    // Lexer warning 11000 ~ 11999
    #[strum(message = "Unknown lexer warning")]
    LexerWarning = 11000,

    // Parser warning 12000 ~ 12999
    #[strum(message = "Unknown parser warning")]
    ParserWarning = 12000,

    // Analyzer warning 13000 ~ 13999
    #[strum(message = "Unknown analyzer warning")]
    AnalyzerWarning = 13000,

    // Emitter warning 14000 ~ 14999
    #[strum(message = "Unknown emitter warning")]
    EmitterWarning = 14000,

    // Optimizer warning 15000 ~ 15999
    #[strum(message = "Unknown optimizer warning")]
    OptimizerWarning = 15000,

    // Lexer info 21000 ~ 21999
    #[strum(message = "Unknown lexer info")]
    LexerInfo = 21000,

    // Parser info 22000 ~ 22999
    #[strum(message = "Unknown parser info")]
    ParserInfo = 22000,

    // Analyzer info 23000 ~ 23999
    #[strum(message = "Unknown analyzer info")]
    AnalyzerInfo = 23000,

    // Emitter info 24000 ~ 24999
    #[strum(message = "Unknown emitter info")]
    EmitterInfo = 24000,

    // Optimizer info 25000 ~ 25999
    #[strum(message = "Unknown optimizer info")]
    OptimizerInfo = 25000,

    // Lexer hint 31000 ~ 31999
    #[strum(message = "Unknown lexer hint")]
    LexerHint = 31000,

    // Parser hint 32000 ~ 32999
    #[strum(message = "Unknown parser hint")]
    ParserHint = 32000,

    // Analyzer hint 33000 ~ 33999
    #[strum(message = "Unknown analyzer hint")]
    AnalyzerHint = 33000,

    // Emitter hint 34000 ~ 34999
    #[strum(message = "Unknown emitter hint")]
    EmitterHint = 34000,

    // Optimizer hint 35000 ~ 35999
    #[strum(message = "Unknown optimizer hint")]
    OptimizerHint = 35000,

    // Lexer reference 41000 ~ 41999
    #[strum(message = "Unknown lexer reference")]
    LexerReference = 41000,

    // Parser reference 42000 ~ 42999
    #[strum(message = "Unknown parser reference")]
    ParserReference = 42000,

    // Analyzer reference 43000 ~ 43999
    #[strum(message = "Unknown analyzer reference")]
    AnalyzerReference = 43000,

    // Emitter reference 44000 ~ 44999
    #[strum(message = "Unknown emitter reference")]
    EmitterReference = 44000,

    // Optimizer reference 45000 ~ 45999
    #[strum(message = "Unknown optimizer reference")]
    OptimizerReference = 45000,
}

impl From<ErrorCode> for u16 {
    fn from(val: ErrorCode) -> Self {
        val.code()
    }
}

impl TryInto<ErrorCode> for u16 {
    type Error = ();

    fn try_into(self) -> Result<ErrorCode, Self::Error> {
        ErrorCode::from_repr(self).ok_or(())
    }
}

impl ErrorCode {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn from_code(code: u16) -> Option<Self> {
        ErrorCode::from_repr(code)
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
        self.code() < 10000
    }

    pub fn is_warning(&self) -> bool {
        self.code() >= 10000 && self.code() < 20000
    }

    pub fn is_info(&self) -> bool {
        self.code() >= 20000 && self.code() < 30000
    }

    pub fn is_hint(&self) -> bool {
        self.code() >= 30000 && self.code() < 40000
    }

    pub fn is_reference(&self) -> bool {
        self.code() >= 40000 && self.code() < 50000
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

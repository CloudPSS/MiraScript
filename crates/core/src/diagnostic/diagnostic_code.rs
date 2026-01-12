use strum::{Display, EnumMessage, FromRepr, IntoStaticStr};

/// Diagnostic codes for MiraScript compiler and tools.
#[cfg_attr(feature = "wasm-constants", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, EnumMessage, FromRepr, Display, IntoStaticStr,
)]
#[repr(u16)]
pub enum DiagnosticCode {
    // Preserved 0~999

    // Error 1000~1999
    ErrorStart = 1000,

    #[strum(message = "Unknown internal error occurred")]
    InternalError,
    #[strum(message = "Unknown lexer error occurred")]
    LexerError,
    #[strum(message = "Unknown parser error occurred")]
    ParserError,
    #[strum(message = "Unknown emitter error occurred")]
    EmitterError,
    #[strum(message = "Unknown optimizer error occurred")]
    OptimizerError,
    #[strum(message = "This feature is not implemented yet")]
    Unimplemented,

    #[strum(message = "Unknown token encountered")]
    UnknownToken,
    #[strum(message = "Unexpected token found")]
    UnexpectedToken,
    #[strum(message = "`$0` is a reserved keyword and cannot be used as an identifier")]
    InvalidReservedKeyword,
    #[strum(message = "`$0` is a keyword and cannot be used as an identifier")]
    InvalidKeyword,
    #[strum(message = "A number literal cannot start or end with an underscore")]
    InvalidNumberLiteralUnderscore,
    #[strum(message = "Invalid number literal")]
    InvalidNumberLiteral,
    #[strum(message = "Number literal is too large")]
    OverflowNumberLiteral,
    #[strum(message = "Integer literal is too large")]
    OverflowIntegerLiteral,
    #[strum(
        message = "Invalid ordinal literal; consider remove leading zeros and underscores, or use `[$0]` instead"
    )]
    InvalidOrdinalLiteral,
    #[strum(message = "String literal is not terminated")]
    UnterminatedString,
    #[strum(message = "Invalid escape sequence in string")]
    InvalidEscapeSequence,
    #[strum(message = "The value of hex escape sequence is not a valid ASCII character")]
    InvalidHexEscapeSequence,
    #[strum(message = "The value of Unicode escape sequence is not a valid Unicode code point")]
    InvalidUnicodeEscapeSequence,
    #[strum(message = "An expression is expected after `..`")]
    BadArraySpread,
    #[strum(message = "Interpolation expression is not terminated")]
    UnterminatedInterpolation,
    #[strum(message = "Invalid interpolation expression")]
    BadInterpolation,
    #[strum(message = "Empty interpolation expression")]
    EmptyInterpolation,
    #[strum(message = "Unexpected `_`; it is a reserved keyword for discarding values")]
    UnexpectedUnderscore,
    #[strum(message = "Unexpected `global`; it is a reserved keyword for global variables")]
    UnexpectedGlobal,
    #[strum(message = "Missing `,` in the list")]
    MissingComma,
    #[strum(message = "Missing `]` to close the bracket")]
    MissingCloseBracket,
    #[strum(message = "Missing `{` to open the brace")]
    MissingOpenBrace,
    #[strum(message = "Missing `}` to close the brace")]
    MissingCloseBrace,
    #[strum(message = "Missing `)` to close the parenthesis")]
    MissingCloseParen,
    #[strum(message = "Missing `;` at the end of the statement")]
    MissingSemicolon,
    #[strum(message = "Missing `:` in the conditional expression")]
    MissingColon,
    #[strum(message = "Operator `=` is expected in a bind statement or const statement")]
    MissingBindOperator,
    #[strum(message = "Constant name must start with '@'")]
    InvalidConstantName,
    #[strum(message = "Missing function name in the declaration")]
    MissingFunctionName,
    #[strum(message = "Extension call must be ended with parameter list; add `(` here")]
    MissingOpenParenAfterExtension,
    #[strum(message = "`type` is a function-like keyword; add `(` here")]
    MissingOpenParenAfterType,
    #[strum(message = "`type` call must have exactly one argument")]
    InvalidTypeCall,
    #[strum(message = "Unexpected record literal; a grouping expression is expected")]
    RecordLiteralInExtensionCaller,
    #[strum(message = "Missing `case` in the statement")]
    MissingCase,
    #[strum(message = "Unknown expression encountered")]
    UnknownExpression,
    #[strum(message = "Unmatched `}` found")]
    UnmatchedCloseBrace,
    #[strum(message = "Unmatched `]` found")]
    UnmatchedCloseBracket,
    #[strum(message = "Unmatched `)` found")]
    UnmatchedCloseParen,
    #[strum(message = "Unknown pattern encountered")]
    UnknownPattern,
    #[strum(message = "Only constants or literal values are allowed here")]
    InvalidConstantLiteral,
    #[strum(message = "Unknown statement encountered")]
    UnknownStatement,
    #[strum(message = "An expression is expected here")]
    ExpressionExpected,
    #[strum(message = "A pattern is expected here")]
    PatternExpected,
    #[strum(message = "Operator `!` is not allowed in a literal pattern")]
    ExclamationInLiteralPattern,
    #[strum(message = "`mut` is not allowed during rebinding")]
    MutInRebindPattern,
    #[strum(message = "variable whose name starts with `@` is not allowed to be rebound")]
    ConstantInBindPattern,
    #[strum(message = "Cannot use `mut` in a discard pattern")]
    MutInDiscardPattern,
    #[strum(message = "Discard pattern should be omitted in a spread pattern")]
    DiscardInSpreadPattern,
    #[strum(message = "Spread discard in record pattern is not allowed")]
    SpreadDiscardInRecordPattern,
    #[strum(message = "Spread in record pattern should be the last field")]
    MispositionedSpreadInRecordPattern,
    #[strum(message = "Interpolated names are not allowed in record patterns")]
    InterpolatedNameRecordPattern,
    #[strum(message = "A bind pattern is required when omitting a record field name")]
    BadOmitKeyRecordPattern,
    #[strum(message = "Range pattern in array pattern should be parenthesized")]
    AmbiguousRangePattern,
    #[strum(message = "Spread pattern can only be used once in an array pattern")]
    DuplicateSpreadPattern,
    #[strum(message = "Rest parameter should be the last parameter in a function declaration")]
    MispositionedRestParameter,
    #[strum(message = "Cannot assign to an undeclared variable")]
    UndefinedVariableAssignment,
    #[strum(message = "Cannot assign to an immutable variable...")]
    ImmutableVariableAssignment,
    #[strum(message = "Cannot access a variable before it is...")]
    UninitializedVariable,
    #[strum(message = "The variable is already...")]
    DuplicateVariableDeclaration,
    #[strum(message = "Unexpected `break` outside of a loop")]
    UnexpectedBreak,
    #[strum(message = "Unexpected `continue` outside of a loop")]
    UnexpectedContinue,
    #[strum(
        message = "`global` keyword can only be used as `global.<name>`, `global[<name>]`, or on the right-hand side of the `in` operator"
    )]
    MisuseOfGlobalKeyword,
    #[strum(message = "Cannot infer key from the expression")]
    BadOmitKeyRecordExpression,
    #[strum(message = "Can only assign to a variable or a field access")]
    UnassignableExpression,

    ErrorEnd = 1999,
    // Warning 2000~2999
    WarningStart = 2000,

    #[strum(message = "Unnecessary parentheses; consider removing them")]
    UnnecessaryParentheses,
    // The null value in MiraScript is represented by `nil`,
    // Emit a warning when a global variable is read as `null` `undefined` or similar.
    #[strum(
        message = "Either use `global.$0` explicitly or `nil` if you want to use the nil value"
    )]
    MisleadingNilVariable,
    #[strum(
        message = "This pattern in a irrefutable matching is unnecessary; consider removing it or using in an `is` expression instead"
    )]
    UnnecessaryIrrefutablePattern,
    #[strum(message = "This `match` expression has no cases; it will never match any value")]
    MatchExpressionHasNoCases,

    // Static type checking warnings
    #[strum(message = "Non-number literal cannot be used in range")]
    NonNumberInRange,
    #[strum(message = "Non-number-or-string literal cannot be used in comparison expression")]
    NonNumberOrStringInComparison,
    #[strum(message = "Non-number literal cannot be used in arithmetic expression")]
    NonNumberInArithmetic,
    #[strum(message = "Non-boolean literal cannot be used in logical expression")]
    NonBooleanInLogical,
    #[strum(message = "Literal cannot be called as a function")]
    LiteralNotCallable,
    #[strum(message = "Literal cannot be accessed as a record or array")]
    LiteralNotIndexable,

    // Code style warnings
    #[strum(message = "Prefer if expression over conditional expression")]
    PreferIfExpression,
    #[strum(message = "Prefer `()` over `{}` for record literal declaration")]
    PreferParenthesesForRecordLiteral,
    #[strum(message = "Prefer `&&` over `and` for logical operations")]
    PreferLogicalOperatorAnd,
    #[strum(message = "Prefer `||` over `or` for logical operations")]
    PreferLogicalOperatorOr,
    #[strum(message = "Prefer `!` over `not` for logical operations")]
    PreferLogicalOperatorNot,

    // For analyzer
    #[strum(message = "Global variable `$0` is not declared")]
    GlobalVariableNotDeclared,
    #[strum(message = "Prefer uppercase for constant $0")]
    PreferUppercaseConstant,

    WarningEnd = 2999,
    // Info 3000~3999
    InfoStart = 3000,

    InfoEnd = 3999,
    // Hint 4000~4999
    HintStart = 4000,

    #[strum(message = "Local variable is unused; consider removing it or use `_` to ignore it")]
    UnusedLocalVariable,
    #[strum(message = "Local function is unused; consider removing it")]
    UnusedLocalFunction,

    HintEnd = 4999,
    // Reference 5000~5999
    ReferenceStart = 5000,

    #[strum(message = "...declared here")]
    VariableDeclaredHere,
    #[strum(message = "...declared here")]
    FunctionDeclaredHere,
    #[strum(message = "...declared as a parameter here")]
    ParameterDeclaredHere,
    #[strum(message = "...declared as the auto parameter `it` by this function here")]
    ParameterItDeclaredHere,
    #[strum(message = "...declared as a rest parameter here")]
    ParameterRestDeclaredHere,
    #[strum(message = "...declared as a sub-pattern of parameter pattern here")]
    ParameterSubPatternDeclaredHere,

    ReferenceEnd = 5999,

    // Tags 10000~10999
    TagStart = 10000,

    // mark local declarations

    // non-parameter declarations
    LocalConst,
    LocalImmutable,
    LocalMutable,
    LocalFunction,

    // parameter declarations
    ParameterIt,
    ParameterImmutable,
    ParameterMutable,
    ParameterImmutableRest,
    ParameterMutableRest,
    // parameter patterns holds parameters, but make no declarations
    ParameterPattern,
    ParameterRestPattern,
    // parameter sub-patterns are declarations inside patterns, they are not parameters
    ParameterSubPatternImmutable,
    ParameterSubPatternMutable,

    // mark global accesses
    GlobalVariable,
    GlobalDynamicAccess,

    // mark record initialization
    RecordFieldIdName,
    RecordFieldOrdinalName,
    RecordFieldStringName,
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

    // mark function calls
    FunctionCall,
    ExtensionCall,

    // mark code ranges
    Scope,
    String,
    Interpolation,

    // mark control flows
    ForExpression,
    WhileExpression,
    LoopExpression,
    FnDeclaration,
    FnExpression,
    IfExpression,
    MatchExpression,

    TagEnd = 10999,

    // TagRef 11000~11999
    TagRefStart = 11000,

    // mark local accesses
    ReadLocal,
    ReadWriteLocal,
    WriteLocal,
    RedeclareLocal,

    // mark function calls
    Callable,
    ArgumentExtension,
    ArgumentStart,
    ArgumentEnd,
    ArgumentComma,
    ArgumentSpread,

    // mark control flows
    KeywordFor,
    KeywordIn,
    KeywordWhile,
    KeywordLoop,
    KeywordBreak,
    KeywordContinue,

    KeywordIf,
    KeywordElse,

    KeywordMatch,
    KeywordCase,

    KeywordFn,
    KeywordReturn,

    /// Work with [DiagnosticCode::OmitNamedRecordField]
    OmitNamedRecordFieldName,

    TagRefEnd = 11999,

    SourceMap = 12000,
}

impl From<DiagnosticCode> for u16 {
    fn from(val: DiagnosticCode) -> Self {
        val.code()
    }
}

impl TryFrom<u16> for DiagnosticCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        DiagnosticCode::from_repr(value).ok_or(())
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

    pub fn is_tag(&self) -> bool {
        *self >= DiagnosticCode::TagStart && *self < DiagnosticCode::TagRefEnd
    }

    pub fn is_sourcemap(&self) -> bool {
        *self == DiagnosticCode::SourceMap
    }
}

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

    #[strum(message = "发生未知内部错误")]
    InternalError,
    #[strum(message = "发生未知词法错误")]
    LexerError,
    #[strum(message = "发生未知解析错误")]
    ParserError,
    #[strum(message = "发生未知生成错误")]
    EmitterError,
    #[strum(message = "发生未知优化错误")]
    OptimizerError,
    #[strum(message = "该功能尚未实现")]
    Unimplemented,

    #[strum(message = "遇到未知的记号")]
    UnknownToken,
    #[strum(message = "发现意外的记号")]
    UnexpectedToken,
    #[strum(message = "`$0` 是保留关键字，不能用作标识符")]
    InvalidReservedKeyword,
    #[strum(message = "`$0` 是关键字，不能用作标识符")]
    InvalidKeyword,
    #[strum(message = "数字字面量不能以下划线开头或结尾")]
    InvalidNumberLiteralUnderscore,
    #[strum(message = "无效的数字字面量")]
    InvalidNumberLiteral,
    #[strum(message = "数字字面量过大")]
    OverflowNumberLiteral,
    #[strum(message = "整数字面量过大")]
    OverflowIntegerLiteral,
    #[strum(message = "无效的序数字面量；请移除前导零和下划线，或改用 `[$0]`")]
    InvalidOrdinalLiteral,
    #[strum(message = "字符串字面量未终止")]
    UnterminatedString,
    #[strum(message = "字符串中的转义序列无效")]
    InvalidEscapeSequence,
    #[strum(message = "十六进制转义序列的值不是有效的 ASCII 字符")]
    InvalidHexEscapeSequence,
    #[strum(message = "Unicode 转义序列的值不是有效的 Unicode 码点")]
    InvalidUnicodeEscapeSequence,
    #[strum(message = "在 `..` 之后需要表达式")]
    BadArraySpread,
    #[strum(message = "插值表达式未终止")]
    UnterminatedInterpolation,
    #[strum(message = "无效的插值表达式")]
    BadInterpolation,
    #[strum(message = "插值表达式为空")]
    EmptyInterpolation,
    #[strum(message = "意外的 `_`；它是用于丢弃值的保留关键字")]
    UnexpectedUnderscore,
    #[strum(message = "意外的 `global`；它是用于全局变量的保留关键字")]
    UnexpectedGlobal,
    #[strum(message = "列表中缺少 `,`")]
    MissingComma,
    #[strum(message = "缺少 `]` 以关闭中括号")]
    MissingCloseBracket,
    #[strum(message = "缺少 `{` 以打开花括号")]
    MissingOpenBrace,
    #[strum(message = "缺少 `}` 以关闭花括号")]
    MissingCloseBrace,
    #[strum(message = "缺少 `)` 以关闭括号")]
    MissingCloseParen,
    #[strum(message = "语句末尾缺少 `;`")]
    MissingSemicolon,
    #[strum(message = "条件表达式中缺少 `:`")]
    MissingColon,
    #[strum(message = "在 bind 或 const 语句中需要 `=` 运算符")]
    MissingBindOperator,
    #[strum(message = "常量名必须以 `@` 开头")]
    InvalidConstantName,
    #[strum(message = "声明中缺少函数名")]
    MissingFunctionName,
    #[strum(message = "声明中缺少模块名")]
    MissingModuleName,
    #[strum(message = "扩展调用必须以参数列表结尾；请在此处添加 `(`")]
    MissingOpenParenAfterExtension,
    #[strum(message = "`type` 是类函数关键字；请在此处添加 `(`")]
    MissingOpenParenAfterType,
    #[strum(message = "`type` 调用必须恰好有一个参数")]
    InvalidTypeCall,
    #[strum(message = "意外的记录字面量；此处需要分组表达式")]
    RecordLiteralInExtensionCaller,
    #[strum(message = "语句中缺少 `case`")]
    MissingCase,
    #[strum(message = "遇到未知的表达式")]
    UnknownExpression,
    #[strum(message = "发现未匹配的 `}`")]
    UnmatchedCloseBrace,
    #[strum(message = "发现未匹配的 `]`")]
    UnmatchedCloseBracket,
    #[strum(message = "发现未匹配的 `)`")]
    UnmatchedCloseParen,
    #[strum(message = "遇到未知的模式")]
    UnknownPattern,
    #[strum(message = "此处只允许常量或字面量")]
    InvalidConstantLiteral,
    #[strum(message = "遇到未知的语句")]
    UnknownStatement,
    #[strum(message = "此处需要表达式")]
    ExpressionExpected,
    #[strum(message = "此处需要模式")]
    PatternExpected,
    #[strum(message = "字面量模式中不允许 `!` 运算符")]
    ExclamationInLiteralPattern,
    #[strum(message = "重绑定时不允许使用 `mut`")]
    MutInRebindPattern,
    #[strum(message = "名称以 `@` 开头的变量不允许被重绑定")]
    ConstantInBindPattern,
    #[strum(message = "弃元模式中不能使用 `mut`")]
    MutInDiscardPattern,
    #[strum(message = "展开模式中应省略丢弃弃元")]
    DiscardInSpreadPattern,
    #[strum(message = "记录模式中不允许展开弃元")]
    SpreadDiscardInRecordPattern,
    #[strum(message = "记录模式中的展开模式应为最后一个字段")]
    MispositionedSpreadInRecordPattern,
    #[strum(message = "记录模式中不允许插值名称")]
    InterpolatedNameRecordPattern,
    #[strum(message = "省略记录字段名时需要绑定模式")]
    BadOmitKeyRecordPattern,
    #[strum(message = "数组模式中的范围模式应加括号")]
    AmbiguousRangePattern,
    #[strum(message = "数组模式中展开模式只能使用一次")]
    DuplicateSpreadPattern,
    #[strum(message = "函数声明中剩余参数应为最后一个参数")]
    MispositionedRestParameter,
    #[strum(message = "不能对未声明的变量赋值")]
    UndefinedVariableAssignment,
    #[strum(message = "不能对不可变变量赋值...")]
    ImmutableVariableAssignment,
    #[strum(message = "变量无法在此之前访问...")]
    UninitializedVariable,
    #[strum(message = "该变量已...")]
    DuplicateVariableDeclaration,
    #[strum(message = "在循环之外出现意外的 `break`")]
    UnexpectedBreak,
    #[strum(message = "在循环之外出现意外的 `continue`")]
    UnexpectedContinue,
    #[strum(
        message = "`global` 关键字只能用作 `global.<name>`、`global[<name>]` 或 `in` 运算符右侧"
    )]
    MisuseOfGlobalKeyword,
    #[strum(message = "无法从表达式推断键名")]
    BadOmitKeyRecordExpression,
    #[strum(message = "只能对变量或字段访问赋值")]
    UnassignableExpression,
    #[strum(message = "在模块声明之外出现意外的 `pub`")]
    UnexpectedPub,

    ErrorEnd = 1999,
    // Warning 2000~2999
    WarningStart = 2000,

    // The null value in MiraScript is represented by `nil`,
    // Emit a warning when a global variable is read as `null` `undefined` or similar.
    #[strum(message = "`$0` 不是空值；显式使用全局变量 `global.$0` 或空值 `nil`")]
    MisleadingNilVariable,
    #[strum(message = "不可失败匹配中的该模式是多余的；请考虑移除它或改为在 `is` 表达式中使用")]
    UnnecessaryIrrefutablePattern,
    #[strum(message = "该 `match` 表达式没有分支；它永远不会匹配任何值")]
    MatchExpressionHasNoCases,

    // Static type checking warnings
    #[strum(message = "范围中不能使用非数字字面量")]
    NonNumberInRange,
    #[strum(message = "比较表达式中不能使用非数字或字符串字面量")]
    NonNumberOrStringInComparison,
    #[strum(message = "算术表达式中不能使用非数字字面量")]
    NonNumberInArithmetic,
    #[strum(message = "逻辑表达式中不能使用非布尔字面量")]
    NonBooleanInLogical,
    #[strum(message = "`in` 运算符右侧必须为复合类型")]
    NonCompoundIn,
    #[strum(message = "字面量不能作为函数调用")]
    LiteralNotCallable,
    #[strum(message = "字面量不能作为记录或数组访问")]
    LiteralNotIndexable,

    // For analyzer
    #[strum(message = "全局变量 `$0` 未声明")]
    GlobalVariableNotDeclared,

    WarningEnd = 2999,
    // Info 3000~3999
    InfoStart = 3000,

    InfoEnd = 3999,
    // Hint 4000~4999
    HintStart = 4000,

    #[strum(message = "局部变量未使用；请考虑删除它或使用 `_` 忽略")]
    UnusedLocalVariable,
    #[strum(message = "局部函数未使用；请考虑删除它")]
    UnusedLocalFunction,

    // Code style
    #[strum(message = "逻辑运算中更推荐使用 `&&` 而非 `and`")]
    PreferLogicalOperatorAnd,
    #[strum(message = "逻辑运算中更推荐使用 `||` 而非 `or`")]
    PreferLogicalOperatorOr,
    #[strum(message = "逻辑运算中更推荐使用 `!` 而非 `not`")]
    PreferLogicalOperatorNot,
    #[strum(message = "记录字面量声明中更推荐使用 `()` 而非 `{}`")]
    PreferParenthesesForRecordLiteral,
    #[strum(message = "更推荐使用 if 表达式而非条件表达式")]
    PreferIfExpression,

    #[strum(message = "考虑移除多余的括号")]
    UnnecessaryParentheses,

    // For analyzer
    #[strum(message = "常量 $0 建议使用大写")]
    PreferUppercaseConstant,

    HintEnd = 4999,
    // Reference 5000~5999
    ReferenceStart = 5000,

    #[strum(message = "...在此处声明")]
    VariableDeclaredHere,
    #[strum(message = "...在此处声明")]
    FunctionDeclaredHere,
    #[strum(message = "...在此处作为参数声明")]
    ParameterDeclaredHere,
    #[strum(message = "...在此处被该函数声明为自动参数 `it`")]
    ParameterItDeclaredHere,
    #[strum(message = "...在此处作为剩余参数声明")]
    ParameterRestDeclaredHere,
    #[strum(message = "...在此处作为参数模式的子模式声明")]
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
    LocalModule,

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
    // read
    ReadLocal,
    // compound read-write
    ReadWriteLocal,
    // write
    WriteLocal,
    // redeclare local declarations, will also report as error
    RedeclareLocal,
    // export by `pub` keyword, mark on the module identifier
    ExportedLocal,

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

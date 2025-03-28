use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use crate::{
    ansi::{RECOVER, RESET},
    lexer::Token,
    utils::{SourceError, SourceRange},
};

use super::{ArrayInitElement, Iterable, RecordLikeElement, Statement};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    // primary
    /// literal | 'true' | 'false' | 'nil'
    Literal(Box<Token<'a>>),
    /// interpolated_string
    ///
    /// Holds a [crate::lexer::TokenKind::InterpolatedString].
    InterpolatedString(Box<Token<'a>>),
    /// identifier
    Variable(Box<Token<'a>>),
    /// `(` expression `)`
    Grouping(Box<Expression<'a>>),
    /// `(` element (`,` element)* `)`
    ///
    /// Use `()` for an empty record.
    ///
    /// For a single-element record, use `(element,)`.
    Record(Vec<RecordLikeElement<'a>>),
    /// `[` element (`,` element)* `]`
    ///
    /// Use `[]` for an empty list.
    Array(Vec<ArrayInitElement<'a>>),

    // function
    /// expression `(` arguments `)`
    ///
    /// Arguments are a list of expressions, trailing comma is optional.
    Call(Box<Expression<'a>>, Vec<Expression<'a>>),
    /// expression `.` field
    ///
    /// Field must be an identifier or an ordinal.
    Access(Box<Expression<'a>>, Box<Token<'a>>),
    /// expression `[` expression `]`
    Index(Box<Expression<'a>>, Box<Expression<'a>>),

    // unary
    /// `not` expression
    Not(Box<Expression<'a>>),
    /// `-` expression
    Negate(Box<Expression<'a>>),
    /// `+` expression
    Plus(Box<Expression<'a>>),

    // exponent
    /// expression `^` expression
    Exponent(Box<Expression<'a>>, Box<Expression<'a>>),

    // factor
    /// expression `*` expression
    Multiply(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `/` expression
    Divide(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `%` expression
    Modulo(Box<Expression<'a>>, Box<Expression<'a>>),

    // term
    /// expression `+` expression
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `-` expression
    Subtract(Box<Expression<'a>>, Box<Expression<'a>>),

    // comparison
    /// expression `==` expression
    Equal(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `!=` expression
    NotEqual(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `<` expression
    Less(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `<=` expression
    LessEqual(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `>` expression
    Greater(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `>=` expression
    GreaterEqual(Box<Expression<'a>>, Box<Expression<'a>>),

    // and
    /// expression `and` expression
    And(Box<Expression<'a>>, Box<Expression<'a>>),

    // or
    /// expression `or` expression
    Or(Box<Expression<'a>>, Box<Expression<'a>>),

    // pipe
    /// expression `|>` expression
    ForwardPipe(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `<|` expression
    BackwardPipe(Box<Expression<'a>>, Box<Expression<'a>>),

    // block-like
    /// `{` statements* expression? `}`
    ///
    /// The value of the block is the value of the last expression.
    /// If no expression is present, the value is `nil`.
    Block(Vec<Statement<'a>>, Option<Box<Expression<'a>>>),
    /// `loop` block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is the expression of the `break` statement if present. Otherwise, `nil`.
    Loop(Box<Expression<'a>>),
    /// `while` expression block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is `nil`.
    While(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `for` identifier `in` expression block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is `nil`.
    ForIn(Box<Token<'a>>, Box<Iterable<'a>>, Box<Expression<'a>>),
    /// `if` expression block_expression (`else` expression)?
    ///
    /// The `then_block` is a block expression.
    ///
    /// The `else_block` is a block expression or an if expression.
    If(
        Box<Expression<'a>>,
        Box<Expression<'a>>,
        Option<Box<Expression<'a>>>,
    ),
    /// `match` expression `{` ((literal | '_') block_expression)* `}`
    ///
    /// The value of the block is the value of the matched expression.
    ///
    /// If no match is found, the value is `nil`.
    Match(Box<Expression<'a>>, Vec<(Expression<'a>, Expression<'a>)>),
    /// `fn` (parameters) block_expression
    ///
    /// Just like function declarations, but without the identifier.
    /// See [Statement::Function] for more details.
    Function(Option<Vec<Token<'a>>>, Box<Expression<'a>>),
    /// Unknown expression
    Unknown {
        tokens: Vec<Token<'a>>,
        errors: Vec<SourceError>,
    },
}

impl<'a> Expression<'a> {
    pub(crate) fn unknown<T: Into<Vec<Token<'a>>>, E: Into<Cow<'static, str>>>(
        tokens: T,
        error: E,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Expression::Unknown {
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }
    pub(crate) fn unknown_range<T: Into<Vec<Token<'a>>>, E: Into<Cow<'static, str>>>(
        tokens: T,
        error_range: SourceRange,
        error: E,
    ) -> Self {
        Expression::Unknown {
            tokens: tokens.into(),
            errors: vec![SourceError::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'a>>>, E: Into<Vec<SourceError>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Expression::Unknown {
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Expression::*;
        match self {
            Literal(token) => write!(f, "{}", token),
            InterpolatedString(token) => write!(f, "{}", token),
            Variable(token) => write!(f, "{}", token),
            Grouping(exp) => write!(f, "({})", exp),
            Record(exps) => {
                if exps.is_empty() {
                    return write!(f, "()");
                }
                write!(f, "(")?;
                for exp in exps {
                    write!(f, "{}", exp)?;
                    if exp.has_tail_comma() {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Array(exps) => {
                if exps.is_empty() {
                    return write!(f, "[]");
                }
                write!(f, "[")?;
                let mut iter = exps.iter();
                if let Some(exp) = iter.next() {
                    write!(f, "{}", exp)?;
                    for exp in iter {
                        write!(f, ", {}", exp)?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            Call(exp, args) => {
                write!(f, "{}(", exp)?;
                let mut iter = args.iter();
                if let Some(arg) = iter.next() {
                    write!(f, "{}", arg)?;
                    for arg in iter {
                        write!(f, ", {}", arg)?;
                    }
                }
                write!(f, ")")
            }
            Access(exp, token) => write!(f, "{}.{}", exp, token),
            Index(exp1, exp2) => write!(f, "{}[{}]", exp1, exp2),
            Not(exp) => write!(f, "!{}", exp),
            Negate(exp) => write!(f, "-{}", exp),
            Plus(exp) => write!(f, "+{}", exp),
            Exponent(exp1, exp2) => write!(f, "{} ^ {}", exp1, exp2),
            Multiply(exp1, exp2) => write!(f, "{} * {}", exp1, exp2),
            Divide(exp1, exp2) => write!(f, "{} / {}", exp1, exp2),
            Modulo(exp1, exp2) => write!(f, "{} % {}", exp1, exp2),
            Add(exp1, exp2) => write!(f, "{} + {}", exp1, exp2),
            Subtract(exp1, exp2) => write!(f, "{} - {}", exp1, exp2),
            Equal(exp1, exp2) => write!(f, "{} == {}", exp1, exp2),
            NotEqual(exp1, exp2) => write!(f, "{} != {}", exp1, exp2),
            Less(exp1, exp2) => write!(f, "{} < {}", exp1, exp2),
            LessEqual(exp1, exp2) => write!(f, "{} <= {}", exp1, exp2),
            Greater(exp1, exp2) => write!(f, "{} > {}", exp1, exp2),
            GreaterEqual(exp1, exp2) => write!(f, "{} >= {}", exp1, exp2),
            And(exp1, exp2) => write!(f, "{} && {}", exp1, exp2),
            Or(exp1, exp2) => write!(f, "{} || {}", exp1, exp2),
            ForwardPipe(left, right) => write!(f, "{} |> {}", left, right),
            BackwardPipe(left, right) => write!(f, "{} <| {}", left, right),
            Block(statements, expression) => {
                if statements.is_empty() {
                    if let Some(expression) = expression {
                        return write!(f, "{{{}}}", expression);
                    } else {
                        return write!(f, "{{}}");
                    }
                }
                writeln!(f, "{{")?;
                for statement in statements {
                    write!(f, "{}", statement)?;
                }
                if let Some(expression) = expression {
                    writeln!(f, "{}", expression)?;
                }
                write!(f, "}}")
            }
            Loop(expression) => write!(f, "loop {}", expression),
            While(expression, block) => {
                write!(f, "while {} {}", expression, block)
            }
            ForIn(token, expression, block) => {
                write!(f, "for {} in {} {}", token, expression, block)
            }
            If(expression, then_block, else_block) => {
                if let Some(else_block) = else_block {
                    write!(f, "if {} {} else {}", expression, then_block, else_block)
                } else {
                    write!(f, "if {} {}", expression, then_block)
                }
            }
            Match(expression, arms) => {
                writeln!(f, "match {} {{", expression)?;
                for (pattern, block) in arms {
                    writeln!(f, "{} {}", pattern, block)?;
                }
                write!(f, "}}")
            }
            Function(None, block) => write!(f, "fn {}", block),
            Function(Some(params), block) => {
                write!(f, "fn (")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{}", param)?;
                    for param in iter {
                        write!(f, ", {}", param)?;
                    }
                }
                write!(f, ") {}", block)
            }
            Unknown { .. } => write!(f, "{RECOVER}(<???>){RESET}"),
        }
    }
}

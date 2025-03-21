use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use crate::{
    lexer::Token,
    utils::{Range, SourceError},
};

use super::Statement;

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
    /// `(` expression (`,` expression)* `)`
    ///
    /// Use `()` for an empty tuple.
    ///
    /// For a single-element tuple, use `(expression,)`.
    Tuple(Vec<Expression<'a>>),
    /// `(` name `:` expression (`,` name `:` expression)* `)`
    ///
    /// Name should be an identifier or an ordinal.
    ///
    /// All elements must be named or unnamed.
    NamedTuple(Vec<(Token<'a>, Expression<'a>)>),
    /// `[` expression (`,` expression)* `]`
    ///
    /// Use `[]` for an empty list.
    Array(Vec<Expression<'a>>),

    // function
    /// expression `(` arguments `)`
    ///
    /// Arguments are a list of expressions, trailing comma is optional.
    Call(Box<Expression<'a>>, Vec<Expression<'a>>),
    /// expression `.` field
    ///
    /// Field must be an identifier or an ordinal.
    Access(Box<Expression<'a>>, Box<Token<'a>>),

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
    ForIn(Box<Token<'a>>, Box<Expression<'a>>, Box<Expression<'a>>),
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
        error_range: Range,
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
        match self {
            Expression::Literal(token) => write!(f, "{}", token),
            Expression::InterpolatedString(token) => write!(f, "{}", token),
            Expression::Variable(token) => write!(f, "{}", token),
            Expression::Grouping(exp) => write!(f, "({})", exp),
            Expression::Tuple(exps) => {
                if exps.is_empty() {
                    return write!(f, "()");
                }
                if exps.len() == 1 {
                    return write!(f, "({},)", exps[0]);
                }
                write!(f, "(")?;
                let mut iter = exps.iter();
                if let Some(exp) = iter.next() {
                    write!(f, "{}", exp)?;
                    for exp in iter {
                        write!(f, ", {}", exp)?;
                    }
                } else {
                    write!(f, ",")?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expression::NamedTuple(exps) => {
                if exps.is_empty() {
                    return write!(f, "()");
                }
                write!(f, "(")?;
                let mut iter = exps.iter();
                if let Some((name, exp)) = iter.next() {
                    write!(f, "{}: {}", name, exp)?;
                    for (name, exp) in iter {
                        write!(f, ", {}: {}", name, exp)?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Expression::Array(exps) => {
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
            Expression::Call(exp, args) => {
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
            Expression::Access(exp, token) => write!(f, "{}.{}", exp, token),
            Expression::Not(exp) => write!(f, "not {}", exp),
            Expression::Negate(exp) => write!(f, "-{}", exp),
            Expression::Plus(exp) => write!(f, "+{}", exp),
            Expression::Exponent(exp1, exp2) => write!(f, "{} ^ {}", exp1, exp2),
            Expression::Multiply(exp1, exp2) => write!(f, "{} * {}", exp1, exp2),
            Expression::Divide(exp1, exp2) => write!(f, "{} / {}", exp1, exp2),
            Expression::Modulo(exp1, exp2) => write!(f, "{} % {}", exp1, exp2),
            Expression::Add(exp1, exp2) => write!(f, "{} + {}", exp1, exp2),
            Expression::Subtract(exp1, exp2) => write!(f, "{} - {}", exp1, exp2),
            Expression::Equal(exp1, exp2) => write!(f, "{} == {}", exp1, exp2),
            Expression::NotEqual(exp1, exp2) => write!(f, "{} != {}", exp1, exp2),
            Expression::Less(exp1, exp2) => write!(f, "{} < {}", exp1, exp2),
            Expression::LessEqual(exp1, exp2) => write!(f, "{} <= {}", exp1, exp2),
            Expression::Greater(exp1, exp2) => write!(f, "{} > {}", exp1, exp2),
            Expression::GreaterEqual(exp1, exp2) => write!(f, "{} >= {}", exp1, exp2),
            Expression::And(exp1, exp2) => write!(f, "{} and {}", exp1, exp2),
            Expression::Or(exp1, exp2) => write!(f, "{} or {}", exp1, exp2),
            Expression::Block(statements, expression) => {
                writeln!(f, "{{")?;
                for statement in statements {
                    write!(f, "{}", statement)?;
                }
                if let Some(expression) = expression {
                    writeln!(f, "{}", expression)?;
                }
                write!(f, "}}")
            }
            Expression::Loop(expression) => write!(f, "loop {}", expression),
            Expression::While(expression, block) => {
                write!(f, "while {} {}", expression, block)
            }
            Expression::ForIn(token, expression, block) => {
                write!(f, "for {} in {} {}", token, expression, block)
            }
            Expression::If(expression, then_block, else_block) => {
                if let Some(else_block) = else_block {
                    write!(f, "if {} {} else {}", expression, then_block, else_block)
                } else {
                    write!(f, "if {} {}", expression, then_block)
                }
            }
            Expression::Match(expression, arms) => {
                writeln!(f, "match {} {{", expression)?;
                for (pattern, block) in arms {
                    writeln!(f, "{} {}", pattern, block)?;
                }
                write!(f, "}}")
            }
            Expression::Function(None, block) => write!(f, "fn {}", block),
            Expression::Function(Some(params), block) => {
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
            Expression::Unknown { .. } => write!(f, "(<???>)"),
        }
    }
}

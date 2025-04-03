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
    Grouping(Box<Token<'a>>, Box<Expression<'a>>, Box<Token<'a>>),
    /// `(` element* `)`
    ///
    /// Use `()` for an empty record.
    ///
    /// For a single-element-unnamed record, use `(` expression `,` `)`.
    Record(Vec<RecordLikeElement<'a>>),
    /// `[` element* `]`
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

    /// op expression
    ///
    /// Unary operators are:
    /// - `!` logical not
    /// - `-` negation
    /// - `+` unary plus
    /// `not` expression
    Unary(Box<Token<'a>>, Box<Expression<'a>>),

    /// expression op expression
    ///
    /// Binary operators are:
    /// 1. `^` exponentiation
    /// 2. `*` `/` `%` multiplication, division, modulo
    /// 3. `+` `-` addition, subtraction
    /// 4. `>` `<` `>=` `<=` comparison
    /// 5. `==` `!=` `~=` `!~=` equality
    /// 6. `&&` logical and
    /// 7. `||` logical or
    /// 8. `|>` `<|` forward and backward pipe
    Binary(Box<Expression<'a>>, Box<Token<'a>>, Box<Expression<'a>>),

    // block-like
    /// `{` statements* expression? `}`
    ///
    /// The value of the block is the value of the last expression.
    /// If no expression is present, the value is `nil`.
    Block(
        Box<Token<'a>>,
        Vec<Statement<'a>>,
        Option<Box<Expression<'a>>>,
        Box<Token<'a>>,
    ),
    /// `loop` block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is the expression of the `break` statement if present. Otherwise, `nil`.
    Loop(Box<Token<'a>>, Box<Expression<'a>>),
    /// `while` expression block_expression (`else` expression)?
    ///
    /// The final expression of the block must not present.
    ///
    /// The `else_block` is a block expression or an if expression.
    ///
    /// The value of the block is the expression of the `break` statement if present.
    /// Otherwise, if the `else_block` is present,
    /// the value is the value of the `else_block`. Otherwise, `nil`.
    While(
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Expression<'a>>,
        Option<(Box<Token<'a>>, Box<Expression<'a>>)>,
    ),
    /// `for` identifier `in` expression block_expression (`else` expression)?
    ///
    /// The final expression of the block must not present.
    ///
    /// The `else_block` is a block expression or an if expression.
    ///
    /// The value of the block is the expression of the `break` statement if present.
    /// Otherwise, if the `else_block` is present,
    /// the value is the value of the `else_block`. Otherwise, `nil`.
    ForIn(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<Iterable<'a>>,
        Box<Expression<'a>>,
        Option<(Box<Token<'a>>, Box<Expression<'a>>)>,
    ),
    /// `if` expression block_expression (`else` expression)?
    ///
    /// The `then_block` is a block expression.
    ///
    /// The `else_block` is a block expression or an if expression.
    If(
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Expression<'a>>,
        Option<(Box<Token<'a>>, Box<Expression<'a>>)>,
    ),
    /// `match` expression `{` `case` ((literal | '_') block_expression)* `}`
    ///
    /// The value of the block is the value of the matched expression.
    ///
    /// If no match is found, the value is `nil`.
    Match(
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
        Vec<(Token<'a>, Expression<'a>, Expression<'a>)>,
        Box<Token<'a>>,
    ),
    /// `fn` parameters? block_expression
    ///
    /// Just like function declarations, but without the identifier.
    /// See [Statement::Function] for more details.
    Function(Box<Token<'a>>, Option<Vec<Token<'a>>>, Box<Expression<'a>>),
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
            Literal(token) => write!(f, "{token}"),
            InterpolatedString(token) => write!(f, "{token}"),
            Variable(token) => write!(f, "{token}"),
            Grouping(op, exp, cp) => write!(f, "{op}{exp}{cp}"),
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
            Unary(op, exp) => write!(f, "{op}{exp}"),
            Binary(exp1, op, exp2) => write!(f, "{exp1} {op} {exp2}"),
            Block(op, statements, expression, ed) => {
                if statements.is_empty() {
                    if let Some(expression) = expression {
                        return write!(f, "{op} {expression} {ed}");
                    } else {
                        return write!(f, "{op}{ed}");
                    }
                }
                writeln!(f, "{op}")?;
                for statement in statements {
                    write!(f, "{statement}")?;
                }
                if let Some(expression) = expression {
                    writeln!(f, "{expression}")?;
                }
                write!(f, "{ed}")
            }
            Loop(kw, expression) => write!(f, "{kw} {expression}"),
            While(kw, expression, block, None) => {
                write!(f, "{kw} {expression} {block}")
            }
            While(kw, expression, block, Some((kw_else, else_block))) => {
                write!(f, "{kw} {expression} {block} {kw_else} {else_block}")
            }
            ForIn(kw_for, token, kw_in, iter, block, None) => {
                write!(f, "{kw_for} {token} {kw_in} {iter} {block}")
            }
            ForIn(kw_for, token, kw_in, iter, block, Some((kw_else, else_block))) => {
                write!(
                    f,
                    "{kw_for} {token} {kw_in} {iter} {block} {kw_else} {else_block}"
                )
            }
            If(kw_if, cond, then_block, Some((kw_else, else_block))) => {
                write!(f, "{kw_if} {cond} {then_block} {kw_else} {else_block}")
            }
            If(kw_if, cond, then_block, None) => {
                write!(f, "{kw_if} {cond} {then_block}")
            }
            Match(kw, expression, op, arms, ed) => {
                writeln!(f, "{kw} {expression} {op}")?;
                for (kw_case, pattern, block) in arms {
                    writeln!(f, "{kw_case} {pattern} {block}")?;
                }
                write!(f, "{ed}")
            }
            Function(kw, None, block) => write!(f, "{kw} {}", block),
            Function(kw, Some(params), block) => {
                write!(f, "{kw} (")?;
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

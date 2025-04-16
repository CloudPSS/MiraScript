use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use crate::{
    ansi::{DisplayIdent, GROUP, INTERPOLATED, RECOVER, RESET, STRING},
    lexer::{Token, TokenKind},
    utils::{SourceError, SourceRange},
};

use super::{ArrayInitElement, Iterable, Pattern, RecordElement, Statement};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    // primary
    /// string | number | ordinal | `true` | `false` | `nil`
    Literal(Box<Token<'a>>),
    /// interpolated_string
    ///
    /// Holds a [crate::lexer::TokenKind::InterpolatedString], and a list of expressions
    /// that are interpolated into the string.
    InterpolatedString(Box<Token<'a>>, Vec<Expression<'a>>),
    /// identifier
    Variable(Box<Token<'a>>),
    /// `(` expression `)`
    Grouping(Box<Token<'a>>, Box<Expression<'a>>, Box<Token<'a>>),
    /// `(` element* `)`
    ///
    /// Use `()` for an empty record.
    ///
    /// For a single-element-unnamed record, use `(` expression `,` `)`.
    Record(Vec<RecordElement<'a>>),
    /// `[` element* `]`
    ///
    /// Use `[]` for an empty list.
    Array(Vec<ArrayInitElement<'a>>),

    // postfix
    /// `type` `(` expression `)`
    Type(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// expression `(` arguments `)`
    ///
    /// Arguments are a list of expressions, trailing comma is optional.
    Call(
        Box<Expression<'a>>,
        Box<Token<'a>>,
        Vec<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// expression `.` field
    ///
    /// Field must be an identifier or an ordinal.
    Access(Box<Expression<'a>>, Box<Token<'a>>),
    /// expression `[` expression `]`
    Index(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `!`
    NonNil(Box<Expression<'a>>, Box<Token<'a>>),

    /// op expression
    ///
    /// Prefix operators are:
    /// - `!` logical not
    /// - `-` negation
    /// - `+` unary plus
    Prefix(Box<Token<'a>>, Box<Expression<'a>>),

    // infix
    /// expression op expression
    ///
    /// Infix operators are:
    /// 1. `^` exponentiation
    /// 1. `*` `/` `%` multiplicative
    /// 1. `+` `-` additive
    /// 1. `is` matching *Use [Expression::Is] for this
    /// 1. `>` `<` `>=` `<=` `in` relational
    /// 1. `==` `!=` `~=` `!~=` equality
    /// 1. `&&` logical and
    /// 1. `||` logical or
    /// 1. `|>` `<|` forward and backward pipe
    Infix(Box<Expression<'a>>, Box<Token<'a>>, Box<Expression<'a>>),
    /// expression `is` pattern
    Is(Box<Expression<'a>>, Box<Token<'a>>, Box<Pattern<'a>>),

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
    /// `for` pattern `in` expression block_expression (`else` expression)?
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
        Box<Pattern<'a>>,
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
        Vec<(Token<'a>, Pattern<'a>, Expression<'a>)>,
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
    pub(crate) fn is_unknown(&self) -> bool {
        matches!(self, Expression::Unknown { .. })
    }

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
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Expression<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, ident: usize) -> fmt::Result {
        use Expression::*;
        match self {
            Literal(token) => write!(f, "{token}"),
            InterpolatedString(token, e) => {
                let Token {
                    kind: TokenKind::InterpolatedString(s, _),
                    ..
                } = &**token
                else {
                    return write!(f, "{RECOVER}(\"<???>\"){RESET}");
                };
                write!(f, "{STRING}\"")?;
                assert_eq!(s.len(), e.len() + 1, "Invalid string interpolation");
                let mut s_iter = s.iter();
                let first = s_iter.next().ok_or(std::fmt::Error)?;
                write!(f, "{}", first.escape_debug())?;
                for (s, e) in s_iter.zip(e.iter()) {
                    write!(f, "{RESET}{INTERPOLATED}${RESET}")?;
                    e.fmt_ident(f, ident)?;
                    write!(f, "{STRING}")?;
                    write!(f, "{}", s.escape_debug())?;
                }
                write!(f, "\"{RESET}")
            }
            Variable(token) => write!(f, "{token}"),
            Grouping(op, exp, cp) => {
                write!(f, "{GROUP}{op}{RESET}")?;
                exp.fmt_ident(f, ident)?;
                write!(f, "{GROUP}{cp}{RESET}")?;
                Ok(())
            }
            Record(exps) => {
                if exps.is_empty() {
                    return write!(f, "()");
                }
                write!(f, "(")?;
                for exp in exps {
                    exp.fmt_ident(f, ident)?;
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
                    exp.fmt_ident(f, ident)?;
                    for exp in iter {
                        write!(f, ", ")?;
                        exp.fmt_ident(f, ident)?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            Type(kw_type, op, arg, cp) => {
                write!(f, "{kw_type}{op}")?;
                arg.fmt_ident(f, ident)?;
                write!(f, "{cp}")
            }
            Call(exp, op, args, cp) => {
                exp.fmt_ident(f, ident)?;
                write!(f, "{op}")?;
                let mut iter = args.iter();
                if let Some(arg) = iter.next() {
                    arg.fmt_ident(f, ident)?;
                    for arg in iter {
                        write!(f, ", ")?;
                        arg.fmt_ident(f, ident)?;
                    }
                }
                write!(f, "{cp}")
            }
            Access(exp, token) => {
                exp.fmt_ident(f, ident)?;
                write!(f, ".{token}")
            }
            Index(exp1, exp2) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, "[")?;
                exp2.fmt_ident(f, ident)?;
                write!(f, "]")
            }
            NonNil(exp, op) => {
                exp.fmt_ident(f, ident)?;
                write!(f, "{op}")
            }
            Prefix(op, exp) => {
                match op.kind {
                    TokenKind::Operator(_) => write!(f, "{op}")?,
                    _ => write!(f, "{op} ")?,
                }
                exp.fmt_ident(f, ident)
            }
            Infix(exp1, op, exp2) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, " {op} ")?;
                exp2.fmt_ident(f, ident)
            }
            Is(exp1, op, pattern) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, " {op} ")?;
                pattern.fmt_ident(f, ident)
            }
            Block(op, statements, expression, ed) => {
                if statements.is_empty() {
                    if let Some(expression) = expression {
                        return write!(f, "{GROUP}{op}{RESET} {expression} {GROUP}{ed}{RESET}");
                    } else {
                        return write!(f, "{GROUP}{op}{ed}{RESET}");
                    }
                }
                writeln!(f, "{GROUP}{op}{RESET}")?;
                for statement in statements {
                    statement.fmt_ident(f, Self::next_ident(ident))?;
                }
                if let Some(expression) = expression {
                    Self::write_ident(f, Self::next_ident(ident))?;
                    writeln!(f, "{expression}")?;
                }
                Self::write_ident(f, ident)?;
                write!(f, "{GROUP}{ed}{RESET}")
            }
            Loop(kw, expression) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)
            }
            While(kw, expression, block, None) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)
            }
            While(kw, expression, block, Some((kw_else, else_block))) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
                write!(f, " {kw_else} ")?;
                else_block.fmt_ident(f, ident)
            }
            ForIn(kw_for, pattern, kw_in, iter, block, None) => {
                write!(f, "{kw_for} ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {kw_in} ")?;
                iter.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)
            }
            ForIn(kw_for, pattern, kw_in, iter, block, Some((kw_else, else_block))) => {
                write!(f, "{kw_for} ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {kw_in} ")?;
                iter.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
                write!(f, " {kw_else} ")?;
                else_block.fmt_ident(f, ident)
            }
            If(kw_if, cond, then_block, Some((kw_else, else_block))) => {
                write!(f, "{kw_if} ")?;
                cond.fmt_ident(f, ident)?;
                write!(f, " ")?;
                then_block.fmt_ident(f, ident)?;
                write!(f, " {kw_else} ")?;
                else_block.fmt_ident(f, ident)
            }
            If(kw_if, cond, then_block, None) => {
                write!(f, "{kw_if} ")?;
                cond.fmt_ident(f, ident)?;
                write!(f, " ")?;
                then_block.fmt_ident(f, ident)
            }
            Match(kw, expression, op, arms, ed) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
                writeln!(f, " {GROUP}{op}{RESET}")?;
                let next_ident = Self::next_ident(ident);
                for (kw_case, pattern, block) in arms {
                    Self::write_ident(f, next_ident)?;
                    write!(f, "{kw_case} ")?;
                    pattern.fmt_ident(f, next_ident)?;
                    write!(f, " ")?;
                    block.fmt_ident(f, next_ident)?;
                    writeln!(f)?;
                }
                Self::write_ident(f, ident)?;
                write!(f, "{GROUP}{ed}{RESET}")
            }
            Function(kw, None, block) => {
                write!(f, "{kw} ")?;
                block.fmt_ident(f, ident)
            }
            Function(kw, Some(params), block) => {
                write!(f, "{kw} (")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{}", param)?;
                    for param in iter {
                        write!(f, ", {}", param)?;
                    }
                }
                write!(f, ") ")?;
                block.fmt_ident(f, ident)
            }
            Unknown { tokens, .. } => {
                write!(f, "{RECOVER}<expression{RESET}")?;
                for token in tokens {
                    write!(f, " {token}")?;
                }
                write!(f, "{RECOVER}>{RESET}")
            }
        }
    }
}

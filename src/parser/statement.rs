use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use crate::{
    ansi::{DisplayIdent, RECOVER, RESET},
    lexer::Token,
    utils::{SourceError, SourceRange},
};

use super::{Expression, Pattern};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    /// `';'`
    ///
    /// An empty statement.
    Empty(Box<Token<'a>>),
    /// `expression ';'`
    Expression(Box<Expression<'a>>, Box<Token<'a>>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(Box<Expression<'a>>),
    /// `'let' pattern '=' expression ';'`
    Bind(
        Box<Token<'a>>,
        Box<Pattern<'a>>,
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// `pattern_rebind '=' expression ';'`
    Rebind(
        Box<Pattern<'a>>,
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// `expression ('=' | '+=' | '-=' | '*=' | '/=' | '%=' | '^=' | '&&=' | '||=') expression ';'`
    ///
    /// The assigner must be one of the following:
    /// - `identifier`
    /// - `expression_access` where the accessed is an extern or `global`
    /// - `expression_index` where the indexed is an extern or `global`
    Assign(
        Box<Expression<'a>>,
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// `'fn' identifier (parameters) block_expression`
    ///
    /// Parameters are a list of identifiers, trailing comma is optional.
    ///
    /// If parameters and parentheses are omitted,
    /// the function is considered with an implicit parameter called `it`.
    ///
    /// ```
    /// fn filter { it % 2 == 0 }
    /// ```
    ///
    /// The function body is a block expression.
    Function(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Option<Vec<Token<'a>>>,
        Box<Expression<'a>>,
    ),
    /// `return expression;` or `return;`
    ///
    /// If the expression is omitted, the return value is `nil`.
    Return(Option<Box<Expression<'a>>>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(Option<Box<Expression<'a>>>),
    /// `continue;`
    Continue,
    /// Unknown statement.
    Unknown {
        tokens: Vec<Token<'a>>,
        errors: Vec<SourceError>,
    },
}

impl<'a> Statement<'a> {
    pub(crate) fn is_unknown(&self) -> bool {
        matches!(self, Statement::Unknown { .. })
    }

    pub(crate) fn unknown<T: Into<Vec<Token<'a>>>, E: Into<Cow<'static, str>>>(
        tokens: T,
        error: E,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Statement::Unknown {
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }

    pub(crate) fn unknown_range<T: Into<Vec<Token<'a>>>, E: Into<Cow<'static, str>>>(
        tokens: T,
        error_range: SourceRange,
        error: E,
    ) -> Self {
        Statement::Unknown {
            tokens: tokens.into(),
            errors: vec![SourceError::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'a>>>, E: Into<Vec<SourceError>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Statement::Unknown {
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Statement<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, ident: usize) -> fmt::Result {
        use Statement::*;
        Self::write_ident(f, ident)?;
        match self {
            Empty(c) => {
                writeln!(f, "{c}")
            }
            Expression(expr, c) => {
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            BlockExpression(expr) => {
                expr.fmt_ident(f, ident)?;
                writeln!(f)
            }
            Bind(kw_let, pattern, eq, expr, c) => {
                write!(f, "{kw_let} ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {eq} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Rebind(pattern, eq, expr, c) => {
                pattern.fmt_ident(f, ident)?;
                write!(f, " {eq} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Assign(exp, eq, expr, c) => {
                exp.fmt_ident(f, ident)?;
                write!(f, " {eq} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Function(kw, id, None, body) => {
                write!(f, "{kw} {id} ")?;
                body.fmt_ident(f, ident)?;
                writeln!(f)
            }
            Function(kw, id, Some(params), body) => {
                write!(f, "{kw} {id} (")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{param}")?;
                    for param in iter {
                        write!(f, ", {param}")?;
                    }
                }
                write!(f, ") ")?;
                body.fmt_ident(f, ident)?;
                writeln!(f)
            }
            Return(Some(expr)) => {
                write!(f, "return ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, ";")
            }
            Return(None) => writeln!(f, "return;"),
            Break(Some(expr)) => {
                write!(f, "break ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, ";")
            }
            Break(None) => writeln!(f, "break;"),
            Continue => writeln!(f, "continue;"),
            Unknown { tokens, .. } => {
                write!(f, "{RECOVER}<statement{RESET}")?;
                for token in tokens {
                    write!(f, " {token}")?;
                }
                writeln!(f, "{RECOVER}>{RESET}")
            }
        }
    }
}

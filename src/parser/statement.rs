use std::fmt::{self, Display, Formatter};

use crate::{
    ansi::{DisplayIdent, GROUP, RECOVER, RESET},
    error::{ErrorCode, SourceError, SourceRange},
    lexer::Token,
};

use super::{AstVisitor, Expression, Pattern, AstWalker};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
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
    Return(Box<Token<'a>>, Option<Box<Expression<'a>>>, Box<Token<'a>>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(Box<Token<'a>>, Option<Box<Expression<'a>>>, Box<Token<'a>>),
    /// `continue;`
    Continue(Box<Token<'a>>, Box<Token<'a>>),
    /// Unknown statement.
    Unknown {
        tokens: Vec<Token<'a>>,
        errors: Vec<SourceError>,
    },
}

impl<'a> Statement<'a> {
    pub(crate) fn unknown<T: Into<Vec<Token<'a>>>>(tokens: T, error: ErrorCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Statement::Unknown {
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }

    pub(crate) fn unknown_range<T: Into<Vec<Token<'a>>>>(
        tokens: T,
        error_range: SourceRange,
        error: ErrorCode,
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

impl<'a> AstWalker<'a> for Statement<'a> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
        use Statement::*;
        visitor.visit_statement(self);
        match self {
            Empty(c) => c.walk(visitor),
            Expression(expr, c) => {
                expr.walk(visitor);
                c.walk(visitor);
            }
            BlockExpression(expr) => expr.walk(visitor),
            Bind(kw_let, pattern, eq, expr, c) => {
                kw_let.walk(visitor);
                pattern.walk(visitor);
                eq.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Rebind(pattern, eq, expr, c) => {
                pattern.walk(visitor);
                eq.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Assign(exp, eq, expr, c) => {
                exp.walk(visitor);
                eq.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Function(kw, id, params, body) => {
                kw.walk(visitor);
                id.walk(visitor);
                params.walk(visitor);
                body.walk(visitor);
            }
            Return(kw, expr, c) => {
                kw.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Break(kw, expr, c) => {
                kw.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Continue(kw, c) => {
                kw.walk(visitor);
                c.walk(visitor);
            }
            Unknown { tokens, errors: _ } => {
                tokens.walk(visitor);
            }
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
        match self {
            Empty(c) => {
                Self::write_ident(f, ident, "")?;
                writeln!(f, "{c}")
            }
            Expression(expr, c) => {
                Self::write_ident(f, ident, "")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            BlockExpression(expr) => {
                Self::write_ident(f, ident, "")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f)
            }
            Bind(kw_let, pattern, eq, expr, c) => {
                Self::write_ident(f, ident, "")?;
                write!(f, "{kw_let} ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {eq} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Rebind(pattern, eq, expr, c) => {
                Self::write_ident(f, ident, "rebind")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {eq} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Assign(exp, eq, expr, c) => {
                Self::write_ident(f, ident, "assign")?;
                exp.fmt_ident(f, ident)?;
                write!(f, " {eq} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Function(kw, id, None, body) => {
                Self::write_ident(f, ident, "declare")?;
                write!(f, "{kw} {id} ")?;
                body.fmt_ident(f, ident)?;
                writeln!(f)
            }
            Function(kw, id, Some(params), body) => {
                Self::write_ident(f, ident, "declare")?;
                write!(f, "{kw} {id}{GROUP}({RESET}")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{param}")?;
                    for param in iter {
                        write!(f, ", {param}")?;
                    }
                }
                write!(f, "{GROUP}){RESET} ")?;
                body.fmt_ident(f, ident)?;
                writeln!(f)
            }
            Return(kw, Some(expr), c) | Break(kw, Some(expr), c) => {
                Self::write_ident(f, ident, "")?;
                write!(f, "{kw} ")?;
                expr.fmt_ident(f, ident)?;
                writeln!(f, "{c}")
            }
            Return(kw, None, c) | Break(kw, None, c) => {
                Self::write_ident(f, ident, "")?;
                writeln!(f, "{kw}{c}")
            }
            Continue(kw, c) => {
                Self::write_ident(f, ident, "")?;
                writeln!(f, "{kw}{c}")
            }
            Unknown { tokens, .. } => {
                Self::write_ident(f, ident, "???")?;
                write!(f, "{RECOVER}<statement{RESET}")?;
                for token in tokens {
                    write!(f, " {token}")?;
                }
                writeln!(f, "{RECOVER}>{RESET}")
            }
        }
    }
}

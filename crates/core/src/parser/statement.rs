use std::fmt::{self, Display, Formatter};

use crate::{
    ansi::{DisplayIdent, GROUP, RECOVER, RESET},
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    lexer::Token,
};

use super::{AstVisitor, AstVisitorMut, AstWalker, Expression, Pattern};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Statement<'s> {
    /// `';'`
    ///
    /// An empty statement.
    Empty(Box<Token<'s>>),
    /// `expression ';'`
    Expression(Box<Expression<'s>>, Box<Token<'s>>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(Box<Expression<'s>>),
    /// `'let' pattern '=' expression ';'`
    Bind(
        Box<Token<'s>>,
        Box<Pattern<'s>>,
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Token<'s>>,
    ),
    /// `pattern_rebind '=' expression ';'`
    Rebind(
        Box<Pattern<'s>>,
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Token<'s>>,
    ),
    /// `expression ('=' | '+=' | '-=' | '*=' | '/=' | '%=' | '^=' | '&&=' | '||=') expression ';'`
    ///
    /// The assigner must be one of the following:
    /// - `identifier`
    /// - `expression_access` where the accessed is an extern
    /// - `expression_index` where the indexed is an extern
    Assign(
        Box<Expression<'s>>,
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Token<'s>>,
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
        Box<Token<'s>>,
        Box<Token<'s>>,
        Option<Vec<Token<'s>>>,
        Box<Expression<'s>>,
    ),
    /// `return expression;` or `return;`
    ///
    /// If the expression is omitted, the return value is `nil`.
    Return(Box<Token<'s>>, Option<Box<Expression<'s>>>, Box<Token<'s>>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(Box<Token<'s>>, Option<Box<Expression<'s>>>, Box<Token<'s>>),
    /// `continue;`
    Continue(Box<Token<'s>>, Box<Token<'s>>),
    /// Unknown statement.
    Unknown {
        tokens: Vec<Token<'s>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s> Statement<'s> {
    pub(crate) fn unknown<T: Into<Vec<Token<'s>>>>(tokens: T, error: DiagnosticCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Statement::Unknown {
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }

    pub(crate) fn unknown_range<T: Into<Vec<Token<'s>>>>(
        tokens: T,
        error_range: SourceRange,
        error: DiagnosticCode,
    ) -> Self {
        Statement::Unknown {
            tokens: tokens.into(),
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'s>>>, E: Into<Vec<SourceDiagnostic>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Statement::Unknown {
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl<'s> AstWalker<'s> for Statement<'s> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        use Statement::*;
        visitor.visit_statement(self);
        match self {
            Empty(c) => c.walk_mut(visitor),
            Expression(expr, c) => {
                expr.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            BlockExpression(expr) => expr.walk_mut(visitor),
            Bind(kw_let, pattern, eq, expr, c) => {
                kw_let.walk_mut(visitor);
                pattern.walk_mut(visitor);
                eq.walk_mut(visitor);
                expr.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            Rebind(pattern, eq, expr, c) => {
                pattern.walk_mut(visitor);
                eq.walk_mut(visitor);
                expr.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            Assign(exp, eq, expr, c) => {
                exp.walk_mut(visitor);
                eq.walk_mut(visitor);
                expr.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            Function(kw, id, params, body) => {
                kw.walk_mut(visitor);
                id.walk_mut(visitor);
                params.walk_mut(visitor);
                body.walk_mut(visitor);
            }
            Return(kw, expr, c) => {
                kw.walk_mut(visitor);
                expr.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            Break(kw, expr, c) => {
                kw.walk_mut(visitor);
                expr.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            Continue(kw, c) => {
                kw.walk_mut(visitor);
                c.walk_mut(visitor);
            }
            Unknown { tokens, errors: _ } => {
                tokens.walk_mut(visitor);
            }
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
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

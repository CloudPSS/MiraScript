use std::fmt::Display;

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{Expression, Range};

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayInitElement<'a> {
    Expression(Box<Expression<'a>>),
    Range(Box<Range<'a>>),
    Spread(Box<Token<'a>>, Box<Expression<'a>>),
}

impl Display for ArrayInitElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for ArrayInitElement<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            ArrayInitElement::Expression(expr) => expr.fmt_ident(f, ident),
            ArrayInitElement::Range(range) => range.fmt_ident(f, ident),
            ArrayInitElement::Spread(spread, expr) => {
                write!(f, "{spread}")?;
                expr.fmt_ident(f, ident)
            }
        }
    }
}

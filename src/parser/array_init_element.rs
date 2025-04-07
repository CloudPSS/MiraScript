use std::fmt::Display;

use super::{Expression, Range, display_ident::DisplayIdent};

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayInitElement<'a> {
    Expression(Box<Expression<'a>>),
    Range(Box<Range<'a>>),
    Spread(Box<Expression<'a>>),
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
            ArrayInitElement::Spread(expr) => {
                write!(f, "..")?;
                expr.fmt_ident(f, ident)
            }
        }
    }
}

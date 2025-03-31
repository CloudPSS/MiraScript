use std::fmt::Display;

use super::{Expression, Range};

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayInitElement<'a> {
    Expression(Box<Expression<'a>>),
    Range(Box<Range<'a>>),
    Spread(Box<Expression<'a>>),
}

impl Display for ArrayInitElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayInitElement::Expression(expr) => write!(f, "{}", expr),
            ArrayInitElement::Range(range) => write!(f, "{}", range),
            ArrayInitElement::Spread(expr) => write!(f, "..{}", expr),
        }
    }
}

use std::fmt::Display;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Range<'a> {
    /// `..`
    Inclusive(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `..<`
    RightExclusive(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `>..`
    LeftExclusive(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `>..<`
    Exclusive(Box<Expression<'a>>, Box<Expression<'a>>),
}

impl Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::Inclusive(start, end) => write!(f, "{}..{}", start, end),
            Range::RightExclusive(start, end) => write!(f, "{}..<{}", start, end),
            Range::LeftExclusive(start, end) => write!(f, "{}>..{}", start, end),
            Range::Exclusive(start, end) => write!(f, "{}>..<{}", start, end),
        }
    }
}

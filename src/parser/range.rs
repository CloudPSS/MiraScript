use std::fmt::Display;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Range<'a> {
    /// `..`
    Closed(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `..<`
    HalfOpen(Box<Expression<'a>>, Box<Expression<'a>>),
}

impl Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::Closed(start, end) => write!(f, "{}..{}", start, end),
            Range::HalfOpen(start, end) => write!(f, "{}..<{}", start, end),
        }
    }
}

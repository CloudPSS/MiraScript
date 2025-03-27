use std::fmt::Display;

use super::{Expression, Range};

#[derive(Debug, Clone, PartialEq)]
pub enum Iterable<'a> {
    Range(Box<Range<'a>>),
    Value(Box<Expression<'a>>),
}

impl Display for Iterable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Iterable::Range(range) => write!(f, "{}", range),
            Iterable::Value(value) => write!(f, "{}", value),
        }
    }
}

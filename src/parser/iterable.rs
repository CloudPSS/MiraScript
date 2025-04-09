use std::fmt::Display;

use crate::ansi::DisplayIdent;

use super::{Expression, Range};

#[derive(Debug, Clone, PartialEq)]
pub enum Iterable<'a> {
    Range(Box<Range<'a>>),
    Value(Box<Expression<'a>>),
}

impl Display for Iterable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Iterable<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Iterable::Range(range) => range.fmt_ident(f, ident),
            Iterable::Value(value) => value.fmt_ident(f, ident),
        }
    }
}

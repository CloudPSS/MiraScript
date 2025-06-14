use std::fmt::Display;

use crate::ansi::DisplayIdent;

use super::{AstVisitor, AstVisitorMut, AstWalker, prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub enum Iterable<'s> {
    Range(Box<Range<'s>>),
    Value(Box<Expression<'s>>),
}

impl<'s> AstWalker<'s> for Iterable<'s> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        match self {
            Iterable::Range(range) => range.walk_mut(visitor),
            Iterable::Value(value) => value.walk_mut(visitor),
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Iterable::Range(range) => range.walk(visitor),
            Iterable::Value(value) => value.walk(visitor),
        }
    }
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

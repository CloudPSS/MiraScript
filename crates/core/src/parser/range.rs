use std::fmt::Display;

use crate::{
    ansi::{DisplayIdent, RANGE, RESET},
    lexer::{Operator, Token},
};

use super::{AstVisitor, AstVisitorMut, AstWalker, Expression};

/// A range expression.
///
/// `start..end` or `start..<end`
#[derive(Debug, Clone, PartialEq)]
pub struct Range<'s>(
    pub Box<Expression<'s>>,
    pub Box<Token<'s>>,
    pub Box<Expression<'s>>,
);

impl<'s> Range<'s> {
    pub fn exclusive(&self) -> bool {
        *self.1.as_ref() == Operator::HalfOpenRange
    }
}

impl<'s> AstWalker<'s> for Range<'s> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        let Range(start, op, end) = self;
        start.walk_mut(visitor);
        op.walk_mut(visitor);
        end.walk_mut(visitor);
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        let Range(start, op, end) = self;
        start.walk(visitor);
        op.walk(visitor);
        end.walk(visitor);
    }
    fn range(&self) -> crate::diagnostic::SourceRange {
        self.0.range().start..self.2.range().end
    }
}

impl Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Range<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        let Range(start, op, end) = self;
        start.fmt_ident(f, ident)?;
        write!(f, "{RANGE}{op}{RESET}")?;
        end.fmt_ident(f, ident)
    }
}

use std::fmt::Display;

use crate::{
    ansi::{DisplayIdent, RANGE, RESET},
    lexer::Token,
};

use super::{AstVisitor, AstWalker, Expression};

/// A range expression.
///
/// `start..end` or `start..<end`
#[derive(Debug, Clone, PartialEq)]
pub struct Range<'s>(
    pub Box<Expression<'s>>,
    pub Box<Token<'s>>,
    pub Box<Expression<'s>>,
);

impl<'s> AstWalker<'s> for Range<'s> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        let Range(start, op, end) = self;
        start.walk(visitor);
        op.walk(visitor);
        end.walk(visitor);
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

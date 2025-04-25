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
pub struct Range<'a>(
    pub Box<Expression<'a>>,
    pub Box<Token<'a>>,
    pub Box<Expression<'a>>,
);

impl<'a> AstWalker<'a> for Range<'a> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
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

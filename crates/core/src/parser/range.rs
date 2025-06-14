use std::fmt::Display;

use crate::ansi::{DisplayIdent, RANGE, RESET};

use super::{AstVisitor,  AstWalker, prelude::*};

/// A range expression.
///
/// `start..end` or `start..<end`
#[derive(Debug, Clone, PartialEq)]
pub struct Range<'s>(
    pub Box<Expression<'s>>,
    pub TokenRef<'s>,
    pub Box<Expression<'s>>,
);

impl<'s> Range<'s> {
    pub fn exclusive(&self) -> bool {
        *self.1.as_ref() == Operator::HalfOpenRange
    }
}

impl<'s> AstWalker<'s> for Range<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        let Range(start, op, end) = self;
        start.collect_diagnostics(collector);
        op.collect_diagnostics(collector);
        end.collect_diagnostics(collector);
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

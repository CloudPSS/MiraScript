use super::{AstVisitor, AstWalker, prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub enum Iterable<'s> {
    Range(Box<Range<'s>>),
    Value(Box<Expression<'s>>),
}

impl<'s> AstWalker<'s> for Iterable<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        match self {
            Iterable::Range(range) => range.collect_diagnostics(collector),
            Iterable::Value(value) => value.collect_diagnostics(collector),
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Iterable::Range(range) => range.walk(visitor),
            Iterable::Value(value) => value.walk(visitor),
        }
    }
}

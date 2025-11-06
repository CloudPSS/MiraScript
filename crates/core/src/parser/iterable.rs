use super::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Iterable<'s> {
    Range(Range<'s>),
    Value(Expression<'s>),
}

impl<'s> AstWalker<'s> for Iterable<'s> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        match self {
            Iterable::Range(range) => range.collect_diagnostics(collector),
            Iterable::Value(value) => value.collect_diagnostics(collector),
        }
    }
    fn range(&self) -> SourceRange {
        match self {
            Iterable::Range(range) => range.range(),
            Iterable::Value(value) => value.range(),
        }
    }
}

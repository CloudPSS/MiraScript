use super::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Iterable<'s, 'a> {
    Range(Range<'s, 'a>),
    Value(Expression<'s, 'a>),
}

impl<'s, 'a> AstWalker<'s> for Iterable<'s, 'a> {
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

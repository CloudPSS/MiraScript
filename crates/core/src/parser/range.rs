use super::prelude::*;

/// A range expression.
///
/// `start..end` or `start..<end`
#[derive(Debug, PartialEq)]
pub struct Range<'s, 'a>(
    pub ABox<'a, Expression<'s, 'a>>,
    pub TokenRef<'s>,
    pub ABox<'a, Expression<'s, 'a>>,
);

impl<'s, 'a> Range<'s, 'a> {
    pub fn exclusive(&self) -> bool {
        *self.1.as_ref() == Operator::HalfOpenRange
    }
}

impl<'s, 'a> AstWalker<'s> for Range<'s, 'a> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        let Range(start, op, end) = self;
        start.collect_diagnostics(collector);
        op.collect_diagnostics(collector);
        end.collect_diagnostics(collector);
    }
    fn range(&self) -> crate::diagnostic::SourceRange {
        self.0.range().start..self.2.range().end
    }
}

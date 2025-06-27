use super::prelude::*;

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
    fn range(&self) -> crate::diagnostic::SourceRange {
        self.0.range().start..self.2.range().end
    }
}

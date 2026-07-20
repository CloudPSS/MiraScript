use std::ops::Deref;

use crate::diagnostic::DiagnosticsCollector;

use super::prelude::*;

/// statement* expression? EOF
///
/// A script is a source file that contains a sequence of statements and an optional expression.
#[derive(Debug, PartialEq)]
pub struct Script<'s, 'a>(
    pub Vec<Statement<'s, 'a>>,
    pub Option<ABox<'a, Expression<'s, 'a>>>,
    pub TokenRef<'s>,
);

impl<'s, 'a> AstWalker<'s> for Script<'s, 'a> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        for statement in &mut self.0 {
            statement.collect_diagnostics(collector);
        }
        if let Some(expression) = &mut self.1 {
            expression.collect_diagnostics(collector);
        }
        self.2.collect_diagnostics(collector);
    }
    fn range(&self) -> SourceRange {
        match (self.0.as_slice(), self.1.as_deref(), self.2.as_ref()) {
            ([], None, eof) => eof.range(),
            ([], Some(expr), eof) => expr.range().start..eof.range.end,
            ([s, ..], _, eof) => s.range().start..eof.range.end,
        }
    }
}

impl<'s, 'a> Deref for Script<'s, 'a> {
    type Target = Vec<Statement<'s, 'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

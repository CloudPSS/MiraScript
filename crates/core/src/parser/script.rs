use std::ops::Deref;

use super::{AstVisitor, AstWalker, prelude::*};

/// statement* expression? EOF
///
/// A script is a source file that contains a sequence of statements and an optional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Script<'s>(
    pub Vec<Statement<'s>>,
    pub Option<Box<Expression<'s>>>,
    pub TokenRef<'s>,
);

impl<'s> AstWalker<'s> for Script<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        for statement in &mut self.0 {
            statement.collect_diagnostics(collector);
        }
        if let Some(expression) = &mut self.1 {
            expression.collect_diagnostics(collector);
        }
        self.2.collect_diagnostics(collector);
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        for statement in &self.0 {
            statement.walk(visitor);
        }
        if let Some(expression) = &self.1 {
            expression.walk(visitor);
        }
        self.2.walk(visitor);
    }
}

impl<'s> Deref for Script<'s> {
    type Target = Vec<Statement<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::ops::{Deref, DerefMut};

use super::prelude::*;

/// item ','?
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem<'s, T>(pub Box<T>, pub Option<TokenRef<'s>>);

impl<'s, T: AstWalker<'s>> AstWalker<'s> for ListItem<'s, T> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        self.0.collect_diagnostics(collector);
        self.1.collect_diagnostics(collector);
    }
    fn range(&self) -> SourceRange {
        match self.1 {
            Some(ref tail_comma) => self.0.range().start..tail_comma.range.end,
            None => self.0.range(),
        }
    }
}

impl<'s, T> ListItem<'s, T> {
    pub fn new_with_comma(item: T, tail_comma: TokenRef<'s>) -> Self {
        Self(Box::new(item), Some(tail_comma))
    }

    pub fn new(item: T) -> Self {
        Self(Box::new(item), None)
    }

    pub fn has_tail_comma(&self) -> bool {
        self.tail_comma().is_some()
    }
    pub fn tail_comma(&self) -> Option<&Token<'s>> {
        self.1.as_deref()
    }

    pub fn unwrap(self) -> T {
        *self.0
    }
}

impl<T> Deref for ListItem<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ListItem<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

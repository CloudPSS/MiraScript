use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::ansi::DisplayIdent;

use super::{AstVisitor, AstVisitorMut, AstWalker, prelude::*};

/// item ','?
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem<'s, T>(pub Box<T>, pub Option<Box<Token<'s>>>);

impl<'s, T: AstWalker<'s>> AstWalker<'s> for ListItem<'s, T> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        self.0.walk_mut(visitor);
        self.1.walk_mut(visitor);
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        self.0.walk(visitor);
        self.1.walk(visitor);
    }
}

impl<'s, T> ListItem<'s, T> {
    pub fn new_with_comma(item: T, tail_comma: Token<'s>) -> Self {
        Self(Box::new(item), Some(Box::new(tail_comma)))
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

impl<'s, T> Display for ListItem<'s, T>
where
    ListItem<'s, T>: DisplayIdent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<T: DisplayIdent> DisplayIdent for ListItem<'_, T> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        self.0.fmt_ident(f, ident)?;
        if let Some(tail_comma) = self.tail_comma() {
            write!(f, "{} ", tail_comma)?;
        }
        Ok(())
    }
}

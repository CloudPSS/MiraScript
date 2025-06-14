use std::fmt::Display;

use crate::ansi::DisplayIdent;

use super::{AstVisitor, AstVisitorMut, AstWalker, list_item::ListItem, prelude::*};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum ArrayElementBase<'s, E> {
    /// Element
    Element(Box<E>),
    /// Range
    Range(Box<super::Range<'s>>),
    /// `..` Spread
    Spread(Box<Token<'s>>, Box<E>),
}

use ArrayElementBase::*;

pub type ArrayElement<'s> = ListItem<'s, ArrayElementBase<'s, Expression<'s>>>;

pub type ArrayPattern<'s> = ListItem<'s, ArrayElementBase<'s, Pattern<'s>>>;

impl<'s, E: AstWalker<'s>> AstWalker<'s> for ArrayElementBase<'s, E> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        match self {
            Element(value) => {
                value.walk_mut(visitor);
            }
            ArrayElementBase::Range(range) => {
                range.walk_mut(visitor);
            }
            Spread(sp, value) => {
                sp.walk_mut(visitor);
                value.walk_mut(visitor);
            }
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Element(value) => {
                value.walk(visitor);
            }
            ArrayElementBase::Range(range) => {
                range.walk(visitor);
            }
            Spread(sp, value) => {
                sp.walk(visitor);
                value.walk(visitor);
            }
        }
    }
}

impl<'s, E> Display for ArrayElementBase<'s, E>
where
    ArrayElementBase<'s, E>: DisplayIdent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<E: DisplayIdent> DisplayIdent for ArrayElementBase<'_, E> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Element(value) => {
                value.fmt_ident(f, ident)?;
            }
            ArrayElementBase::Range(range) => {
                range.fmt_ident(f, ident)?;
            }
            Spread(sp, value) => {
                write!(f, "{sp}")?;
                value.fmt_ident(f, ident)?;
            }
        }
        Ok(())
    }
}

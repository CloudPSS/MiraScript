use std::fmt::Display;

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{AstVisitor, AstWalker, Expression, Pattern, Range};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum ArrayElementBase<'s, E: Clone + PartialEq> {
    /// Element `,`?
    Element(Box<E>, Option<Box<Token<'s>>>),
    /// Range `,`?
    Range(Box<Range<'s>>, Option<Box<Token<'s>>>),
    /// `..` Spread `,`?
    Spread(Box<Token<'s>>, Box<E>, Option<Box<Token<'s>>>),
}

use ArrayElementBase::*;

pub type ArrayElement<'s> = ArrayElementBase<'s, Expression<'s>>;

pub type ArrayPattern<'s> = ArrayElementBase<'s, Pattern<'s>>;

impl<'s, E: Clone + PartialEq + AstWalker<'s>> AstWalker<'s> for ArrayElementBase<'s, E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Element(value, c) => {
                value.walk(visitor);
                c.walk(visitor);
            }
            ArrayElementBase::Range(range, c) => {
                range.walk(visitor);
                c.walk(visitor);
            }
            Spread(sp, value, c) => {
                sp.walk(visitor);
                value.walk(visitor);
                c.walk(visitor);
            }
        }
    }
}

impl<'s, E: Clone + PartialEq> ArrayElementBase<'s, E> {
    pub fn has_tail_comma(&self) -> bool {
        self.tail_comma().is_some()
    }
    pub fn tail_comma(&self) -> Option<&Token<'s>> {
        match self {
            Element(.., tail_comma)
            | ArrayElementBase::Range(.., tail_comma)
            | Spread(.., tail_comma) => tail_comma.as_deref(),
        }
    }
    pub(super) fn set_tail_comma(&mut self, token: Box<Token<'s>>) {
        match self {
            Element(.., tail_comma)
            | ArrayElementBase::Range(.., tail_comma)
            | Spread(.., tail_comma) => *tail_comma = Some(token),
        }
    }
}

impl<'s, E: Clone + PartialEq> Display for ArrayElementBase<'s, E>
where
    ArrayElementBase<'s, E>: DisplayIdent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<E: DisplayIdent + Clone + PartialEq> DisplayIdent for ArrayElementBase<'_, E> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Element(value, _) => {
                value.fmt_ident(f, ident)?;
            }
            ArrayElementBase::Range(range, _) => {
                range.fmt_ident(f, ident)?;
            }
            Spread(sp, value, _) => {
                write!(f, "{sp}")?;
                value.fmt_ident(f, ident)?;
            }
        }
        if let Some(tail_comma) = self.tail_comma() {
            write!(f, "{} ", tail_comma)?;
        }
        Ok(())
    }
}

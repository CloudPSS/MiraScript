use std::fmt::Display;

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{AstVisitor, AstWalker, Expression, Pattern, Range};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum ArrayElementBase<'a, E: Clone + PartialEq> {
    /// Element `,`?
    Element(Box<E>, Option<Box<Token<'a>>>),
    /// Range `,`?
    Range(Box<Range<'a>>, Option<Box<Token<'a>>>),
    /// `..` Spread `,`?
    Spread(Box<Token<'a>>, Box<E>, Option<Box<Token<'a>>>),
}

use ArrayElementBase::*;

pub type ArrayElement<'a> = ArrayElementBase<'a, Expression<'a>>;

pub type ArrayPattern<'a> = ArrayElementBase<'a, Pattern<'a>>;

impl<'a, E: Clone + PartialEq + AstWalker<'a>> AstWalker<'a> for ArrayElementBase<'a, E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
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

impl<'a, E: Clone + PartialEq> ArrayElementBase<'a, E> {
    pub fn has_tail_comma(&self) -> bool {
        self.tail_comma().is_some()
    }
    pub fn tail_comma(&self) -> Option<&Token<'a>> {
        match self {
            Element(.., tail_comma)
            | ArrayElementBase::Range(.., tail_comma)
            | Spread(.., tail_comma) => tail_comma.as_deref(),
        }
    }
    pub(super) fn set_tail_comma(&mut self, token: Box<Token<'a>>) {
        match self {
            Element(.., tail_comma)
            | ArrayElementBase::Range(.., tail_comma)
            | Spread(.., tail_comma) => *tail_comma = Some(token),
        }
    }
}

impl<'a, E: Clone + PartialEq> Display for ArrayElementBase<'a, E>
where
    ArrayElementBase<'a, E>: DisplayIdent,
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

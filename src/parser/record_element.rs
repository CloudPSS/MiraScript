use std::fmt::{Debug, Display};

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{AstVisitor, AstWalker, Expression, Pattern};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum RecordElementBase<'s, E: Clone + PartialEq> {
    /// name colon Named `,`?
    Named(
        Box<Token<'s>>,
        Box<Token<'s>>,
        Box<E>,
        Option<Box<Token<'s>>>,
    ),
    /// colon OmitNamed `,`?
    OmitNamed(Box<Token<'s>>, Box<E>, Option<Box<Token<'s>>>),
    /// Unnamed `,`?
    Unnamed(Box<E>, Option<Box<Token<'s>>>),
    /// `..` Spread `,`?
    Spread(Box<Token<'s>>, Box<E>, Option<Box<Token<'s>>>),
}

use RecordElementBase::*;

pub type RecordElement<'s> = RecordElementBase<'s, Expression<'s>>;

pub type RecordPattern<'s> = RecordElementBase<'s, Pattern<'s>>;

impl<'s, E: Clone + PartialEq> RecordElementBase<'s, E> {
    pub fn has_tail_comma(&self) -> bool {
        self.tail_comma().is_some()
    }
    pub fn tail_comma(&self) -> Option<&Token<'s>> {
        match self {
            Named(.., tail_comma)
            | OmitNamed(.., tail_comma)
            | Unnamed(.., tail_comma)
            | Spread(.., tail_comma) => tail_comma.as_deref(),
        }
    }
    pub(super) fn set_tail_comma(&mut self, token: Box<Token<'s>>) {
        match self {
            Named(.., tail_comma)
            | OmitNamed(.., tail_comma)
            | Unnamed(.., tail_comma)
            | Spread(.., tail_comma) => *tail_comma = Some(token),
        }
    }
}

impl<'s, E: Clone + PartialEq + AstWalker<'s>> AstWalker<'s> for RecordElementBase<'s, E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Named(name, colon, value, tail_comma) => {
                name.walk(visitor);
                colon.walk(visitor);
                value.walk(visitor);
                tail_comma.walk(visitor);
            }
            OmitNamed(colon, value, tail_comma) => {
                colon.walk(visitor);
                value.walk(visitor);
                tail_comma.walk(visitor);
            }
            Unnamed(value, tail_comma) => {
                value.walk(visitor);
                tail_comma.walk(visitor);
            }
            Spread(spread, value, tail_comma) => {
                spread.walk(visitor);
                value.walk(visitor);
                tail_comma.walk(visitor);
            }
        }
    }
}

impl<'s, E: Clone + PartialEq> Display for RecordElementBase<'s, E>
where
    RecordElementBase<'s, E>: DisplayIdent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<E: DisplayIdent + Clone + PartialEq> DisplayIdent for RecordElementBase<'_, E> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Named(name, colon, value, _) => {
                write!(f, "{name}{colon} ")?;
                value.fmt_ident(f, ident)?;
            }
            OmitNamed(colon, value, _) => {
                write!(f, "{colon}")?;
                value.fmt_ident(f, ident)?;
            }
            Unnamed(value, _) => value.fmt_ident(f, ident)?,
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

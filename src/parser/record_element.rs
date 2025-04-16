use std::fmt::{Debug, Display};

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{Expression, Pattern};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordElementBase<'a, E: Clone + PartialEq> {
    /// name colon Named `,`?
    Named(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<E>,
        Option<Box<Token<'a>>>,
    ),
    /// colon OmitNamed `,`?
    OmitNamed(Box<Token<'a>>, Box<E>, Option<Box<Token<'a>>>),
    /// Unnamed `,`?
    Unnamed(Box<E>, Option<Box<Token<'a>>>),
    /// `..` Spread `,`?
    Spread(Box<Token<'a>>, Box<E>, Option<Box<Token<'a>>>),
}

use RecordElementBase::*;

pub type RecordElement<'a> = RecordElementBase<'a, Expression<'a>>;

pub type RecordPattern<'a> = RecordElementBase<'a, Pattern<'a>>;

impl<'a, E: Clone + PartialEq> RecordElementBase<'a, E> {
    pub fn is_named(&self) -> bool {
        matches!(self, Named(..))
    }
    pub fn is_omit_named(&self) -> bool {
        matches!(self, OmitNamed(..))
    }
    pub fn is_unnamed(&self) -> bool {
        matches!(self, Unnamed(..))
    }
    pub fn is_spread(&self) -> bool {
        matches!(self, Spread(..))
    }
    pub fn has_tail_comma(&self) -> bool {
        self.tail_comma().is_some()
    }
    pub fn tail_comma(&self) -> Option<&Token<'a>> {
        match self {
            Named(.., tail_comma)
            | OmitNamed(.., tail_comma)
            | Unnamed(.., tail_comma)
            | Spread(.., tail_comma) => tail_comma.as_deref(),
        }
    }
    pub(super) fn set_tail_comma(&mut self, token: Box<Token<'a>>) {
        match self {
            Named(.., tail_comma)
            | OmitNamed(.., tail_comma)
            | Unnamed(.., tail_comma)
            | Spread(.., tail_comma) => *tail_comma = Some(token),
        }
    }
}

impl<'a, E: Clone + PartialEq> Display for RecordElementBase<'a, E>
where
    RecordElementBase<'a, E>: DisplayIdent,
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

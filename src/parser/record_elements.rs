use std::fmt::{Debug, Display};

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{Expression, Pattern};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordElementBase<
    'a,
    Named: DisplayIdent + Clone + PartialEq,
    OmitNamed: DisplayIdent + Clone + PartialEq,
    Unnamed: DisplayIdent + Clone + PartialEq,
    Spread: DisplayIdent + Clone + PartialEq,
> {
    /// name `:` Named `,`?
    Named(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<Named>,
        Option<Box<Token<'a>>>,
    ),
    /// `:` OmitNamed `,`?
    OmitNamed(Box<Token<'a>>, Box<OmitNamed>, Option<Box<Token<'a>>>),
    /// Unnamed `,`?
    Unnamed(Box<Unnamed>, Option<Box<Token<'a>>>),
    /// `..` Spread `,`?
    Spread(Box<Token<'a>>, Box<Spread>, Option<Box<Token<'a>>>),
}

use RecordElementBase::*;

pub type RecordElement<'a> =
    RecordElementBase<'a, Expression<'a>, Token<'a>, Expression<'a>, Expression<'a>>;

pub type RecordPattern<'a> =
    RecordElementBase<'a, Pattern<'a>, Pattern<'a>, Pattern<'a>, Pattern<'a>>;

impl<
    'a,
    Named: DisplayIdent + Clone + PartialEq,
    OmitNamed: DisplayIdent + Clone + PartialEq,
    Unnamed: DisplayIdent + Clone + PartialEq,
    Spread: DisplayIdent + Clone + PartialEq,
> RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>
{
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

impl<
    'a,
    Named: DisplayIdent + Clone + PartialEq,
    OmitNamed: DisplayIdent + Clone + PartialEq,
    Unnamed: DisplayIdent + Clone + PartialEq,
    Spread: DisplayIdent + Clone + PartialEq,
> Display for RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<
    'a,
    Named: DisplayIdent + Clone + PartialEq,
    OmitNamed: DisplayIdent + Clone + PartialEq,
    Unnamed: DisplayIdent + Clone + PartialEq,
    Spread: DisplayIdent + Clone + PartialEq,
> DisplayIdent for RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>
{
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Named(name, colon, pattern, _) => {
                write!(f, "{name}{colon} ")?;
                pattern.fmt_ident(f, ident)?;
            }
            OmitNamed(colon, pattern, _) => {
                write!(f, "{colon}")?;
                pattern.fmt_ident(f, ident)?;
            }
            Unnamed(pattern, _) => pattern.fmt_ident(f, ident)?,
            Spread(sp, pattern, _) => {
                write!(f, "{sp}")?;
                pattern.fmt_ident(f, ident)?;
            }
        }
        if let Some(tail_comma) = self.tail_comma() {
            write!(f, "{} ", tail_comma)?;
        }
        Ok(())
    }
}

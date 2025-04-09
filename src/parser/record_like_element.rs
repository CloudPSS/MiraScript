use std::fmt::Display;

use crate::lexer::Token;

use super::{Expression, display_ident::DisplayIdent};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordLikeElement<'a> {
    /// `name: value [,]`
    Named(Box<Token<'a>>, Box<Expression<'a>>, Option<Box<Token<'a>>>),
    /// `:identifier [,]`
    OmitNamed(Box<Token<'a>>, Option<Box<Token<'a>>>),
    /// `value [,]`
    Unnamed(Box<Expression<'a>>, Option<Box<Token<'a>>>),
    /// `..value [,]`
    Spread(Box<Token<'a>>, Box<Expression<'a>>, Option<Box<Token<'a>>>),
}

impl<'a> RecordLikeElement<'a> {
    pub fn is_named(&self) -> bool {
        matches!(self, RecordLikeElement::Named(_, _, _))
    }
    pub fn is_omit_named(&self) -> bool {
        matches!(self, RecordLikeElement::OmitNamed(_, _))
    }
    pub fn is_unnamed(&self) -> bool {
        matches!(self, RecordLikeElement::Unnamed(_, _))
    }
    pub fn is_spread(&self) -> bool {
        matches!(self, RecordLikeElement::Spread(_, _, _))
    }
    pub fn has_tail_comma(&self) -> bool {
        match self {
            RecordLikeElement::Named(_, _, tail_comma)
            | RecordLikeElement::OmitNamed(_, tail_comma)
            | RecordLikeElement::Unnamed(_, tail_comma)
            | RecordLikeElement::Spread(_, _, tail_comma) => tail_comma.is_some(),
        }
    }
    pub fn tail_comma(&self) -> Option<&Token<'a>> {
        match self {
            RecordLikeElement::Named(_, _, tail_comma)
            | RecordLikeElement::OmitNamed(_, tail_comma)
            | RecordLikeElement::Unnamed(_, tail_comma)
            | RecordLikeElement::Spread(_, _, tail_comma) => tail_comma.as_deref(),
        }
    }
}

impl Display for RecordLikeElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for RecordLikeElement<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            RecordLikeElement::Named(name, value, _) => {
                write!(f, "{name}: ")?;
                value.fmt_ident(f, ident)?;
            }
            RecordLikeElement::OmitNamed(name, _) => {
                write!(f, ":{name}")?;
            }
            RecordLikeElement::Unnamed(value, _) => {
                value.fmt_ident(f, ident)?;
            }
            RecordLikeElement::Spread(spread, value, _) => {
                write!(f, "{spread}")?;
                value.fmt_ident(f, ident)?;
            }
        }
        if let Some(tail_comma) = self.tail_comma() {
            write!(f, "{} ", tail_comma)?;
        }
        Ok(())
    }
}

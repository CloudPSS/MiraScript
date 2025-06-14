use std::fmt::{Debug, Display};

use crate::ansi::DisplayIdent;

use super::{AstVisitor,  AstWalker, list_item::ListItem, prelude::*};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum RecordElementBase<'s, E, I> {
    /// name colon Named
    Named(Box<Token<'s>>, Box<Token<'s>>, Box<E>),
    /// interpolated_string colon Named
    InterpolateNamed(Box<I>, Box<Token<'s>>, Box<E>),
    /// colon OmitNamed
    OmitNamed(Box<Token<'s>>, Box<E>),
    /// Unnamed
    Unnamed(Box<E>),
    /// `..` Spread
    Spread(Box<Token<'s>>, Box<E>),
}

use RecordElementBase::*;

pub type RecordElement<'s> = ListItem<'s, RecordElementBase<'s, Expression<'s>, Expression<'s>>>;

pub type RecordPattern<'s> = ListItem<'s, RecordElementBase<'s, Pattern<'s>, Pattern<'s>>>;

impl<'s, E, I> RecordElementBase<'s, E, I> {
    pub fn colon(&self) -> Option<&Token<'s>> {
        match self {
            Named(_, colon, _) | InterpolateNamed(_, colon, _) | OmitNamed(colon, _) => Some(colon),
            Unnamed(_) | Spread(_, _) => None,
        }
    }
}

impl<'s, E: AstWalker<'s>, I: AstWalker<'s>> AstWalker<'s> for RecordElementBase<'s, E, I> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        match self {
            Named(name, colon, value) => {
                name.collect_diagnostics(collector);
                colon.collect_diagnostics(collector);
                value.collect_diagnostics(collector);
            }
            InterpolateNamed(name_expr, colon, value) => {
                name_expr.collect_diagnostics(collector);
                colon.collect_diagnostics(collector);
                value.collect_diagnostics(collector);
            }
            OmitNamed(colon, value) => {
                colon.collect_diagnostics(collector);
                value.collect_diagnostics(collector);
            }
            Unnamed(value) => {
                value.collect_diagnostics(collector);
            }
            Spread(spread, value) => {
                spread.collect_diagnostics(collector);
                value.collect_diagnostics(collector);
            }
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Named(name, colon, value) => {
                name.walk(visitor);
                colon.walk(visitor);
                value.walk(visitor);
            }
            InterpolateNamed(name_expr, colon, value) => {
                name_expr.walk(visitor);
                colon.walk(visitor);
                value.walk(visitor);
            }
            OmitNamed(colon, value) => {
                colon.walk(visitor);
                value.walk(visitor);
            }
            Unnamed(value) => {
                value.walk(visitor);
            }
            Spread(spread, value) => {
                spread.walk(visitor);
                value.walk(visitor);
            }
        }
    }
}

impl<'s, E, I> Display for RecordElementBase<'s, E, I>
where
    RecordElementBase<'s, E, I>: DisplayIdent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<E: DisplayIdent, I: DisplayIdent> DisplayIdent for RecordElementBase<'_, E, I> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Named(name, colon, value) => {
                name.fmt_ident(f, ident)?;
                colon.fmt_ident(f, ident)?;
                write!(f, " ")?;
                value.fmt_ident(f, ident)?;
            }
            InterpolateNamed(name_expr, colon, value) => {
                name_expr.fmt_ident(f, ident)?;
                colon.fmt_ident(f, ident)?;
                write!(f, " ")?;
                value.fmt_ident(f, ident)?;
            }
            OmitNamed(colon, value) => {
                colon.fmt_ident(f, ident)?;
                value.fmt_ident(f, ident)?;
            }
            Unnamed(value) => value.fmt_ident(f, ident)?,
            Spread(sp, value) => {
                sp.fmt_ident(f, ident)?;
                value.fmt_ident(f, ident)?;
            }
        }
        Ok(())
    }
}

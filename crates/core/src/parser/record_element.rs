use std::fmt::{Debug, Display};

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{AstVisitor, AstVisitorMut, AstWalker, Expression, Pattern};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum RecordElementBase<'s, E: Clone + PartialEq, CE: Clone + PartialEq> {
    /// name colon Named `,`?
    Named(
        Box<Token<'s>>,
        Box<Token<'s>>,
        Box<E>,
        Option<Box<Token<'s>>>,
    ),
    /// `[` name_expr `]` colon Named `,`?
    CalculatedNamed(
        Box<Token<'s>>,
        Box<CE>,
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

pub type RecordElement<'s> = RecordElementBase<'s, Expression<'s>, Expression<'s>>;

pub type RecordPattern<'s> = RecordElementBase<'s, Pattern<'s>, Pattern<'s>>;

impl<'s, E: Clone + PartialEq, CE: Clone + PartialEq> RecordElementBase<'s, E, CE> {
    pub fn has_tail_comma(&self) -> bool {
        self.tail_comma().is_some()
    }
    pub fn tail_comma(&self) -> Option<&Token<'s>> {
        match self {
            Named(.., tail_comma)
            | CalculatedNamed(.., tail_comma)
            | OmitNamed(.., tail_comma)
            | Unnamed(.., tail_comma)
            | Spread(.., tail_comma) => tail_comma.as_deref(),
        }
    }
    pub(super) fn set_tail_comma(&mut self, token: Box<Token<'s>>) {
        match self {
            Named(.., tail_comma)
            | CalculatedNamed(.., tail_comma)
            | OmitNamed(.., tail_comma)
            | Unnamed(.., tail_comma)
            | Spread(.., tail_comma) => *tail_comma = Some(token),
        }
    }
}

impl<'s, E: Clone + PartialEq + AstWalker<'s>, CE: Clone + PartialEq + AstWalker<'s>> AstWalker<'s>
    for RecordElementBase<'s, E, CE>
{
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        match self {
            Named(name, colon, value, tail_comma) => {
                name.walk_mut(visitor);
                colon.walk_mut(visitor);
                value.walk_mut(visitor);
                tail_comma.walk_mut(visitor);
            }
            CalculatedNamed(op, name_expr, cp, colon, value, tail_comma) => {
                op.walk_mut(visitor);
                name_expr.walk_mut(visitor);
                cp.walk_mut(visitor);
                colon.walk_mut(visitor);
                value.walk_mut(visitor);
                tail_comma.walk_mut(visitor);
            }
            OmitNamed(colon, value, tail_comma) => {
                colon.walk_mut(visitor);
                value.walk_mut(visitor);
                tail_comma.walk_mut(visitor);
            }
            Unnamed(value, tail_comma) => {
                value.walk_mut(visitor);
                tail_comma.walk_mut(visitor);
            }
            Spread(spread, value, tail_comma) => {
                spread.walk_mut(visitor);
                value.walk_mut(visitor);
                tail_comma.walk_mut(visitor);
            }
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        match self {
            Named(name, colon, value, tail_comma) => {
                name.walk(visitor);
                colon.walk(visitor);
                value.walk(visitor);
                tail_comma.walk(visitor);
            }
            CalculatedNamed(op, name_expr, cp, colon, value, tail_comma) => {
                op.walk(visitor);
                name_expr.walk(visitor);
                cp.walk(visitor);
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

impl<'s, E: Clone + PartialEq, CE: Clone + PartialEq> Display for RecordElementBase<'s, E, CE>
where
    RecordElementBase<'s, E, CE>: DisplayIdent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl<E: DisplayIdent + Clone + PartialEq, CE: DisplayIdent + Clone + PartialEq> DisplayIdent
    for RecordElementBase<'_, E, CE>
{
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Named(name, colon, value, _) => {
                name.fmt_ident(f, ident)?;
                colon.fmt_ident(f, ident)?;
                write!(f, " ")?;
                value.fmt_ident(f, ident)?;
            }
            CalculatedNamed(op, name_expr, cp, colon, value, _) => {
                op.fmt_ident(f, ident)?;
                name_expr.fmt_ident(f, ident)?;
                cp.fmt_ident(f, ident)?;
                colon.fmt_ident(f, ident)?;
                write!(f, " ")?;
                value.fmt_ident(f, ident)?;
            }
            OmitNamed(colon, value, _) => {
                colon.fmt_ident(f, ident)?;
                value.fmt_ident(f, ident)?;
            }
            Unnamed(value, _) => value.fmt_ident(f, ident)?,
            Spread(sp, value, _) => {
                sp.fmt_ident(f, ident)?;
                value.fmt_ident(f, ident)?;
            }
        }
        if let Some(tail_comma) = self.tail_comma() {
            tail_comma.fmt_ident(f, ident)?;
        }
        Ok(())
    }
}

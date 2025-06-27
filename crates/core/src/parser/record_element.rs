use super::prelude::*;

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum RecordElementBase<'s, E, I> {
    /// name colon Named
    Named(TokenRef<'s>, TokenRef<'s>, Box<E>),
    /// interpolated_string colon Named
    InterpolateNamed(Box<I>, TokenRef<'s>, Box<E>),
    /// colon OmitNamed
    OmitNamed(TokenRef<'s>, Box<E>),
    /// Unnamed
    Unnamed(Box<E>),
    /// `..` Spread
    Spread(TokenRef<'s>, Box<E>),
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
    fn range(&self) -> SourceRange {
        match self {
            Named(name, _, value) => name.range.start..value.range().end,
            InterpolateNamed(name_expr, _, value) => name_expr.range().start..value.range().end,
            OmitNamed(colon, value) => colon.range.start..value.range().end,
            Unnamed(value) => value.range(),
            Spread(spread, value) => spread.range.start..value.range().end,
        }
    }
}

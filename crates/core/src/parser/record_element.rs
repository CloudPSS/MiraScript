use super::prelude::*;

#[derive(Debug, PartialEq, strum::EnumIs)]
pub enum RecordElementBase<'s, 'a, E, I> {
    /// name colon Named
    Named(TokenRef<'s>, TokenRef<'s>, ABox<'a, E>),
    /// interpolated_string colon Named
    InterpolateNamed(ABox<'a, I>, TokenRef<'s>, ABox<'a, E>),
    /// colon OmitNamed
    OmitNamed(TokenRef<'s>, ABox<'a, E>),
    /// Unnamed
    Unnamed(ABox<'a, E>),
    /// `..` Spread
    Spread(TokenRef<'s>, ABox<'a, E>),
}

use RecordElementBase::*;

pub type RecordElement<'s, 'a> =
    ListItem<'s, 'a, RecordElementBase<'s, 'a, Expression<'s, 'a>, Expression<'s, 'a>>>;

pub type RecordPattern<'s, 'a> =
    ListItem<'s, 'a, RecordElementBase<'s, 'a, Pattern<'s, 'a>, Pattern<'s, 'a>>>;

impl<'s, 'a, E, I> RecordElementBase<'s, 'a, E, I> {
    pub fn colon(&self) -> Option<&Token<'s>> {
        match self {
            Named(_, colon, _) | InterpolateNamed(_, colon, _) | OmitNamed(colon, _) => Some(colon),
            Unnamed(_) | Spread(_, _) => None,
        }
    }
}

impl<'s, 'a, E: AstWalker<'s>, I: AstWalker<'s>> AstWalker<'s> for RecordElementBase<'s, 'a, E, I> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
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

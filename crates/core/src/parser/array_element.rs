use super::prelude::*;

#[derive(Debug, PartialEq, strum::EnumIs)]
pub enum ArrayElementBase<'s, 'a, E, S> {
    /// Element
    Element(ABox<'a, E>),
    /// `..` Spread
    Spread(TokenRef<'s>, ABox<'a, S>),
}

use ArrayElementBase::*;

pub type ArrayElement<'s, 'a> = ListItem<'s, 'a, ArrayElementBase<'s, 'a, Iterable<'s, 'a>, Expression<'s, 'a>>>;

pub type ArgElement<'s, 'a> = ListItem<'s, 'a, ArrayElementBase<'s, 'a, Expression<'s, 'a>, Expression<'s, 'a>>>;

pub type ArrayPattern<'s, 'a> = ListItem<'s, 'a, ArrayElementBase<'s, 'a, Pattern<'s, 'a>, Pattern<'s, 'a>>>;

impl<'s, 'a, E: AstWalker<'s>, S: AstWalker<'s>> AstWalker<'s> for ArrayElementBase<'s, 'a, E, S> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        match self {
            Element(value) => {
                value.collect_diagnostics(collector);
            }
            Spread(sp, value) => {
                sp.collect_diagnostics(collector);
                value.collect_diagnostics(collector);
            }
        }
    }
    fn range(&self) -> SourceRange {
        match self {
            Element(value) => value.range(),
            Spread(sp, value) => sp.range.start..value.range().end,
        }
    }
}

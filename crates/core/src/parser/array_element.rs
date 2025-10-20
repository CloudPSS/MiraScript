use super::prelude::*;

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum ArrayElementBase<'s, E, S> {
    /// Element
    Element(Box<E>),
    /// `..` Spread
    Spread(TokenRef<'s>, Box<S>),
}

use ArrayElementBase::*;

pub type ArrayElement<'s> = ListItem<'s, ArrayElementBase<'s, Iterable<'s>, Expression<'s>>>;

pub type ArgElement<'s> = ListItem<'s, ArrayElementBase<'s, Expression<'s>, Expression<'s>>>;

pub type ArrayPattern<'s> = ListItem<'s, ArrayElementBase<'s, Pattern<'s>, Pattern<'s>>>;

impl<'s, E: AstWalker<'s>, S: AstWalker<'s>> AstWalker<'s> for ArrayElementBase<'s, E, S> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
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

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum ArrayElementBase<'s, E> {
    /// Element
    Element(Box<E>),
    /// Range
    Range(Box<super::Range<'s>>),
    /// `..` Spread
    Spread(TokenRef<'s>, Box<E>),
}

use ArrayElementBase::*;

pub type ArrayElement<'s> = ListItem<'s, ArrayElementBase<'s, Expression<'s>>>;

pub type ArrayPattern<'s> = ListItem<'s, ArrayElementBase<'s, Pattern<'s>>>;

impl<'s, E: AstWalker<'s>> AstWalker<'s> for ArrayElementBase<'s, E> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        match self {
            Element(value) => {
                value.collect_diagnostics(collector);
            }
            ArrayElementBase::Range(range) => {
                range.collect_diagnostics(collector);
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
            ArrayElementBase::Range(range) => range.range(),
            Spread(sp, value) => sp.range.start..value.range().end,
        }
    }
}

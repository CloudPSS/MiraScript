use winnow::{ModalResult, Parser, combinator::alt};

use super::{Input, Iterable, expressions::expression, ranges::range};

pub(super) fn iterable<'s>(i: &mut Input<'_, 's>) -> ModalResult<Iterable<'s>> {
    alt((
        range.map(|r| Iterable::Range(Box::new(r))),
        expression.map(|e| Iterable::Value(Box::new(e))),
    ))
    .parse_next(i)
}

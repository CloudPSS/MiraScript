use winnow::{ModalResult, Parser, combinator::alt};

use super::{Input, Iterable, expression, ranges::range};

pub(super) fn iterable<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Iterable<'a>> {
    alt((
        range.map(|r| Iterable::Range(Box::new(r))),
        expression.map(|e| Iterable::Value(Box::new(e))),
    ))
    .parse_next(i)
}

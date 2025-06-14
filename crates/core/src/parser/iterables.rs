use winnow::combinator::alt;

use super::{expressions::expression, prelude::*, ranges::range};

pub(super) fn iterable<'s>(i: &mut Input<'s>) -> Result<Iterable<'s>> {
    alt((
        range.map(|r| Iterable::Range(Box::new(r))),
        expression.map(|e| Iterable::Value(Box::new(e))),
    ))
    .parse_next(i)
}

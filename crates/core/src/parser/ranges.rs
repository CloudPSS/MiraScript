use winnow::{combinator::seq, token::one_of};

use super::{basic_expressions::additive, prelude::*};

pub(super) fn range<'s>(i: &mut Input<'s>) -> Result<Range<'s>> {
    seq!(Range(
        additive.map(Box::new),
        one_of(|t: &Token<'s>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(TokenRef::borrow),
        additive.map(Box::new),
    ))
    .parse_next(i)
}

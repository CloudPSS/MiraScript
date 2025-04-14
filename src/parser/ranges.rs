use winnow::{ModalResult, Parser, combinator::seq, token::one_of};

use crate::lexer::{Operator, Token};

use super::{Input, Range, basic_expressions::additive};

pub(super) fn range<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Range<'a>> {
    seq!(Range(
        additive.map(Box::new),
        one_of(|t: &Token<'a>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(|t: &Token<'a>| Box::new(t.to_owned())),
        additive.map(Box::new),
    ))
    .parse_next(i)
}

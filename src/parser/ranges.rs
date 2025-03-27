use winnow::{ModalResult, Parser, token::one_of};

use crate::lexer::{Operator, Token, TokenKind};

use super::{Input, Range, basic_expressions::term};

pub(super) fn range<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Range<'a>> {
    (
        term.map(Box::new),
        one_of(|t: &Token<'a>| {
            *t == Operator::InclusiveRange
                || *t == Operator::RightExclusiveRange
                || *t == Operator::LeftExclusiveRange
                || *t == Operator::ExclusiveRange
        }),
        term.map(Box::new),
    )
        .map(|(first, op, second)| {
            let Token {
                kind: TokenKind::Operator(op),
                ..
            } = op
            else {
                unreachable!();
            };
            match op {
                Operator::InclusiveRange => Range::Inclusive(first, second),
                Operator::RightExclusiveRange => Range::RightExclusive(first, second),
                Operator::LeftExclusiveRange => Range::LeftExclusive(first, second),
                Operator::ExclusiveRange => Range::Exclusive(first, second),
                _ => unreachable!(),
            }
        })
        .parse_next(i)
}

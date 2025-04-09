use winnow::{ModalResult, Parser, token::one_of};

use crate::lexer::{Operator, Token, TokenKind};

use super::{Input, Range, basic_expressions::additive};

pub(super) fn range<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Range<'a>> {
    (
        additive.map(Box::new),
        one_of(|t: &Token<'a>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange),
        additive.map(Box::new),
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
                Operator::SpreadRange => Range::Closed(first, second),
                Operator::HalfOpenRange => Range::HalfOpen(first, second),
                _ => unreachable!(),
            }
        })
        .parse_next(i)
}

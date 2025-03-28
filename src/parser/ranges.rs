use winnow::{ModalResult, Parser, token::one_of};

use crate::lexer::{Operator, Token, TokenKind};

use super::{Input, Range, basic_expressions::term};

pub(super) fn range<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Range<'a>> {
    (
        term.map(Box::new),
        one_of(|t: &Token<'a>| *t == Operator::ClosedRange || *t == Operator::HalfOpenRange),
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
                Operator::ClosedRange => Range::Closed(first, second),
                Operator::HalfOpenRange => Range::HalfOpen(first, second),
                _ => unreachable!(),
            }
        })
        .parse_next(i)
}

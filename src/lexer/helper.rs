use winnow::combinator::{delimited, opt, peek, repeat, terminated};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::tokenizer::{Operator, TokenKind};

use super::{Input, TokenRef};

pub(super) fn parameter_list<'a>(i: &mut Input<'a>) -> ModalResult<Option<Vec<TokenRef<'a>>>> {
    let t = peek(any).parse_next(i)?;
    if *t != Operator::OpenParen {
        return Ok(None);
    }

    delimited(
        literal(Operator::OpenParen),
        (
            repeat(
                0..,
                terminated(
                    one_of(|t: TokenRef<'a>| matches!(&t.kind, &TokenKind::Identifier(_))),
                    literal(Operator::Comma),
                ),
            )
            .fold(Vec::new, |mut v, t| {
                v.push(t);
                v
            }),
            opt(one_of(|t: TokenRef<'a>| {
                matches!(&t.kind, &TokenKind::Identifier(_))
            })),
        ),
        literal(Operator::CloseParen),
    )
    .map(|(mut v, t)| {
        if let Some(t) = t {
            v.push(t);
        }
        Some(v)
    })
    .parse_next(i)
}

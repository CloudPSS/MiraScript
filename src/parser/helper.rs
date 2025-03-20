use winnow::combinator::{delimited, opt, peek, repeat, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenError, TokenKind};

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

pub(super) fn literal_token<'a>(i: &mut Input<'a>) -> ModalResult<TokenRef<'a>> {
    one_of(|t: TokenRef<'a>| {
        matches!(&t.kind, &TokenKind::Number(_))
            || matches!(&t.kind, &TokenKind::String(_))
            || *t == Keyword::True
            || *t == Keyword::False
            || *t == Keyword::Nil
    })
    .parse_next(i)
}

pub(super) fn variable_token<'a>(
    include_underscore: bool,
) -> impl Parser<Input<'a>, TokenRef<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        let t = one_of(|t: TokenRef<'a>| {
            matches!(&t.kind, &TokenKind::Identifier(_)) || (*t == Keyword::Underscore)
        })
        .parse_next(i)?;
        let e = if !include_underscore && *t == Keyword::Underscore {
            let u = i.state.add_token(Token::unknown(
                t.range.clone(),
                t.kind.clone(),
                vec![TokenError::new(
                    t.range.clone(),
                    "Unexpected `_`, it is a reserved variable name",
                )],
            ));
            u
        } else {
            t
        };
        Ok(e)
    }
}

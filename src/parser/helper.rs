use winnow::combinator::{delimited, opt, peek, repeat, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::Input;

pub(super) fn parameter_list<'t, 'a:'t>(i: &mut Input<'t, 'a>) -> ModalResult<Option<Vec<Token<'a>>>> {
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
                    one_of(|t: &Token<'a>| matches!(&t.kind, &TokenKind::Identifier(_))),
                    literal(Operator::Comma),
                ),
            )
            .fold(Vec::new, |mut v, t: &Token<'a>| {
                v.push(t.to_owned());
                v
            }),
            opt(one_of(|t: &Token<'a>| {
                matches!(&t.kind, &TokenKind::Identifier(_))
            })),
        ),
        literal(Operator::CloseParen),
    )
    .map(|(mut v, t): (Vec<Token<'a>>, Option<&Token<'a>>)| {
        if let Some(t) = t {
            v.push(t.to_owned());
        }
        Some(v)
    })
    .parse_next(i)
}

pub(super) fn literal_token<'t, 'a:'t>(i: &mut Input<'t, 'a>) -> ModalResult<Token<'a>> {
    one_of(|t: &Token<'a>| {
        matches!(&t.kind, &TokenKind::Number(_))
            || matches!(&t.kind, &TokenKind::String(_))
            || *t == Keyword::True
            || *t == Keyword::False
            || *t == Keyword::Nil
    })
    .map(|t: &Token<'a>| t.to_owned())
    .parse_next(i)
}

pub(super) fn interpolation_token<'t, 'a:'t>(i: &mut Input<'t, 'a>) -> ModalResult<Token<'a>> {
    one_of(|t: &Token<'a>| matches!(&t.kind, &TokenKind::InterpolatedString(_, _)))
        .map(|t: &Token<'a>| t.to_owned())
        .parse_next(i)
}

pub(super) fn variable_token<'t, 'a: 't>(
    include_underscore: bool,
    include_global: bool,
) -> impl Parser<Input<'t, 'a>, Token<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'t, 'a>| {
        let t = one_of(|t: &Token<'a>| {
            matches!(&t.kind, &TokenKind::Identifier(_))
                || (*t == Keyword::Underscore)
                || (*t == Keyword::Global)
        })
        .map(|t: &Token<'a>| t.to_owned())
        .parse_next(i)?;
        let e = if !include_underscore && t == Keyword::Underscore {
            Token::unknown(
                t.range,
                t.kind,
                "Unexpected `_`, it is a reserved variable name",
            )
        } else if !include_global && t == Keyword::Global {
            Token::unknown(
                t.range,
                t.kind,
                "Unexpected `global`, it is a reserved variable name",
            )
        } else {
            t.to_owned()
        };
        Ok(e)
    }
}

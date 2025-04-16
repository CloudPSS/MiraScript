use winnow::combinator::{delimited, opt, peek, repeat, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenKind};
use crate::utils::SourceRange;

use super::{Expression, Input, expression};

pub(super) fn spread_expression<'a>(
    i: &mut Input<'_, 'a>,
) -> ModalResult<(Token<'a>, Expression<'a>)> {
    (token(Operator::SpreadRange), opt(expression))
        .map(|(spread, e)| {
            let e = if let Some(e) = e {
                e
            } else {
                Expression::unknown_range(
                    [],
                    spread.range.clone(),
                    "Expression expected after `..`",
                )
            };
            (spread, e)
        })
        .parse_next(i)
}

pub(super) fn parameter_list<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Option<Vec<Token<'a>>>> {
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

pub(super) fn literal_token<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Token<'a>> {
    one_of(|t: &Token<'a>| {
        matches!(&t.kind, &TokenKind::Number(_))
            || matches!(&t.kind, &TokenKind::Ordinal(_))
            || matches!(&t.kind, &TokenKind::String(_))
            || *t == Keyword::True
            || *t == Keyword::False
            || *t == Keyword::Nil
            || *t == Keyword::Nan
            || *t == Keyword::Inf
    })
    .map(|t: &Token<'a>| t.to_owned())
    .parse_next(i)
}

pub(super) fn variable_token<'t, 'a: 't>(
    include_underscore: bool,
    include_global: bool,
) -> impl Parser<Input<'t, 'a>, Token<'a>, ErrMode<ContextError>> + Copy {
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
                "Unexpected `_`, it is a reserved keyword for discarding",
            )
        } else if !include_global && t == Keyword::Global {
            Token::unknown(
                t.range,
                t.kind,
                "Unexpected `global`, it is a reserved keyword for global variable",
            )
        } else {
            t.to_owned()
        };
        Ok(e)
    }
}

pub(super) fn token<'t, 'a: 't>(
    token: impl Into<TokenKind<'a>> + Clone + PartialEq<Token<'a>>,
) -> impl Parser<Input<'t, 'a>, Token<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'t, 'a>| {
        one_of(|t: &Token<'a>| token == *t)
            .map(|t: &Token<'a>| t.to_owned())
            .parse_next(i)
    }
}
pub(super) fn token_boxed<'t, 'a: 't>(
    token: impl Into<TokenKind<'a>> + Clone + PartialEq<Token<'a>>,
) -> impl Parser<Input<'t, 'a>, Box<Token<'a>>, ErrMode<ContextError>> {
    move |i: &mut Input<'t, 'a>| {
        one_of(|t: &Token<'a>| token == *t)
            .map(|t: &Token<'a>| Box::new(t.to_owned()))
            .parse_next(i)
    }
}

pub(super) fn token_or_insert<'t, 'a: 't, T>(
    token: T,
    error: &'static str,
) -> impl Parser<Input<'t, 'a>, Token<'a>, ErrMode<ContextError>>
where
    T: Into<TokenKind<'a>> + Clone,
    Token<'a>: PartialEq<T>,
{
    move |i: &mut Input<'_, 'a>| -> ModalResult<Token<'a>> {
        let pos = Location::previous_token_end(i);
        opt(one_of(|t: &Token<'a>| *t == token))
            .map(|t: Option<&Token<'a>>| match t {
                Some(t) => t.to_owned(),
                None => Token::unknown(
                    SourceRange {
                        start: pos,
                        end: pos,
                    },
                    token.clone(),
                    error,
                ),
            })
            .parse_next(i)
    }
}

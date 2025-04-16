use winnow::combinator::{delimited, opt, peek, repeat, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenKind};
use crate::utils::SourceRange;

use super::statements::statement;
use super::{Expression, Input, Statement, expression};

pub(super) fn statements_and_expression<'a>(
    i: &mut Input<'_, 'a>,
) -> ModalResult<(Vec<Statement<'a>>, Option<Box<Expression<'a>>>)> {
    let (mut statements, expression): (Vec<_>, _) =
        (repeat(0.., statement), opt(expression.map(Box::new))).parse_next(i)?;
    if expression.is_some() || statements.is_empty() {
        return Ok((statements, expression));
    }

    let last_statement = statements.pop().unwrap();
    let expression = match last_statement {
        Statement::BlockExpression(e) => Some(e),
        _ => {
            statements.push(last_statement);
            None
        }
    };
    Ok((statements, expression))
}

pub(super) fn parameter_list<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Option<Vec<Token<'a>>>> {
    let t = peek(any).parse_next(i)?;
    if *t != Operator::OpenParen {
        return Ok(None);
    }

    delimited(
        token(Operator::OpenParen),
        (
            repeat(
                0..,
                terminated(
                    one_of(|t: &Token<'a>| matches!(&t.kind, &TokenKind::Identifier(_))),
                    token(Operator::Comma),
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
        token(Operator::CloseParen),
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

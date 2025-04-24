use winnow::{
    ModalResult, Parser,
    combinator::{alt, fail, peek, repeat},
    error::{ContextError, ErrMode},
    token::{any, one_of},
};

use crate::{
    error::ErrorCode,
    lexer::{Keyword, Operator, Token, TokenKind},
};

use super::{
    Input,
    helper::{token_boxed, token_or_insert, variable_token},
    record_element::RecordElementBase,
};

fn record_name<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Token<'a>> {
    alt((
        variable_token(false, false),
        one_of(|t: &Token<'_>| matches!(t.kind, TokenKind::Ordinal(_))).map(ToOwned::to_owned),
    ))
    .parse_next(i)
}

fn record_element<'t, 'a: 't, E: Clone + PartialEq + 'a>(
    named: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
    omit_named: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
    unnamed: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<Input<'t, 'a>, RecordElementBase<'a, E>, ErrMode<ContextError>> + Copy {
    let colon = |t: &Token<'a>| -> bool {
        *t == Operator::Colon || *t == Operator::QuestionColon || *t == Operator::ExclamationColon
    };
    move |i: &mut Input<'t, 'a>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseParen {
            return fail.parse_next(i);
        }
        let mut result = if *first == Operator::SpreadRange {
            (token_boxed(Operator::SpreadRange), spread)
                .map(|(s, e)| RecordElementBase::Spread(s, e.into(), None))
                .parse_next(i)?
        } else if colon(first) {
            (one_of(colon), omit_named)
                .map(|(c, o)| RecordElementBase::OmitNamed(c.to_owned().into(), o.into(), None))
                .parse_next(i)?
        } else {
            alt((
                (record_name, one_of(colon), named).map(|(r, c, n)| {
                    RecordElementBase::Named(Box::new(r), c.to_owned().into(), n.into(), None)
                }),
                unnamed.map(|u| RecordElementBase::Unnamed(u.into(), None)),
            ))
            .parse_next(i)?
        };
        let last = peek(any).parse_next(i)?;
        if *last == Operator::CloseParen
            || *last == Operator::CloseBrace
            || *last == Operator::CloseBracket
            || *last == Operator::Semicolon
            || *last == TokenKind::Eof
            || *last == Keyword::Return
            || *last == Keyword::Break
            || *last == Keyword::Continue
            || *last == Keyword::Case
            || *last == Keyword::Else
            || *last == Keyword::In
            || *last == Keyword::Let
        {
            return Ok(result);
        }
        let comma = token_or_insert(Operator::Comma, ErrorCode::MissingComma).parse_next(i)?;
        result.set_tail_comma(Box::new(comma));
        Ok(result)
    }
}

pub(super) fn record_base<'t, 'a: 't, E: Clone + PartialEq + 'a>(
    named: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
    omit_named: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
    unnamed: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 'a>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<
    Input<'t, 'a>,
    (
        Box<Token<'a>>,
        Vec<RecordElementBase<'a, E>>,
        Box<Token<'a>>,
    ),
    ErrMode<ContextError>,
> + Copy {
    move |i: &mut Input<'t, 'a>| {
        let open = token_boxed(Operator::OpenParen).parse_next(i)?;
        let parts: Vec<_> =
            repeat(0.., record_element(named, omit_named, unnamed, spread)).parse_next(i)?;
        let close = token_or_insert(Operator::CloseParen, ErrorCode::MissingCloseParen)
            .map(Box::new)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}

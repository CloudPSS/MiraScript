use winnow::{
    ModalResult, Parser,
    combinator::{fail, opt, peek, terminated},
    token::{any, literal, one_of},
};

use crate::lexer::{Operator, Token, TokenKind};

use super::{
    Input, RecordLikeElement, expression,
    helper::{literal_or_insert, spread_expression},
};

fn record_name<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Token<'a>> {
    one_of(|t: &Token<'a>| {
        matches!(
            t.kind,
            TokenKind::Identifier(_)
                | TokenKind::Ordinal(_)
                | TokenKind::Number(_)
                | TokenKind::String(_)
                | TokenKind::InterpolatedString(_, _)
        )
    })
    .map(ToOwned::to_owned)
    .parse_next(i)
}

pub(super) fn record_like_element<'a>(i: &mut Input<'_, 'a>) -> ModalResult<RecordLikeElement<'a>> {
    let first = peek(any).parse_next(i)?;
    if *first == Operator::CloseParen {
        return fail.parse_next(i);
    }
    let mut result = if *first == Operator::Spread {
        spread_expression
            .map(|e| RecordLikeElement::Spread(Box::new(e), None))
            .parse_next(i)?
    } else {
        (
            opt(terminated(record_name, literal(Operator::Colon))),
            expression,
        )
            .map(|(name, exp)| {
                if let Some(name) = name {
                    RecordLikeElement::Named(Box::new(name), Box::new(exp), None)
                } else {
                    RecordLikeElement::Unnamed(Box::new(exp), None)
                }
            })
            .parse_next(i)?
    };
    let last = peek(any).parse_next(i)?;
    if *last == Operator::CloseParen {
        return Ok(result);
    }
    let comma = literal_or_insert(Operator::Comma, "Missing comma").parse_next(i)?;
    *(match &mut result {
        RecordLikeElement::Spread(_, c)
        | RecordLikeElement::Named(_, _, c)
        | RecordLikeElement::Unnamed(_, c) => c,
    }) = Some(Box::new(comma));
    Ok(result)
}

use winnow::{
    ModalResult, Parser,
    combinator::{fail, opt, peek, preceded, terminated},
    token::{any, literal, one_of},
};

use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::{
    Input, RecordLikeElement, expression,
    helper::{spread_expression, token_or_insert, variable_token},
};

fn record_name<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Token<'a>> {
    one_of(|t: &Token<'a>| {
        matches!(
            t.kind,
            TokenKind::Identifier(_)
                | TokenKind::Ordinal(_)
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
    let mut result = if *first == Operator::SpreadRange {
        spread_expression
            .map(|(s, e)| RecordLikeElement::Spread(Box::new(s), Box::new(e), None))
            .parse_next(i)?
    } else if *first == Operator::Colon {
        preceded(literal(Operator::Colon), variable_token(false, false))
            .map(|t| RecordLikeElement::OmitNamed(Box::new(t), None))
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
    let comma = token_or_insert(Operator::Comma, "Missing comma").parse_next(i)?;
    *(match &mut result {
        RecordLikeElement::Spread(_, _, c)
        | RecordLikeElement::Named(_, _, c)
        | RecordLikeElement::OmitNamed(_, c)
        | RecordLikeElement::Unnamed(_, c) => c,
    }) = Some(Box::new(comma));
    Ok(result)
}

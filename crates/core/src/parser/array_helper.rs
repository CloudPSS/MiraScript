use winnow::{
    Parser,
    combinator::{alt, fail, peek, repeat},
    error::{ContextError, ErrMode},
    token::any,
};

use crate::{
    diagnostic::DiagnosticCode,
    lexer::{Keyword, Operator, Token, TokenKind},
};

use super::{
    Input, Range,
    array_element::ArrayElementBase,
    helper::{token_boxed, token_or_insert},
};

fn array_element<'t, 's: 't, E: Clone + PartialEq + 's>(
    element: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    range: impl Parser<Input<'t, 's>, Range<'s>, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<Input<'t, 's>, ArrayElementBase<'s, E>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'t, 's>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseBracket {
            return fail.parse_next(i);
        }
        let mut result = if *first == Operator::SpreadRange {
            (token_boxed(Operator::SpreadRange), spread)
                .map(|(s, e)| ArrayElementBase::Spread(s, e.into(), None))
                .parse_next(i)?
        } else {
            alt((
                range.map(|r| ArrayElementBase::Range(Box::new(r), None)),
                element.map(|e| ArrayElementBase::Element(Box::new(e), None)),
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
        let comma = token_or_insert(Operator::Comma, DiagnosticCode::MissingComma).parse_next(i)?;
        result.set_tail_comma(Box::new(comma));
        Ok(result)
    }
}

pub(super) fn array_base<'t, 's: 't, E: Clone + PartialEq + 's>(
    element: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    range: impl Parser<Input<'t, 's>, Range<'s>, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<
    Input<'t, 's>,
    (Box<Token<'s>>, Vec<ArrayElementBase<'s, E>>, Box<Token<'s>>),
    ErrMode<ContextError>,
> + Copy {
    move |i: &mut Input<'t, 's>| {
        let open = token_boxed(Operator::OpenBracket).parse_next(i)?;
        let parts: Vec<_> = repeat(0.., array_element(element, range, spread)).parse_next(i)?;
        let close = token_or_insert(Operator::CloseBracket, DiagnosticCode::MissingCloseBracket)
            .map(Box::new)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}

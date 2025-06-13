use winnow::{
    Parser,
    combinator::{alt, fail, peek, repeat},
    error::{ContextError, ErrMode},
    token::any,
};

use crate::{
    diagnostic::DiagnosticCode,
    lexer::{Keyword, Operator, Token, TokenKind},
    parser::list_item::ListItem,
};

use super::{
    Input, Range,
    array_element::ArrayElementBase,
    helper::{token_boxed, token_or_insert},
};

type _ArrayElement<'s, E> = ListItem<'s, ArrayElementBase<'s, E>>;
fn array_element<'t, 's: 't, E: Clone + PartialEq + 's>(
    element: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    range: impl Parser<Input<'t, 's>, Range<'s>, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<Input<'t, 's>, _ArrayElement<'s, E>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'t, 's>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseBracket
            || *first == Operator::CloseBrace
            || *first == Operator::CloseParen
        {
            return fail.parse_next(i);
        }
        let result = if *first == Operator::SpreadRange {
            (token_boxed(Operator::SpreadRange), spread)
                .map(|(s, e)| ArrayElementBase::Spread(s, e.into()))
                .parse_next(i)?
        } else {
            alt((
                range.map(|r| ArrayElementBase::Range(Box::new(r))),
                element.map(|e| ArrayElementBase::Element(Box::new(e))),
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
            return Ok(ListItem::new(result));
        }
        let comma = token_or_insert(Operator::Comma, DiagnosticCode::MissingComma).parse_next(i)?;
        Ok(ListItem::new_with_comma(result, comma))
    }
}

type _ArrayLike<'s, E> = (Box<Token<'s>>, Vec<_ArrayElement<'s, E>>, Box<Token<'s>>);
pub(super) fn array_base<'t, 's: 't, E: Clone + PartialEq + 's>(
    brace: [Operator; 2],
    element: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    range: impl Parser<Input<'t, 's>, Range<'s>, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<Input<'t, 's>, _ArrayLike<'s, E>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'t, 's>| {
        let open = token_boxed(brace[0]).parse_next(i)?;
        let parts: Vec<_> = repeat(0.., array_element(element, range, spread)).parse_next(i)?;
        let close = token_or_insert(brace[1], DiagnosticCode::MissingCloseBracket)
            .map(Box::new)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}

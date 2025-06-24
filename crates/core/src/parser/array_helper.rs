use winnow::{
    combinator::{alt, fail, peek, repeat},
    token::any,
};

use super::{
    array_element::ArrayElementBase,
    helper::{token, token_or_insert},
    list_item::ListItem,
    prelude::*,
};

type _ArrayElement<'s, E> = ListItem<'s, ArrayElementBase<'s, E>>;
fn array_element<'s, E: Clone + PartialEq + 's>(
    element: impl Parser<'s, E>,
    range: impl Parser<'s, Range<'s>>,
    spread: impl Parser<'s, E>,
) -> impl Parser<'s, _ArrayElement<'s, E>> {
    move |i: &mut Input<'s>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseBracket
            || *first == Operator::CloseBrace
            || *first == Operator::CloseParen
        {
            return fail.parse_next(i);
        }
        let result = if *first == Operator::SpreadRange {
            (token(Operator::SpreadRange), spread)
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

type _ArrayLike<'s, E> = (TokenRef<'s>, Vec<_ArrayElement<'s, E>>, TokenRef<'s>);
pub(super) fn array_base<'t, 's: 't, E: Clone + PartialEq + 's>(
    mut open: impl Parser<'s, TokenRef<'s>>,
    mut close: impl Parser<'s, TokenRef<'s>>,
    element: impl Parser<'s, E>,
    range: impl Parser<'s, Range<'s>>,
    spread: impl Parser<'s, E>,
) -> impl Parser<'s, _ArrayLike<'s, E>> {
    move |i: &mut Input<'s>| {
        let open = open.parse_next(i)?;
        let parts: Vec<_> = repeat(0.., array_element(element, range, spread)).parse_next(i)?;
        let close = close.parse_next(i)?;
        Ok((open, parts, close))
    }
}

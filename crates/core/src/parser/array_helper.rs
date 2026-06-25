use winnow::{
    combinator::{fail, peek, repeat},
    token::any,
};

use super::{
    helper::{token, token_or_insert},
    prelude::*,
};

type _ArrayElement<'s, E, S> = ListItem<'s, ArrayElementBase<'s, E, S>>;
fn array_element<'s, E: Clone + PartialEq + 's, S: Clone + PartialEq + 's>(
    element: impl Parser<'s, E>,
    spread: impl Parser<'s, S>,
    mut missing: impl FnMut(usize) -> E + Copy,
) -> impl Parser<'s, _ArrayElement<'s, E, S>> {
    move |i: &mut Input<'s>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::Comma {
            let comma = token(Operator::Comma).parse_next(i)?;
            let pos = comma.range.start;
            let missing = missing(pos);
            return Ok(ListItem::new_with_comma(
                ArrayElementBase::Element(Box::new(missing)),
                comma,
            ));
        }
        if *first == Operator::CloseBracket
            || *first == Operator::CloseBrace
            || *first == Operator::CloseParen
            || *first == TokenKind::Eof
        {
            return fail.parse_next(i);
        }
        let result = if *first == Operator::SpreadRange {
            (token(Operator::SpreadRange), spread)
                .map(|(s, e)| ArrayElementBase::Spread(s, e.into()))
                .parse_next(i)?
        } else {
            element
                .map(|e| ArrayElementBase::Element(Box::new(e)))
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
            || *last == Keyword::NotIn
            || *last == Keyword::Let
            || *last == Keyword::Const
        {
            return Ok(ListItem::new(result));
        }
        let comma = token_or_insert(Operator::Comma, DiagnosticCode::MissingComma).parse_next(i)?;
        Ok(ListItem::new_with_comma(result, comma))
    }
}

type _ArrayLike<'s, E, S> = (TokenRef<'s>, Vec<_ArrayElement<'s, E, S>>, TokenRef<'s>);
pub(super) fn array_base<'t, 's: 't, E: Clone + PartialEq + 's, S: Clone + PartialEq + 's>(
    mut open: impl Parser<'s, TokenRef<'s>>,
    mut close: impl Parser<'s, TokenRef<'s>>,
    element: impl Parser<'s, E>,
    spread: impl Parser<'s, S>,
    missing: impl FnMut(usize) -> E + Copy,
) -> impl Parser<'s, _ArrayLike<'s, E, S>> {
    move |i: &mut Input<'s>| {
        let open = open.parse_next(i)?;
        let parts: Vec<_> = repeat(0.., array_element(element, spread, missing)).parse_next(i)?;
        let close = close.parse_next(i)?;
        Ok((open, parts, close))
    }
}

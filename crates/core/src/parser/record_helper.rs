use winnow::{
    combinator::{alt, fail, peek, repeat},
    token::{any, one_of},
};

use super::{
    helper::{token, token_or_insert, variable_token},
    prelude::*,
};

fn record_name<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
    alt((
        // (x: ..)
        variable_token(true, true),
        one_of(|t: &Token<'_>| {
            // (1: ..)     || (`x`: ..)
            t.is_ordinal() || t.is_string()
        })
        .map(TokenRef::borrow),
    ))
    .parse_next(i)
}

fn record_element<'t, 's: 't, E: Clone + PartialEq + 's, I: Clone + PartialEq + 's>(
    named: impl Parser<'s, E>,
    mut interpolate_name: impl FnMut(&'s Token<'s>) -> I + Copy,
    omit_named: impl Parser<'s, E>,
    unnamed: impl Parser<'s, E>,
    spread: impl Parser<'s, E>,
    mut missing: impl FnMut(usize) -> E + Copy,
) -> impl Parser<'s, ListItem<'s, RecordElementBase<'s, E, I>>> {
    let colon = |t: &Token<'s>| -> bool { *t == Operator::Colon || *t == Operator::QuestionColon };
    move |i: &mut Input<'s>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::Comma {
            return Ok(ListItem::new_with_comma(
                RecordElementBase::Unnamed(missing(first.range.start).into()),
                token(Operator::Comma).parse_next(i)?,
            ));
        }
        if *first == Operator::CloseBracket
            || *first == Operator::CloseBrace
            || *first == Operator::CloseParen
            || *first == TokenKind::Eof
        {
            return fail.parse_next(i);
        }
        let result = alt((
            (token(Operator::SpreadRange), spread)
                .map(|(s, e)| RecordElementBase::Spread(s, e.into())),
            (one_of(colon), omit_named)
                .map(|(c, o)| RecordElementBase::OmitNamed(c.into(), o.into())),
            (
                one_of(|t: &Token<'s>| t.is_interpolated_string()),
                one_of(colon),
                named,
            )
                .map(|(r, c, n)| {
                    RecordElementBase::InterpolateNamed(
                        Box::new(interpolate_name(r)),
                        c.into(),
                        n.into(),
                    )
                }),
            (record_name, one_of(colon), named)
                .map(|(r, c, n)| RecordElementBase::Named(r, c.into(), n.into())),
            unnamed.map(|u| RecordElementBase::Unnamed(u.into())),
        ))
        .parse_next(i)?;
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
            || *last == Keyword::Const
        {
            return Ok(ListItem::new(result));
        }
        let comma = token_or_insert(Operator::Comma, DiagnosticCode::MissingComma).parse_next(i)?;
        Ok(ListItem::new_with_comma(result, comma))
    }
}

pub(super) fn record_base<'t, 's: 't, E: Clone + PartialEq + 's, I: Clone + PartialEq + 's>(
    named: impl Parser<'s, E>,
    interpolate_name: impl FnMut(&'s Token<'s>) -> I + Copy,
    omit_named: impl Parser<'s, E>,
    unnamed: impl Parser<'s, E>,
    spread: impl Parser<'s, E>,
    missing: impl FnMut(usize) -> E + Copy,
) -> impl Parser<
    's,
    (
        TokenRef<'s>,
        Vec<ListItem<'s, RecordElementBase<'s, E, I>>>,
        TokenRef<'s>,
    ),
> {
    move |i: &mut Input<'s>| {
        let open = token(Operator::OpenParen).parse_next(i)?;
        let parts: Vec<_> = repeat(
            0..,
            record_element(
                named,
                interpolate_name,
                omit_named,
                unnamed,
                spread,
                missing,
            ),
        )
        .parse_next(i)?;
        let close = token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}

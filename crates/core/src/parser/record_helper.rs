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

fn record_element<'t, 's: 't + 'a, 'a, E: PartialEq + 'a, I: PartialEq + 'a>(
    arena: &'a AstArena,
    named: impl Parser<'s, E>,
    mut interpolate_name: impl FnMut(&'s Token<'s>) -> I + Copy,
    omit_named: impl Parser<'s, E>,
    unnamed: impl Parser<'s, E>,
    spread: impl Parser<'s, E>,
    mut missing: impl FnMut(usize) -> E + Copy,
) -> impl Parser<'s, ListItem<'s, 'a, RecordElementBase<'s, 'a, E, I>>> {
    let colon = |t: &Token<'s>| -> bool { *t == Operator::Colon || *t == Operator::QuestionColon };
    move |i: &mut Input<'s>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::Comma {
            return Ok(ListItem::new_with_comma(
                arena,
                RecordElementBase::Unnamed(arena.alloc(missing(first.range.start))),
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
                .map(|(s, e)| RecordElementBase::Spread(s, arena.alloc(e))),
            (one_of(colon), omit_named)
                .map(|(c, o)| RecordElementBase::OmitNamed(c.into(), arena.alloc(o))),
            (
                one_of(|t: &Token<'s>| t.is_interpolated_string()),
                one_of(colon),
                named,
            )
                .map(|(r, c, n)| {
                    RecordElementBase::InterpolateNamed(
                        arena.alloc(interpolate_name(r)),
                        c.into(),
                        arena.alloc(n),
                    )
                }),
            (record_name, one_of(colon), named)
                .map(|(r, c, n)| RecordElementBase::Named(r, c.into(), arena.alloc(n))),
            unnamed.map(|u| RecordElementBase::Unnamed(arena.alloc(u))),
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
            || *last == Keyword::NotIn
            || *last == Keyword::Let
            || *last == Keyword::Const
        {
            return Ok(ListItem::new(arena, result));
        }
        let comma = token_or_insert(Operator::Comma, DiagnosticCode::MissingComma).parse_next(i)?;
        Ok(ListItem::new_with_comma(arena, result, comma))
    }
}

pub(super) fn record_base<'t, 's: 't + 'a, 'a, E: PartialEq + 'a, I: PartialEq + 'a>(
    arena: &'a AstArena,
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
        Vec<ListItem<'s, 'a, RecordElementBase<'s, 'a, E, I>>>,
        TokenRef<'s>,
    ),
> {
    move |i: &mut Input<'s>| {
        let open = token(Operator::OpenParen).parse_next(i)?;
        let parts: Vec<_> = repeat(
            0..,
            record_element(
                arena,
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

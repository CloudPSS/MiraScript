use winnow::{
    ModalResult, Parser,
    combinator::{alt, fail, peek, repeat},
    error::{ContextError, ErrMode},
    token::{any, one_of},
};

use crate::{
    diagnostic::DiagnosticCode,
    lexer::{Keyword, Operator, Token, TokenKind},
};

use super::{
    Input,
    helper::{token_boxed, token_or_insert, variable_token},
    list_item::ListItem,
    record_element::RecordElementBase,
};

fn record_name<'s>(i: &mut Input<'_, 's>) -> ModalResult<Token<'s>> {
    alt((
        // (x: ..)
        variable_token(true, true),
        one_of(|t: &Token<'_>| {
            // (1: ..)     || (`x`: ..)
            t.is_ordinal() || t.is_string()
        })
        .map(ToOwned::to_owned),
    ))
    .parse_next(i)
}

fn record_element<'t, 's: 't, E: Clone + PartialEq + 's, I: Clone + PartialEq + 's>(
    named: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    mut interpolate_name: impl FnMut(&Token<'s>) -> I + Copy,
    omit_named: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    unnamed: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<Input<'t, 's>, ListItem<'s, RecordElementBase<'s, E, I>>, ErrMode<ContextError>> + Copy
{
    let colon = |t: &Token<'s>| -> bool { *t == Operator::Colon || *t == Operator::QuestionColon };
    move |i: &mut Input<'t, 's>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseBracket
            || *first == Operator::CloseBrace
            || *first == Operator::CloseParen
        {
            return fail.parse_next(i);
        }
        let result = alt((
            (token_boxed(Operator::SpreadRange), spread)
                .map(|(s, e)| RecordElementBase::Spread(s, e.into())),
            (one_of(colon), omit_named)
                .map(|(c, o)| RecordElementBase::OmitNamed(c.to_owned().into(), o.into())),
            (
                one_of(|t: &Token<'s>| t.is_interpolated_string()),
                one_of(colon),
                named,
            )
                .map(|(r, c, n)| {
                    RecordElementBase::InterpolateNamed(
                        Box::new(interpolate_name(r)),
                        c.to_owned().into(),
                        n.into(),
                    )
                }),
            (record_name, one_of(colon), named).map(|(r, c, n)| {
                RecordElementBase::Named(Box::new(r), c.to_owned().into(), n.into())
            }),
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
        {
            return Ok(ListItem::new(result));
        }
        let comma = token_or_insert(Operator::Comma, DiagnosticCode::MissingComma).parse_next(i)?;
        Ok(ListItem::new_with_comma(result, comma))
    }
}

pub(super) fn record_base<'t, 's: 't, E: Clone + PartialEq + 's, I: Clone + PartialEq + 's>(
    named: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    interpolate_name: impl FnMut(&Token<'s>) -> I + Copy,
    omit_named: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    unnamed: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 's>, E, ErrMode<ContextError>> + Copy,
) -> impl Parser<
    Input<'t, 's>,
    (
        Box<Token<'s>>,
        Vec<ListItem<'s, RecordElementBase<'s, E, I>>>,
        Box<Token<'s>>,
    ),
    ErrMode<ContextError>,
> + Copy {
    move |i: &mut Input<'t, 's>| {
        let open = token_boxed(Operator::OpenParen).parse_next(i)?;
        let parts: Vec<_> = repeat(
            0..,
            record_element(named, interpolate_name, omit_named, unnamed, spread),
        )
        .parse_next(i)?;
        let close = token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen)
            .map(Box::new)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}

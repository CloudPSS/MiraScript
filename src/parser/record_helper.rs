use winnow::{
    ModalResult, Parser,
    combinator::{alt, fail, peek, repeat},
    error::{ContextError, ErrMode},
    token::{any, one_of},
};

use crate::{
    ansi::DisplayIdent,
    lexer::{Keyword, Operator, Token, TokenKind},
};

use super::{
    Input,
    helper::{token_boxed, token_or_insert},
    record_elements::RecordElementBase,
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

fn record_element_base<
    't,
    'a: 't,
    Named: DisplayIdent + Clone + PartialEq,
    OmitNamed: DisplayIdent + Clone + PartialEq,
    Unnamed: DisplayIdent + Clone + PartialEq,
    Spread: DisplayIdent + Clone + PartialEq,
>(
    named: impl Parser<Input<'t, 'a>, Named, ErrMode<ContextError>> + Copy,
    omit_named: impl Parser<Input<'t, 'a>, OmitNamed, ErrMode<ContextError>> + Copy,
    unnamed: impl Parser<Input<'t, 'a>, Unnamed, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 'a>, Spread, ErrMode<ContextError>> + Copy,
) -> impl Parser<
    Input<'t, 'a>,
    RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>,
    ErrMode<ContextError>,
> + Copy {
    move |i: &mut Input<'t, 'a>| {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseParen {
            return fail.parse_next(i);
        }
        let mut result = if *first == Operator::SpreadRange {
            (token_boxed(Operator::SpreadRange), spread)
                .map(|(s, e)| RecordElementBase::Spread(s, Box::new(e), None))
                .parse_next(i)?
        } else if *first == Operator::Colon {
            (token_boxed(Operator::Colon), omit_named)
                .map(|(c, o)| RecordElementBase::OmitNamed(c, Box::new(o), None))
                .parse_next(i)?
        } else {
            alt((
                (record_name, token_boxed(Operator::Colon), named).map(|(name, colon, exp)| {
                    RecordElementBase::Named(Box::new(name), colon, Box::new(exp), None)
                }),
                unnamed.map(|o| RecordElementBase::Unnamed(Box::new(o), None)),
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
        let comma = token_or_insert(Operator::Comma, "Missing comma").parse_next(i)?;
        result.set_tail_comma(Box::new(comma));
        Ok(result)
    }
}

type RecordBase<'a, Named, OmitNamed, Unnamed, Spread> = (
    Box<Token<'a>>,
    Vec<RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>>,
    Box<Token<'a>>,
);

pub(super) fn record_base<
    't,
    'a: 't,
    Named: DisplayIdent + Clone + PartialEq,
    OmitNamed: DisplayIdent + Clone + PartialEq,
    Unnamed: DisplayIdent + Clone + PartialEq,
    Spread: DisplayIdent + Clone + PartialEq,
>(
    named: impl Parser<Input<'t, 'a>, Named, ErrMode<ContextError>> + Copy,
    omit_named: impl Parser<Input<'t, 'a>, OmitNamed, ErrMode<ContextError>> + Copy,
    unnamed: impl Parser<Input<'t, 'a>, Unnamed, ErrMode<ContextError>> + Copy,
    spread: impl Parser<Input<'t, 'a>, Spread, ErrMode<ContextError>> + Copy,
) -> impl Parser<Input<'t, 'a>, RecordBase<'a, Named, OmitNamed, Unnamed, Spread>, ErrMode<ContextError>>
+ Copy {
    move |i: &mut Input<'t, 'a>| {
        let open = token_boxed(Operator::OpenParen).parse_next(i)?;
        let parts: Vec<_> =
            repeat(0.., record_element_base(named, omit_named, unnamed, spread)).parse_next(i)?;
        let close = token_or_insert(Operator::CloseParen, "Missing ')'")
            .map(Box::new)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}

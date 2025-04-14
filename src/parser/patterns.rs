use winnow::{
    ModalResult, Parser,
    combinator::{alt, empty, opt},
    error::{ContextError, ErrMode},
    stream::Location,
    token::take_till,
};

use crate::{
    lexer::{Keyword, Operator, Token, TokenKind},
    utils::SourceRange,
};

use super::{
    Input, Pattern,
    helper::{literal_token, token_boxed, variable_token},
    record_helper::record_base,
};

fn unknown_pattern<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Pattern<'a>> {
    take_till(1.., |t: &Token<'a>| {
        *t == TokenKind::Eof
            || *t == Keyword::If
            || *t == Keyword::Fn
            || *t == Keyword::Loop
            || *t == Keyword::While
            || *t == Keyword::Match
            || *t == Keyword::For
            || *t == Operator::Comma
            || *t == Operator::Equal
            || *t == Operator::Semicolon
            || *t == Operator::OpenBrace
            || *t == Operator::CloseBrace
            || *t == Operator::OpenBracket
            || *t == Operator::CloseBracket
    })
    .map(|t: &[Token<'a>]| Pattern::unknown(t, "Unknown pattern"))
    .parse_next(i)
}

pub(super) fn pattern_or_insert<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        let start = i.previous_token_end();
        alt((
            pattern(rebind),
            empty.map(|_| {
                Pattern::unknown_range(
                    vec![],
                    SourceRange { start, end: start },
                    "Pattern expected",
                )
            }),
        ))
        .parse_next(i)
    }
}

pub(super) fn pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        alt((
            literal_pattern,
            record_pattern(rebind),
            discard_bind_pattern(rebind),
            unknown_pattern,
        ))
        .parse_next(i)
    }
}

fn literal_pattern<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Pattern<'a>> {
    literal_token
        .map(|t| Pattern::Literal(Box::new(t)))
        .parse_next(i)
}

fn discard_bind_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        (opt(token_boxed(Keyword::Mut)), variable_token(true, false))
            .map(|(kw_mut, id)| {
                if rebind && kw_mut.is_some() {
                    let kw_mut = kw_mut
                        .unwrap()
                        .wrap_as_unknown("'mut' is not allowed while rebinding");
                    return Pattern::Bind(Some(Box::new(kw_mut)), Box::new(id));
                }
                if id.kind == Keyword::Underscore {
                    if kw_mut.is_some() {
                        return Pattern::unknown(
                            vec![*kw_mut.unwrap(), id],
                            "Can not use 'mut' in discard pattern",
                        );
                    }
                    Pattern::Discard(Box::new(id))
                } else {
                    Pattern::Bind(kw_mut, Box::new(id))
                }
            })
            .parse_next(i)
    }
}

fn record_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    let omit_named = move |i: &mut Input<'_, 'a>| -> ModalResult<Pattern<'a>> {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if matches!(p, Pattern::Bind(..)) || matches!(p, Pattern::Unknown { .. }) {
                    p
                } else {
                    p.wrap_as_unknown(t, "Must be bind pattern while record field name omitted")
                }
            })
            .parse_next(i)
    };
    let spread = move |i: &mut Input<'_, 'a>| -> ModalResult<Pattern<'a>> {
        pattern(rebind)
            .with_taken()
            .map(|(p, t)| {
                if matches!(p, Pattern::Bind(..))
                    || matches!(p, Pattern::Record(..))
                    || matches!(p, Pattern::Unknown { .. })
                {
                    p
                } else if matches!(p, Pattern::Discard(..)) {
                    p.wrap_as_unknown(t, "Discard pattern should be omitted in spread pattern")
                } else {
                    p.wrap_as_unknown(t, "Spread pattern only matches record")
                }
            })
            .parse_next(i)
    };

    move |i: &mut Input<'_, 'a>| {
        let (open, parts, close) = record_base(
            pattern_or_insert(rebind).map(Box::new),
            omit_named.map(Box::new),
            pattern_or_insert(rebind).map(Box::new),
            opt(spread.map(Box::new)),
        )
        .parse_next(i)?;
        Ok(Pattern::Record(open, parts, close))
    }
}

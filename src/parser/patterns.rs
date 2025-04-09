use winnow::{
    ModalResult, Parser,
    combinator::{alt, empty, opt},
    error::{ContextError, ErrMode},
    stream::Location,
};

use crate::{lexer::Keyword, utils::SourceRange};

use super::{
    Input, Pattern,
    helper::{literal_token, token_boxed, variable_token},
    record_helper::record_base,
};

pub(super) fn pattern_or_insert<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
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
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'_, 'a>| {
        alt((
            literal_pattern,
            record_pattern(rebind),
            discard_bind_pattern(rebind),
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
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
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
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
    let omit_named = move |i: &mut Input<'_, 'a>| -> ModalResult<Pattern<'a>> {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if matches!(p, Pattern::Bind(..)) || matches!(p, Pattern::Unknown { .. }) {
                    p
                } else {
                    Pattern::unknown(t, "Must be bind pattern while record field name omitted")
                }
            })
            .parse_next(i)
    };

    move |i: &mut Input<'_, 'a>| {
        let (open, parts, close) = record_base(
            pattern_or_insert(rebind),
            omit_named,
            pattern_or_insert(rebind),
            pattern_or_insert(rebind),
        )
        .parse_next(i)?;
        Ok(Pattern::Record(open, parts, close))
    }
}

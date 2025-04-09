use winnow::{
    Parser,
    combinator::{alt, empty, opt},
    error::{ContextError, ErrMode},
    stream::Location,
};

use crate::{lexer::Keyword, utils::SourceRange};

use super::{
    Input, Pattern,
    helper::{literal_token, token_boxed, variable_token},
};

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
            literal_token.map(|t| Pattern::Literal(Box::new(t))),
            (opt(token_boxed(Keyword::Mut)), variable_token(true, false)).map(|(kw_mut, id)| {
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
            }),
        ))
        .parse_next(i)
    }
}

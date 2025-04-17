use winnow::{
    ModalResult, Parser,
    combinator::{alt, empty, fail, opt, peek, preceded, separated_foldl1, seq},
    error::{ContextError, ErrMode},
    stream::Location,
    token::{one_of, take_till},
};

use crate::{
    lexer::{Keyword, Operator, Token, TokenKind},
    utils::SourceRange,
};

use super::{
    Input, Pattern, RecordPattern,
    array_helper::array_base,
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
            || *t == Keyword::Let
            || *t == Operator::Comma
            || *t == Operator::Equal
            || *t == Operator::Semicolon
            || *t == Operator::OpenBrace
            || *t == Operator::CloseBrace
            || *t == Operator::OpenBracket
            || *t == Operator::CloseBracket
            || *t == Operator::OpenParen
            || *t == Operator::CloseParen
    })
    .map(|t: &[Token<'a>]| Pattern::unknown(t, "Unknown pattern"))
    .parse_next(i)
}

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
    move |i: &mut Input<'_, 'a>| or_pattern(rebind).parse_next(i)
}

fn primary_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        alt((
            relation_pattern,
            record_like_pattern(rebind),
            array_pattern(rebind),
            range_pattern,
            constants_pattern,
            discard_bind_pattern(rebind),
            not_pattern(rebind),
            unknown_pattern,
        ))
        .parse_next(i)
    }
}

fn not_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        (token_boxed(Keyword::Not), primary_pattern(rebind))
            .map(|(kw_not, p)| Pattern::Not(kw_not, Box::new(p)))
            .parse_next(i)
    }
}

fn and_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        separated_foldl1(
            primary_pattern(rebind),
            token_boxed(Keyword::And),
            |left, op, right| Pattern::And(Box::new(left), op, Box::new(right)),
        )
        .parse_next(i)
    }
}

fn or_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 'a>| {
        separated_foldl1(
            and_pattern(rebind),
            token_boxed(Keyword::Or),
            |left, op, right| Pattern::Or(Box::new(left), op, Box::new(right)),
        )
        .parse_next(i)
    }
}

fn constants_pattern<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Pattern<'a>> {
    (
        opt(one_of(|t: &Token<'a>| {
            *t == Operator::Plus || *t == Operator::Minus || *t == Operator::Exclamation
        })),
        literal_token,
    )
        .map(|(t, l)| {
            let Some(op) = t else {
                return Pattern::Constant(None, Box::new(l));
            };
            let result = Pattern::Constant(Some(op.to_owned().into()), Box::new(l.clone()));
            if *op == Operator::Exclamation {
                return result
                    .wrap_as_unknown([op.to_owned(), l], "Not operator is not allowed in pattern");
            }
            if !(matches!(l.kind, TokenKind::Number(..))
                || matches!(l.kind, TokenKind::Ordinal(..))
                || l == Keyword::Inf)
            {
                return result.wrap_as_unknown(
                    [op.to_owned(), l],
                    "Unexpected operator in pattern, only number is allowed",
                );
            }
            result
        })
        .parse_next(i)
}

fn relation_pattern<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Pattern<'a>> {
    seq!(Pattern::Relation(
        one_of(|t: &Token<'a>| *t == Operator::Less
            || *t == Operator::LessEqual
            || *t == Operator::Greater
            || *t == Operator::GreaterEqual
            || *t == Operator::EqualEqual
            || *t == Operator::NotEqual
            || *t == Operator::TildeEqual
            || *t == Operator::NotTildeEqual)
        .map(|t: &Token<'a>| Box::new(t.to_owned())),
        constants_pattern.map(Box::new),
    ))
    .parse_next(i)
}

fn range_pattern<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Pattern<'a>> {
    seq!(Pattern::Range(
        constants_pattern.map(Box::new),
        one_of(|t: &Token<'a>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(|t: &Token<'a>| Box::new(t.to_owned())),
        constants_pattern.map(Box::new),
    ))
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

fn pattern_spread<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'_, 'a>| {
        opt(pattern(rebind))
            .with_taken()
            .map(|(p, t)| {
                if p.is_none() {
                    return Pattern::SpreadDiscard;
                }
                let p = p.unwrap();
                if matches!(p, Pattern::Discard(..)) {
                    p.wrap_as_unknown(t, "Discard pattern should be omitted in spread pattern")
                } else {
                    p
                }
            })
            .parse_next(i)
    }
}

fn record_like_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
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

    let unnamed = move |i: &mut Input<'_, 'a>| {
        alt((
            preceded(
                peek(one_of(|t: &Token<'a>| {
                    *t == Operator::Comma || *t == Operator::CloseParen
                })),
                pattern_or_insert(rebind),
            ),
            pattern(rebind),
        ))
        .parse_next(i)
    };

    move |i: &mut Input<'_, 'a>| {
        let (open, parts, close) = record_base(
            pattern_or_insert(rebind),
            omit_named,
            unnamed,
            pattern_spread(rebind),
        )
        .parse_next(i)?;
        let result = if parts.len() == 1 {
            let part = parts.into_iter().next().unwrap();
            if let RecordPattern::Unnamed(exp, None) = part {
                Pattern::Grouping(open, exp, close)
            } else {
                Pattern::Record(open, vec![part], close)
            }
        } else {
            Pattern::Record(open, parts, close)
        };
        Ok(result)
    }
}

fn array_pattern<'t, 'a: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 'a>, Pattern<'a>, ErrMode<ContextError>> + Copy {
    let element_pattern = move |i: &mut Input<'_, 'a>| {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if matches!(p, Pattern::Range(..)) {
                    p.wrap_as_unknown(t, "Range pattern in array pattern should be parenthesised ")
                } else {
                    p
                }
            })
            .parse_next(i)
    };
    move |i: &mut Input<'_, 'a>| {
        let (open, parts, close) =
            array_base(element_pattern, fail, pattern_spread(rebind)).parse_next(i)?;
        Ok(Pattern::Array(open, parts, close))
    }
}

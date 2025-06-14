use std::ops::DerefMut;

use winnow::{
    combinator::{alt, empty, fail, opt, peek, preceded, separated_foldl1, seq},
    token::{one_of, take_till},
};

use super::{
    ArrayElementBase, AstWalker,
    array_helper::array_base,
    helper::{literal_token, token, variable_token},
    list_item::ListItem,
    prelude::*,
    record_element::RecordElementBase,
    record_helper::record_base,
};

fn unknown_pattern<'s>(i: &mut Input<'s>) -> Result<Pattern<'s>> {
    take_till(1.., |t: &Token<'s>| {
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
    .map(|t: &[Token<'s>]| {
        Pattern::unknown(
            t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
            DiagnosticCode::UnknownPattern,
        )
    })
    .parse_next(i)
}

pub(super) fn pattern_or_insert<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
        let start = i.previous_token_end();
        alt((
            pattern(rebind),
            empty.map(|_| {
                Pattern::unknown_range(
                    vec![Token::empty(start).into()],
                    start..start,
                    DiagnosticCode::PatternExpected,
                )
            }),
        ))
        .parse_next(i)
    }
}

pub(super) fn pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| or_pattern(rebind).parse_next(i)
}

fn primary_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
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

fn not_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
        (token(Keyword::Not), primary_pattern(rebind))
            .map(|(kw_not, p)| Pattern::Not(kw_not, Box::new(p)))
            .parse_next(i)
    }
}

fn and_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
        separated_foldl1(
            primary_pattern(rebind),
            token(Keyword::And),
            |left, op, right| Pattern::And(Box::new(left), op, Box::new(right)),
        )
        .parse_next(i)
    }
}

fn or_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
        separated_foldl1(
            and_pattern(rebind),
            token(Keyword::Or),
            |left, op, right| Pattern::Or(Box::new(left), op, Box::new(right)),
        )
        .parse_next(i)
    }
}

fn constants_pattern<'s>(i: &mut Input<'s>) -> Result<Pattern<'s>> {
    (
        opt(one_of(|t: &Token<'s>| {
            *t == Operator::Plus || *t == Operator::Minus || *t == Operator::Exclamation
        })),
        literal_token,
    )
        .map(|(t, l)| {
            let Some(op) = t else {
                return Pattern::Constant(None, l);
            };
            if *op == Operator::Exclamation {
                return Pattern::Constant(None, l.clone()).wrap_as_unknown(
                    [op.into(), l],
                    DiagnosticCode::ExclamationInConstantsPattern,
                );
            }
            if !(l.is_number() || l.is_ordinal() || *l == Keyword::Inf) {
                return Pattern::Constant(None, l.clone()).wrap_as_unknown(
                    [op.into(), l],
                    DiagnosticCode::UnexpectedOperatorInConstantsPattern,
                );
            }
            Pattern::Constant(Some(op.into()), l)
        })
        .parse_next(i)
}

fn relation_pattern<'s>(i: &mut Input<'s>) -> Result<Pattern<'s>> {
    seq!(Pattern::Relation(
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Operator(op) if op.is_relation()))
            .map(TokenRef::borrow),
        constants_pattern.map(Box::new),
    ))
    .parse_next(i)
}

fn range_pattern<'s>(i: &mut Input<'s>) -> Result<Pattern<'s>> {
    seq!(Pattern::Range(
        constants_pattern.map(Box::new),
        one_of(|t: &Token<'s>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(TokenRef::borrow),
        constants_pattern.map(Box::new),
    ))
    .parse_next(i)
}

fn discard_bind_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
        (opt(token(Keyword::Mut)), variable_token(true, false))
            .map(|(kw_mut, id)| {
                if id.is_unknown() {
                    let tokens = if let Some(kw_mut) = kw_mut {
                        vec![kw_mut, id]
                    } else {
                        vec![id]
                    };
                    return Pattern::unknown_errors(tokens, vec![]);
                }
                if id.kind == Keyword::Underscore {
                    if let Some(kw_mut) = kw_mut {
                        return Pattern::unknown(
                            vec![kw_mut, id],
                            DiagnosticCode::MutInDiscardPattern,
                        );
                    }
                    return Pattern::Discard(id);
                }
                if rebind && kw_mut.is_some() {
                    let kw_mut = kw_mut
                        .unwrap()
                        .wrap_as_unknown(DiagnosticCode::MutInRebindPattern);
                    return Pattern::Bind(Some(kw_mut), id);
                }
                Pattern::Bind(kw_mut, id)
            })
            .parse_next(i)
    }
}

fn pattern_spread<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| {
        let pos = i.previous_token_end();
        opt(pattern(rebind))
            .map(|p| {
                let Some(p) = p else {
                    return Pattern::SpreadDiscard(pos);
                };
                if let Pattern::Discard(t) = &p {
                    let tokens = [t.clone()];
                    p.wrap_as_unknown(tokens, DiagnosticCode::DiscardInSpreadPattern)
                } else {
                    p
                }
            })
            .parse_next(i)
    }
}

fn record_like_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    let omit_named = move |i: &mut Input<'s>| -> Result<Pattern<'s>> {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if p.is_bind() || p.is_unknown() {
                    p
                } else {
                    p.wrap_as_unknown(
                        t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                        DiagnosticCode::BadOmitKeyRecordPattern,
                    )
                }
            })
            .parse_next(i)
    };

    let unnamed = move |i: &mut Input<'s>| {
        alt((
            preceded(
                peek(one_of(|t: &Token<'s>| {
                    *t == Operator::Comma || *t == Operator::CloseParen
                })),
                pattern_or_insert(rebind),
            ),
            pattern(rebind),
        ))
        .parse_next(i)
    };

    move |i: &mut Input<'s>| {
        let (open, mut parts, close) = record_base(
            pattern_or_insert(rebind),
            |t| Pattern::unknown([t.into()], DiagnosticCode::InterpolatedNameRecordPattern),
            omit_named,
            unnamed,
            pattern_spread(rebind),
        )
        .parse_next(i)?;
        let len = parts.len();
        let result = if len == 1 && !parts[0].has_tail_comma() && parts[0].is_unnamed() {
            let RecordElementBase::Unnamed(part) = parts.into_iter().next().unwrap().unwrap()
            else {
                unreachable!();
            };
            Pattern::Grouping(open, part, close)
        } else {
            for (i, part) in parts.iter_mut().enumerate() {
                if !part.is_spread() {
                    continue;
                }
                let ListItem(el, _) = part;
                let range = el.range();
                let RecordElementBase::Spread(token, pattern) = el.as_mut() else {
                    unreachable!();
                };
                if pattern.is_spread_discard() {
                    *token = Token::unknown_range(
                        token.range(),
                        token.kind.to_owned(),
                        range,
                        DiagnosticCode::SpreadDiscardInRecordPattern,
                    )
                    .into();
                } else if i != len - 1 {
                    *token = Token::unknown_range(
                        token.range(),
                        token.kind.to_owned(),
                        range,
                        DiagnosticCode::MispositionedSpreadInRecordPattern,
                    )
                    .into();
                }
            }
            Pattern::Record(open, parts, close)
        };
        Ok(result)
    }
}

pub(crate) fn array_pattern_like<'s>(
    brace: [Operator; 2],
    rebind: bool,
) -> impl Parser<'s, Pattern<'s>> {
    let element_pattern = move |i: &mut Input<'s>| {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if matches!(p, Pattern::Range(..)) {
                    p.wrap_as_unknown(
                        t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                        DiagnosticCode::AmbiguousRangePattern,
                    )
                } else {
                    p
                }
            })
            .parse_next(i)
    };
    move |i: &mut Input<'s>| {
        let (open, parts, close) =
            array_base(brace, element_pattern, fail, pattern_spread(rebind)).parse_next(i)?;
        Ok(Pattern::Array(open, parts, close))
    }
}

fn array_pattern<'s>(rebind: bool) -> impl Parser<'s, Pattern<'s>> {
    move |i: &mut Input<'s>| -> Result<Pattern<'s>> {
        let mut p = array_pattern_like([Operator::OpenBracket, Operator::CloseBracket], rebind)
            .parse_next(i)?;
        let Pattern::Array(_, parts, _) = &mut p else {
            unreachable!();
        };
        let mut spread = vec![];
        for part in parts.iter_mut() {
            if part.is_spread() {
                spread.push(part.deref_mut());
            }
        }
        if spread.len() > 1 {
            for part in spread.into_iter().skip(1) {
                let ArrayElementBase::Spread(kw, p) = part else {
                    unreachable!();
                };
                *part = ArrayElementBase::Element(Box::new(
                    p.to_owned()
                        .wrap_as_unknown([kw.clone()], DiagnosticCode::DuplicateSpreadPattern),
                ));
            }
        }

        Ok(p)
    }
}

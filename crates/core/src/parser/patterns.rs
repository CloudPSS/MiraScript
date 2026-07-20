use std::ops::DerefMut;

use winnow::{
    combinator::{alt, opt, peek, separated_foldl1, seq},
    error::EmptyError,
    token::{any, one_of, take_till},
};

use crate::parser::helper::token_or_insert;

use super::{
    array_helper::array_base,
    helper::{literal_token, token, variable_token},
    prelude::*,
    record_helper::record_base,
};

fn unknown_pattern<'s, 'a>(_arena: &'a AstArena, i: &mut Input<'s>) -> Result<Pattern<'s, 'a>> {
    take_till(1.., |t: &Token<'s>| {
        *t == TokenKind::Eof
            || *t == Keyword::If
            || *t == Keyword::Fn
            || *t == Keyword::Loop
            || *t == Keyword::While
            || *t == Keyword::Match
            || *t == Keyword::For
            || *t == Keyword::Let
            || *t == Keyword::Const
            || *t == Operator::Comma
            || *t == Operator::Assign
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

pub(super) fn pattern_or_insert<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
    mut insert_cond: impl FnMut(&'s Token<'s>) -> bool + Copy,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        let p = opt(pattern(arena, rebind)).parse_next(i)?;
        if let Some(p) = p {
            return Ok(p);
        }
        let start = i.previous_token_end();
        let next = peek(any).parse_next(i)?;
        if *next != TokenKind::Eof && !insert_cond(next) {
            return Err(winnow::error::ErrMode::Backtrack(EmptyError));
        }
        Ok(pattern_expected(start))
    }
}

pub(super) fn pattern_expected<'s, 'a>(pos: usize) -> Pattern<'s, 'a> {
    Pattern::unknown_range(
        vec![Token::empty(pos).into()],
        pos..pos,
        DiagnosticCode::PatternExpected,
    )
}

pub(super) fn pattern<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| or_pattern(arena, rebind).parse_next(i)
}

fn primary_pattern<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        alt((
            |i: &mut Input<'s>| relation_pattern(arena, i),
            record_like_pattern(arena, rebind),
            array_pattern(arena, rebind),
            |i: &mut Input<'s>| range_pattern(arena, i),
            |i: &mut Input<'s>| literal_constant_pattern::<false>(arena, i),
            discard_bind_pattern(arena, rebind),
            not_pattern(arena, rebind),
            |i: &mut Input<'s>| unknown_pattern(arena, i),
        ))
        .parse_next(i)
    }
}

fn not_pattern<'s: 'a, 'a>(arena: &'a AstArena, rebind: bool) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        (token(Keyword::Not), primary_pattern(arena, rebind))
            .map(|(kw_not, p)| Pattern::Not(kw_not, arena.alloc(p)))
            .parse_next(i)
    }
}

fn and_pattern<'s: 'a, 'a>(arena: &'a AstArena, rebind: bool) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        separated_foldl1(
            primary_pattern(arena, rebind),
            token(Keyword::And),
            |left, op, right| Pattern::And(arena.alloc(left), op, arena.alloc(right)),
        )
        .parse_next(i)
    }
}

fn or_pattern<'s: 'a, 'a>(arena: &'a AstArena, rebind: bool) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        separated_foldl1(
            and_pattern(arena, rebind),
            token(Keyword::Or),
            |left, op, right| Pattern::Or(arena.alloc(left), op, arena.alloc(right)),
        )
        .parse_next(i)
    }
}

fn literal_constant_pattern<'s, 'a, const CUT: bool>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Pattern<'s, 'a>> {
    let literal_pattern = |i: &mut Input<'s>| -> Result<Pattern<'s, 'a>> {
        (
            opt(one_of(|t: &Token<'s>| {
                *t == Operator::Plus || *t == Operator::Minus || *t == Operator::Exclamation
            })),
            literal_token,
        )
            .map(|(t, l)| {
                let Some(op) = t else {
                    return Pattern::Literal(None, l);
                };
                if *op == Operator::Exclamation {
                    return Pattern::Literal(None, l.clone()).wrap_as_unknown(
                        arena,
                        [op.into(), l],
                        DiagnosticCode::ExclamationInLiteralPattern,
                    );
                }
                if !(l.is_number_literal()) {
                    return Pattern::Literal(None, l.clone()).wrap_as_unknown(
                        arena,
                        [op.into(), l],
                        DiagnosticCode::NonNumberInArithmetic,
                    );
                }
                Pattern::Literal(Some(op.into()), l)
            })
            .parse_next(i)
    };

    let constant_pattern = |i: &mut Input<'s>| -> Result<Pattern<'s, 'a>> {
        variable_token(false, false)
            .verify_map(|t: TokenRef<'s>| {
                let name = t.to_id_name()?;
                let is_const = name.starts_with('@');
                Some(if !is_const {
                    if CUT {
                        Pattern::Constant(t.clone()).wrap_as_unknown(
                            arena,
                            [t],
                            DiagnosticCode::InvalidConstantLiteral,
                        )
                    } else {
                        return None;
                    }
                } else {
                    Pattern::Constant(t)
                })
            })
            .parse_next(i)
    };
    alt((constant_pattern, literal_pattern)).parse_next(i)
}

fn relation_pattern<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Pattern<'s, 'a>> {
    seq!(Pattern::Relation(
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Operator(op) if op.is_relation()))
            .map(TokenRef::borrow),
        (|i: &mut Input<'s>| literal_constant_pattern::<true>(arena, i)).map(|p| arena.alloc(p)),
    ))
    .parse_next(i)
}

fn range_pattern<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Pattern<'s, 'a>> {
    fn range_guard<'s, 'a>(arena: &'a AstArena, p: Pattern<'s, 'a>) -> ABox<'a, Pattern<'s, 'a>> {
        arena.alloc(match p {
            Pattern::Literal(op, l) => {
                if l.is_number_literal() {
                    Pattern::Literal(op, l)
                } else {
                    let tokens = match op {
                        Some(op) => vec![op, l],
                        _ => vec![l],
                    };
                    Pattern::unknown(tokens, DiagnosticCode::NonNumberInRange)
                }
            }
            _ => p,
        })
    }
    seq!(Pattern::Range(
        (|i: &mut Input<'s>| literal_constant_pattern::<true>(arena, i))
            .map(|p| range_guard(arena, p)),
        one_of(|t: &Token<'s>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(TokenRef::borrow),
        (|i: &mut Input<'s>| literal_constant_pattern::<true>(arena, i))
            .map(|p| range_guard(arena, p)),
    ))
    .parse_next(i)
}

fn discard_bind_pattern<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        (opt(token(Keyword::Mut)), variable_token(true, false))
            .map(|(kw_mut, id)| {
                if id.kind == Keyword::Underscore {
                    if let Some(kw_mut) = kw_mut {
                        return Pattern::unknown(
                            vec![kw_mut, id],
                            DiagnosticCode::MutInDiscardPattern,
                        );
                    }
                    return Pattern::Discard(id);
                }
                if id.is_unknown() {
                    let tokens = if let Some(kw_mut) = kw_mut {
                        vec![kw_mut, id]
                    } else {
                        vec![id]
                    };
                    return Pattern::unknown_errors(tokens, vec![]);
                }
                let Some(name) = id.to_id_name() else {
                    let tokens = if let Some(kw_mut) = kw_mut {
                        vec![kw_mut, id]
                    } else {
                        vec![id]
                    };
                    return Pattern::unknown(tokens, DiagnosticCode::UnknownPattern);
                };
                if rebind && let Some(mut kw_mut) = kw_mut {
                    kw_mut.wrap_as_unknown(DiagnosticCode::MutInRebindPattern);
                    return Pattern::Bind(Some(kw_mut), id);
                }
                if name.starts_with('@') {
                    let tokens = vec![kw_mut.clone().unwrap(), id.clone()];
                    return Pattern::Bind(kw_mut, id).wrap_as_unknown(
                        arena,
                        tokens,
                        DiagnosticCode::ConstantInBindPattern,
                    );
                }
                Pattern::Bind(kw_mut, id)
            })
            .parse_next(i)
    }
}

fn pattern_spread<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| {
        let pos = i.previous_token_end();
        opt(pattern(arena, rebind))
            .map(|p| {
                let Some(p) = p else {
                    return Pattern::SpreadDiscard(pos);
                };
                if let Pattern::Discard(t) = &p {
                    let tokens = [t.clone()];
                    p.wrap_as_unknown(arena, tokens, DiagnosticCode::DiscardInSpreadPattern)
                } else {
                    p
                }
            })
            .parse_next(i)
    }
}

fn record_like_pattern<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    let omit_named = move |i: &mut Input<'s>| -> Result<Pattern<'s, 'a>> {
        // omit named 会消费 `:`，可以无条件插入
        pattern_or_insert(arena, rebind, |_| true)
            .with_taken()
            .map(|(p, t)| {
                if p.is_bind() || p.is_unknown() {
                    p
                } else {
                    p.wrap_as_unknown(
                        arena,
                        t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                        DiagnosticCode::BadOmitKeyRecordPattern,
                    )
                }
            })
            .parse_next(i)
    };

    // unnamed 不额外消费字符，仅在分隔符前插入
    let unnamed = move |i: &mut Input<'s>| {
        pattern_or_insert(arena, rebind, |t| {
            *t == Operator::Comma || *t == Operator::CloseParen
        })
        .parse_next(i)
    };

    move |i: &mut Input<'s>| {
        let (open, mut parts, close) = record_base(
            arena,
            // named 会消费 name `:`，可以无条件插入
            pattern_or_insert(arena, rebind, |_| true),
            |t| Pattern::unknown([t.into()], DiagnosticCode::InterpolatedNameRecordPattern),
            omit_named,
            unnamed,
            pattern_spread(arena, rebind),
            pattern_expected,
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
                let RecordElementBase::Spread(token, _) = el.as_mut() else {
                    unreachable!();
                };
                if i != len - 1 {
                    token.wrap_as_unknown(DiagnosticCode::MispositionedSpreadInRecordPattern);
                }
            }
            Pattern::Record(open, parts, close)
        };
        Ok(result)
    }
}

pub(crate) fn array_pattern_like<'s: 'a, 'a>(
    arena: &'a AstArena,
    open: impl Parser<'s, TokenRef<'s>>,
    close: impl Parser<'s, TokenRef<'s>>,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    let element_pattern = move |i: &mut Input<'s>| {
        pattern_or_insert(arena, rebind, |t| {
            *t == Operator::Comma || *t == Operator::CloseBracket
        })
        .with_taken()
        .map(|(p, t)| {
            if matches!(p, Pattern::Range(..)) {
                p.wrap_as_unknown(
                    arena,
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
        let (open, parts, close) = array_base(
            arena,
            open,
            close,
            element_pattern,
            pattern_spread(arena, rebind),
            pattern_expected,
        )
        .parse_next(i)?;
        Ok(Pattern::Array(open, parts, close))
    }
}

fn array_pattern<'s: 'a, 'a>(
    arena: &'a AstArena,
    rebind: bool,
) -> impl Parser<'s, Pattern<'s, 'a>> {
    move |i: &mut Input<'s>| -> Result<Pattern<'s, 'a>> {
        let mut p = array_pattern_like(
            arena,
            token(Operator::OpenBracket),
            token_or_insert(Operator::CloseBracket, DiagnosticCode::MissingCloseBracket),
            rebind,
        )
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
                let pattern = std::mem::replace(&mut **p, Pattern::SpreadDiscard(kw.range.start));
                *part = ArrayElementBase::Element(arena.alloc(pattern.wrap_as_unknown(
                    arena,
                    [kw.clone()],
                    DiagnosticCode::DuplicateSpreadPattern,
                )));
            }
        }

        Ok(p)
    }
}

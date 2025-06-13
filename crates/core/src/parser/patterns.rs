use std::ops::DerefMut;

use winnow::{
    ModalResult, Parser,
    combinator::{alt, empty, fail, opt, peek, preceded, separated_foldl1, seq},
    error::{ContextError, ErrMode},
    stream::Location,
    token::{one_of, take_till},
};

use crate::{
    diagnostic::{DiagnosticCode, SourceRange},
    lexer::{Keyword, Operator, Token, TokenKind},
    parser::record_element::RecordElementBase,
};

use super::{
    ArrayElementBase, AstWalker, Input, Pattern,
    array_helper::array_base,
    helper::{literal_token, token_boxed, variable_token},
    list_item::ListItem,
    record_helper::record_base,
};

fn unknown_pattern<'s>(i: &mut Input<'_, 's>) -> ModalResult<Pattern<'s>> {
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
    .map(|t: &[Token<'s>]| Pattern::unknown(t, DiagnosticCode::UnknownPattern))
    .parse_next(i)
}

pub(super) fn pattern_or_insert<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'_, 's>| {
        let start = i.previous_token_end();
        alt((
            pattern(rebind),
            empty.map(|_| {
                Pattern::unknown_range(
                    vec![Token::empty(start)],
                    start..start,
                    DiagnosticCode::PatternExpected,
                )
            }),
        ))
        .parse_next(i)
    }
}

pub(super) fn pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'_, 's>| or_pattern(rebind).parse_next(i)
}

fn primary_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 's>| {
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

fn not_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 's>| {
        (token_boxed(Keyword::Not), primary_pattern(rebind))
            .map(|(kw_not, p)| Pattern::Not(kw_not, Box::new(p)))
            .parse_next(i)
    }
}

fn and_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 's>| {
        separated_foldl1(
            primary_pattern(rebind),
            token_boxed(Keyword::And),
            |left, op, right| Pattern::And(Box::new(left), op, Box::new(right)),
        )
        .parse_next(i)
    }
}

fn or_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 's>| {
        separated_foldl1(
            and_pattern(rebind),
            token_boxed(Keyword::Or),
            |left, op, right| Pattern::Or(Box::new(left), op, Box::new(right)),
        )
        .parse_next(i)
    }
}

fn constants_pattern<'s>(i: &mut Input<'_, 's>) -> ModalResult<Pattern<'s>> {
    (
        opt(one_of(|t: &Token<'s>| {
            *t == Operator::Plus || *t == Operator::Minus || *t == Operator::Exclamation
        })),
        literal_token,
    )
        .map(|(t, l)| {
            let Some(op) = t else {
                return Pattern::Constant(None, Box::new(l));
            };
            if *op == Operator::Exclamation {
                return Pattern::Constant(None, Box::from(l.clone())).wrap_as_unknown(
                    [op.to_owned(), l],
                    DiagnosticCode::ExclamationInConstantsPattern,
                );
            }
            if !(l.is_number() || l.is_ordinal() || l == Keyword::Inf) {
                return Pattern::Constant(None, Box::from(l.clone())).wrap_as_unknown(
                    [op.to_owned(), l.clone()],
                    DiagnosticCode::UnexpectedOperatorInConstantsPattern,
                );
            }
            Pattern::Constant(Some(op.to_owned().into()), Box::new(l.clone()))
        })
        .parse_next(i)
}

fn relation_pattern<'s>(i: &mut Input<'_, 's>) -> ModalResult<Pattern<'s>> {
    seq!(Pattern::Relation(
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Operator(op) if op.is_relation()))
            .map(|t: &Token<'s>| Box::new(t.to_owned())),
        constants_pattern.map(Box::new),
    ))
    .parse_next(i)
}

fn range_pattern<'s>(i: &mut Input<'_, 's>) -> ModalResult<Pattern<'s>> {
    seq!(Pattern::Range(
        constants_pattern.map(Box::new),
        one_of(|t: &Token<'s>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(|t: &Token<'s>| Box::new(t.to_owned())),
        constants_pattern.map(Box::new),
    ))
    .parse_next(i)
}

fn discard_bind_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 's>| {
        (opt(token_boxed(Keyword::Mut)), variable_token(true, false))
            .map(|(kw_mut, id)| {
                if rebind && kw_mut.is_some() {
                    let kw_mut = kw_mut
                        .unwrap()
                        .wrap_as_unknown(DiagnosticCode::MutInRebindPattern);
                    return Pattern::Bind(Some(Box::new(kw_mut)), Box::new(id));
                }
                if id.kind == Keyword::Underscore {
                    if kw_mut.is_some() {
                        return Pattern::unknown(
                            vec![*kw_mut.unwrap(), id],
                            DiagnosticCode::MutInDiscardPattern,
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

fn pattern_spread<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'_, 's>| {
        let pos = i.previous_token_end();
        opt(pattern(rebind))
            .with_taken()
            .map(|(p, t)| {
                if p.is_none() {
                    return Pattern::SpreadDiscard(pos);
                }
                let p = p.unwrap();
                if p.is_discard() {
                    p.wrap_as_unknown(t, DiagnosticCode::DiscardInSpreadPattern)
                } else {
                    p
                }
            })
            .parse_next(i)
    }
}

fn record_like_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> + Copy {
    let omit_named = move |i: &mut Input<'_, 's>| -> ModalResult<Pattern<'s>> {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if p.is_bind() || p.is_unknown() {
                    p
                } else {
                    p.wrap_as_unknown(t, DiagnosticCode::BadOmitKeyRecordPattern)
                }
            })
            .parse_next(i)
    };

    let unnamed = move |i: &mut Input<'_, 's>| {
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

    move |i: &mut Input<'_, 's>| {
        let (open, mut parts, close) = record_base(
            pattern_or_insert(rebind),
            |t| {
                Pattern::unknown(
                    [t.to_owned()],
                    DiagnosticCode::InterpolatedNameRecordPattern,
                )
            },
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
                    *token.as_mut() = Token::unknown_range(
                        token.range(),
                        token.kind.to_owned(),
                        range,
                        DiagnosticCode::SpreadDiscardInRecordPattern,
                    );
                } else if i != len - 1 {
                    *token.as_mut() = Token::unknown_range(
                        token.range(),
                        token.kind.to_owned(),
                        range,
                        DiagnosticCode::MispositionedSpreadInRecordPattern,
                    );
                }
            }
            Pattern::Record(open, parts, close)
        };
        Ok(result)
    }
}

pub(crate) fn array_pattern_like<'t, 's: 't>(
    brace: [Operator; 2],
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> + Copy {
    let element_pattern = move |i: &mut Input<'_, 's>| {
        pattern_or_insert(rebind)
            .with_taken()
            .map(|(p, t)| {
                if matches!(p, Pattern::Range(..)) {
                    p.wrap_as_unknown(t, DiagnosticCode::AmbiguousRangePattern)
                } else {
                    p
                }
            })
            .parse_next(i)
    };
    move |i: &mut Input<'_, 's>| {
        let (open, parts, close) =
            array_base(brace, element_pattern, fail, pattern_spread(rebind)).parse_next(i)?;
        Ok(Pattern::Array(open, parts, close))
    }
}

fn array_pattern<'t, 's: 't>(
    rebind: bool,
) -> impl Parser<Input<'t, 's>, Pattern<'s>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'t, 's>| -> ModalResult<Pattern<'s>> {
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
                let kw = kw.to_owned().deref_mut().clone();
                *part = ArrayElementBase::Element(Box::new(
                    p.to_owned()
                        .wrap_as_unknown([kw], DiagnosticCode::DuplicateSpreadPattern),
                ));
            }
        }

        Ok(p)
    }
}

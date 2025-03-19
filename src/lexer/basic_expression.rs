use winnow::combinator::{alt, dispatch, fail, opt, peek, preceded, repeat, seq, terminated};
use winnow::prelude::*;
use winnow::token::{any, one_of};

use crate::tokenizer::{Keyword, Operator, Range, Token, TokenError, TokenKind};

use super::expression::expression;
use super::{Expression, Input, TokenRef};

fn literal<'a, V: PartialEq<Token<'a>>>(
    op: V,
) -> impl Fn(&mut Input<'a>) -> ModalResult<TokenRef<'a>> {
    move |i: &mut Input<'a>| one_of(|t: TokenRef<'a>| op == *t).parse_next(i)
}

fn value<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    one_of(|t: TokenRef<'a>| {
        matches!(&t.kind, &TokenKind::Identifier(_))
            || matches!(&t.kind, &TokenKind::Number(_))
            || matches!(&t.kind, &TokenKind::String(_))
    })
    .map(Expression::Value)
    .parse_next(i)
}

struct TupleLikePart<'a> {
    name: Option<TokenRef<'a>>,
    value: Expression<'a>,
    range: Range,
    tail_comma: bool,
}

fn tuple_like_part<'a>(i: &mut Input<'a>) -> ModalResult<TupleLikePart<'a>> {
    let first = peek(any).parse_next(i)?;
    if *first == Operator::CloseParen {
        return fail.parse_next(i);
    }
    (
        opt(terminated(
            one_of(|t: TokenRef<'a>| {
                matches!(
                    t.kind,
                    TokenKind::Identifier(_) | TokenKind::Ordinal(_) | TokenKind::Number(_)
                )
            }),
            literal(Operator::Colon),
        )),
        expression,
        alt((
            literal(Operator::Comma),
            peek(literal(Operator::CloseParen)),
        )),
    )
        .with_taken()
        .map(|((name, exp, tail), taken)| {
            let tail_comma = *tail == Operator::Comma;
            let mut range = taken[0].range.clone();
            range.end = tail.range.start;
            TupleLikePart {
                name,
                value: exp,
                range,
                tail_comma,
            }
        })
        .parse_next(i)
}

fn tuple_like<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    let next = peek(any).parse_next(i)?;
    if *next == Operator::CloseParen {
        any.parse_next(i)?;
        return Ok(Expression::Tuple(Vec::new()));
    }
    let parts: Vec<TupleLikePart<'a>> = repeat(1.., tuple_like_part).parse_next(i)?;
    one_of(|t: TokenRef<'a>| *t == Operator::CloseParen).parse_next(i)?;
    if parts.len() == 1 {
        let part = parts.into_iter().next().unwrap();
        if let Some(name) = part.name {
            return Ok(Expression::NamedTuple(vec![(name, part.value)]));
        }
        if part.tail_comma {
            return Ok(Expression::Tuple(vec![part.value]));
        }
        return Ok(Expression::Grouping(Box::new(part.value)));
    }
    if parts.iter().all(|part| part.name.is_none()) {
        return Ok(Expression::Tuple(
            parts.into_iter().map(|part| part.value).collect(),
        ));
    }

    let mut items: Vec<(TokenRef<'a>, Expression<'a>)> = Vec::new();
    for (pos, part) in parts.into_iter().enumerate() {
        if let Some(name) = part.name {
            items.push((name, part.value));
            continue;
        }
        let mut range = part.range;
        range.end = range.start;
        let recovered = TokenKind::Ordinal(pos as u64);
        let token = Token::unknown(
            range.clone(),
            recovered,
            vec![TokenError::new(
                range,
                "All tuple elements must be named or unnamed",
            )],
        );
        let name = i.state.add_token(token);
        items.push((name, part.value));
    }
    Ok(Expression::NamedTuple(items))
}

fn primary<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    (alt((value, preceded(literal(Operator::OpenParen), tuple_like)))).parse_next(i)
}

fn arg_list<'a>(i: &mut Input<'a>) -> ModalResult<Vec<Expression<'a>>> {
    terminated(
        (
            repeat(0.., terminated(expression, literal(Operator::Comma))),
            opt(expression),
        )
            .map(
                |(mut v, e): (Vec<Expression<'a>>, Option<Expression<'a>>)| {
                    if let Some(e) = e {
                        v.push(e);
                    }
                    v
                },
            ),
        literal(Operator::CloseParen),
    )
    .parse_next(i)
}

fn function<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    enum Function<'a> {
        Call(Vec<Expression<'a>>),
        Access(TokenRef<'a>),
    }
    let first = primary.parse_next(i)?;
    let functions: Vec<Function<'a>> = repeat(
        0..,
        alt((
            preceded(literal(Operator::OpenParen), arg_list).map(Function::Call),
            preceded(
                literal(Operator::Dot),
                one_of(|t: TokenRef<'a>| {
                    matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_))
                }),
            )
            .map(Function::Access),
        )),
    )
    .fold(Vec::new, |mut v, t| {
        v.push(t);
        v
    })
    .parse_next(i)?;
    if functions.is_empty() {
        return Ok(first);
    }
    // left-associative
    Ok(functions.into_iter().fold(first, |acc, exp| match exp {
        Function::Call(args) => Expression::Call(Box::new(acc), args),
        Function::Access(token) => Expression::Access(Box::new(acc), token),
    }))
}

fn unary<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus => seq!(Expression::Plus(_: any, unary.map(Box::new))),
        token if *token == Operator::Minus => seq!(Expression::Negate(_: any, unary.map(Box::new))),
        token if *token == Keyword::not =>seq!(Expression::Not(_: any, unary.map(Box::new))),
        &Token{..} => function,
    }
    .parse_next(i)
}

fn exponent<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    let first = unary.parse_next(i)?;
    let mut exponents: Vec<Expression<'a>> = repeat(0.., preceded(literal(Operator::Caret), unary))
        .fold(Vec::new, |mut v, t| {
            v.push(t);
            v
        })
        .parse_next(i)?;
    if exponents.is_empty() {
        return Ok(first);
    }
    // right-associative
    let last = exponents.pop().unwrap();
    exponents.reverse();
    exponents.push(first);
    Ok(exponents.into_iter().fold(last, |acc, exp| {
        Expression::Exponent(Box::new(exp), Box::new(acc))
    }))
}

fn factor<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    let first = exponent.parse_next(i)?;
    let factors: Vec<(TokenRef<'a>, Expression<'a>)> = repeat(
        0..,
        (
            one_of(|t: TokenRef<'a>| {
                *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent
            }),
            exponent,
        ),
    )
    .fold(Vec::new, |mut v, t| {
        v.push(t);
        v
    })
    .parse_next(i)?;
    if factors.is_empty() {
        return Ok(first);
    }
    // left-associative
    Ok(factors
        .into_iter()
        .fold(first, |acc, (op, exp)| match op.kind {
            TokenKind::Operator(Operator::Asterisk) => {
                Expression::Multiply(Box::new(acc), Box::new(exp))
            }
            TokenKind::Operator(Operator::Slash) => {
                Expression::Divide(Box::new(acc), Box::new(exp))
            }
            TokenKind::Operator(Operator::Percent) => {
                Expression::Modulo(Box::new(acc), Box::new(exp))
            }
            _ => unreachable!(),
        }))
}

fn term<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    let first = factor.parse_next(i)?;
    let terms: Vec<(TokenRef<'a>, Expression<'a>)> = repeat(
        0..,
        (
            one_of(|t: TokenRef<'a>| *t == Operator::Plus || *t == Operator::Minus),
            factor,
        ),
    )
    .fold(Vec::new, |mut v, t| {
        v.push(t);
        v
    })
    .parse_next(i)?;
    if terms.is_empty() {
        return Ok(first);
    }
    // left-associative
    Ok(terms
        .into_iter()
        .fold(first, |acc, (op, exp)| match op.kind {
            TokenKind::Operator(Operator::Plus) => Expression::Add(Box::new(acc), Box::new(exp)),
            TokenKind::Operator(Operator::Minus) => {
                Expression::Subtract(Box::new(acc), Box::new(exp))
            }
            _ => unreachable!(),
        }))
}

fn and<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    let first = term.parse_next(i)?;
    let ands: Vec<(TokenRef<'a>, Expression<'a>)> =
        repeat(0.., (one_of(|t: TokenRef<'a>| *t == Keyword::and), term))
            .fold(Vec::new, |mut v, t| {
                v.push(t);
                v
            })
            .parse_next(i)?;
    if ands.is_empty() {
        return Ok(first);
    }
    // left-associative
    Ok(ands
        .into_iter()
        .fold(first, |acc, (op, exp)| match op.kind {
            TokenKind::Keyword(Keyword::and) => Expression::And(Box::new(acc), Box::new(exp)),
            _ => unreachable!(),
        }))
}

pub(super) fn or<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    let first = and.parse_next(i)?;
    let ors: Vec<(TokenRef<'a>, Expression<'a>)> =
        repeat(0.., (one_of(|t: TokenRef<'a>| *t == Keyword::or), and))
            .fold(Vec::new, |mut v, t| {
                v.push(t);
                v
            })
            .parse_next(i)?;
    if ors.is_empty() {
        return Ok(first);
    }
    // left-associative
    Ok(ors.into_iter().fold(first, |acc, (op, exp)| match op.kind {
        TokenKind::Keyword(Keyword::or) => Expression::Or(Box::new(acc), Box::new(exp)),
        _ => unreachable!(),
    }))
}

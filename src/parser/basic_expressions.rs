use winnow::combinator::{alt, dispatch, fail, opt, peek, preceded, repeat, seq, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Keyword, Operator, Range, Token, TokenKind};

use super::block_expressions::block_like_expression;
use super::expressions::expression;
use super::helper::{interpolation_token, literal_token, variable_token};
use super::{Expression, Input};

struct TupleLikePart<'a> {
    name: Option<Token<'a>>,
    value: Expression<'a>,
    range: Range,
    tail_comma: bool,
}

fn tuple_like_part<'t, 'a: 't, V>(
    close: V,
) -> impl Parser<Input<'t, 'a>, TupleLikePart<'a>, ErrMode<ContextError>>
where
    Token<'a>: PartialEq<V> + PartialEq + PartialEq<Operator>,
{
    move |i: &mut Input<'t, 'a>| {
        let first = peek(any).parse_next(i)?;
        if *first == close {
            return fail.parse_next(i);
        }
        (
            opt(terminated(
                one_of(|t: &Token<'a>| {
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
                let tail_comma = tail[0] == Operator::Comma;
                let mut range = taken[0].range.clone();
                range.end = tail[0].range.start;
                TupleLikePart {
                    name: name.cloned(),
                    value: exp,
                    range,
                    tail_comma,
                }
            })
            .parse_next(i)
    }
}

fn tuple_like<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let next = peek(any).parse_next(i)?;
    if *next == Operator::CloseParen {
        any.parse_next(i)?;
        return Ok(Expression::Tuple(Vec::new()));
    }
    let parts: Vec<TupleLikePart<'a>> =
        repeat(1.., tuple_like_part(Operator::CloseParen)).parse_next(i)?;
    one_of(|t: &Token<'a>| *t == Operator::CloseParen).parse_next(i)?;
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

    let mut items: Vec<(Token<'a>, Expression<'a>)> = Vec::new();
    for (pos, part) in parts.into_iter().enumerate() {
        if let Some(name) = part.name {
            items.push((name, part.value));
            continue;
        }
        let mut range = part.range;
        range.end = range.start;
        let recovered = TokenKind::Ordinal(pos as u64);
        let token = Token::unknown(
            range,
            recovered,
            "All tuple elements must be named or unnamed",
        );
        items.push((token, part.value));
    }
    Ok(Expression::NamedTuple(items))
}

fn array<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let elements: Vec<Expression<'a>> = terminated(
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
        literal(Operator::CloseBracket),
    )
    .parse_next(i)?;
    Ok(Expression::Array(elements))
}

fn primary<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    (alt((
        block_like_expression,
        literal_token.map(Box::new).map(Expression::Literal),
        interpolation_token
            .map(Box::new)
            .map(Expression::InterpolatedString),
        variable_token(false, true)
            .map(Box::new)
            .map(Expression::Variable),
        preceded(literal(Operator::OpenParen), tuple_like),
        preceded(literal(Operator::OpenBracket), array),
    )))
    .parse_next(i)
}

fn arg_list<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Vec<Expression<'a>>> {
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

fn function<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    enum Function<'a> {
        Call(Vec<Expression<'a>>),
        Access(Box<Token<'a>>),
    }
    let first = primary.parse_next(i)?;
    let functions: Vec<Function<'a>> = repeat(
        0..,
        alt((
            preceded(literal(Operator::OpenParen), arg_list).map(Function::Call),
            preceded(
                literal(Operator::Dot),
                one_of(|t: &Token<'a>| {
                    matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_))
                })
                .map(|t: &Token<'a>| Box::new(t.to_owned())),
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

fn unary<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus => seq!(Expression::Plus(_: any, unary.map(Box::new))),
        token if *token == Operator::Minus => seq!(Expression::Negate(_: any, unary.map(Box::new))),
        token if *token == Keyword::Not =>seq!(Expression::Not(_: any, unary.map(Box::new))),
        &Token{..} => function,
    }
    .parse_next(i)
}

fn exponent<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
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

fn factor<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let first = exponent.parse_next(i)?;
    let factors: Vec<(&Token<'a>, Expression<'a>)> = repeat(
        0..,
        (
            one_of(|t: &Token<'a>| {
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

fn term<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let first = factor.parse_next(i)?;
    let terms: Vec<(&Token<'a>, Expression<'a>)> = repeat(
        0..,
        (
            one_of(|t: &Token<'a>| *t == Operator::Plus || *t == Operator::Minus),
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

fn comparison<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let first = term.parse_next(i)?;
    let comparisons: Vec<(&Token<'a>, Expression<'a>)> = repeat(
        0..,
        (
            one_of(|t: &Token<'a>| {
                *t == Operator::EqualEqual
                    || *t == Operator::NotEqual
                    || *t == Operator::Less
                    || *t == Operator::LessEqual
                    || *t == Operator::Greater
                    || *t == Operator::GreaterEqual
            }),
            term,
        ),
    )
    .fold(Vec::new, |mut v, t| {
        v.push(t);
        v
    })
    .parse_next(i)?;
    if comparisons.is_empty() {
        return Ok(first);
    }
    // left-associative
    Ok(comparisons.into_iter().fold(first, |acc, (t, exp)| {
        if let Token {
            kind: TokenKind::Operator(op),
            ..
        } = t
        {
            match op {
                Operator::EqualEqual => Expression::Equal(Box::new(acc), Box::new(exp)),
                Operator::NotEqual => Expression::NotEqual(Box::new(acc), Box::new(exp)),
                Operator::Less => Expression::Less(Box::new(acc), Box::new(exp)),
                Operator::LessEqual => Expression::LessEqual(Box::new(acc), Box::new(exp)),
                Operator::Greater => Expression::Greater(Box::new(acc), Box::new(exp)),
                Operator::GreaterEqual => Expression::GreaterEqual(Box::new(acc), Box::new(exp)),
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }
    }))
}

fn and<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let first = comparison.parse_next(i)?;
    let ands: Vec<(&Token<'a>, Expression<'a>)> = repeat(
        0..,
        (one_of(|t: &Token<'a>| *t == Keyword::And), comparison),
    )
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
            TokenKind::Keyword(Keyword::And) => Expression::And(Box::new(acc), Box::new(exp)),
            _ => unreachable!(),
        }))
}

fn or<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    let first = and.parse_next(i)?;
    let ors: Vec<(&Token<'a>, Expression<'a>)> =
        repeat(0.., (one_of(|t: &Token<'a>| *t == Keyword::Or), and))
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
        TokenKind::Keyword(Keyword::Or) => Expression::Or(Box::new(acc), Box::new(exp)),
        _ => unreachable!(),
    }))
}

pub(super) fn basic_expression<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Expression<'a>> {
    or.parse_next(i)
}

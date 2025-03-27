use winnow::combinator::{
    alt, delimited, dispatch, fail, opt, peek, preceded, repeat, seq, terminated,
};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Operator, Token, TokenKind};
use crate::utils::SourceRange;

use super::block_expressions::block_like_expression;
use super::expressions::expression;
use super::helper::{interpolation_token, literal_token, variable_token};
use super::{Expression, Input};

struct TupleLikePart<'a> {
    name: Option<Token<'a>>,
    value: Expression<'a>,
    range: SourceRange,
    tail_comma: bool,
}

fn tuple_like_part<'t, 'a, V>(
    close: V,
) -> impl Parser<Input<'t, 'a>, TupleLikePart<'a>, ErrMode<ContextError>>
where
    Token<'a>: PartialEq<V> + PartialEq + PartialEq<Operator>,
{
    move |i: &mut Input<'_, 'a>| {
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

fn tuple_like<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
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

fn array<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
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

fn primary<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
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

fn arg_list<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Vec<Expression<'a>>> {
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

fn function<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    enum Function<'a> {
        Call(Vec<Expression<'a>>),
        Access(Box<Token<'a>>),
        Index(Expression<'a>),
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
            delimited(
                literal(Operator::OpenBracket),
                expression,
                literal(Operator::CloseBracket),
            )
            .map(Function::Index),
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
        Function::Index(index) => Expression::Index(Box::new(acc), Box::new(index)),
    }))
}

fn unary<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus => seq!(Expression::Plus(_: any, unary.map(Box::new))),
        token if *token == Operator::Minus => seq!(Expression::Negate(_: any, unary.map(Box::new))),
        token if *token == Operator::LogicalNot =>seq!(Expression::Not(_: any, unary.map(Box::new))),
        &Token{..} => function,
    }
    .parse_next(i)
}

fn exponent<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
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

fn left_associative_infix<'a, I, F, G>(
    i: &mut Input<'_, 'a>,
    mut item: I,
    filter: F,
    folder: G,
) -> ModalResult<Expression<'a>>
where
    I: for<'f> Parser<Input<'f, 'a>, Expression<'a>, ErrMode<ContextError>>,
    F: Fn(&Token<'a>) -> bool,
    G: Fn(Expression<'a>, (&Token<'a>, Expression<'a>)) -> Expression<'a>,
{
    let first = item.parse_next(i)?;
    let items: Vec<(&Token<'a>, Expression<'a>)> = repeat(0.., (one_of(filter), item))
        .fold(Vec::new, |mut v, t| {
            v.push(t);
            v
        })
        .parse_next(i)?;
    if items.is_empty() {
        return Ok(first);
    }
    Ok(items.into_iter().fold(first, folder))
}

fn factor<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        exponent,
        |t| *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent,
        |acc, (op, exp)| match op.kind {
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
        },
    )
}

pub(super) fn term<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        factor,
        |t| *t == Operator::Plus || *t == Operator::Minus,
        |acc, (op, exp)| match op.kind {
            TokenKind::Operator(Operator::Plus) => Expression::Add(Box::new(acc), Box::new(exp)),
            TokenKind::Operator(Operator::Minus) => {
                Expression::Subtract(Box::new(acc), Box::new(exp))
            }
            _ => unreachable!(),
        },
    )
}

fn comparison<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        term,
        |t| {
            *t == Operator::Less
                || *t == Operator::LessEqual
                || *t == Operator::Greater
                || *t == Operator::GreaterEqual
        },
        |acc, (t, exp)| {
            let Token {
                kind: TokenKind::Operator(op),
                ..
            } = t
            else {
                unreachable!();
            };
            match op {
                Operator::Less => Expression::Less(Box::new(acc), Box::new(exp)),
                Operator::LessEqual => Expression::LessEqual(Box::new(acc), Box::new(exp)),
                Operator::Greater => Expression::Greater(Box::new(acc), Box::new(exp)),
                Operator::GreaterEqual => Expression::GreaterEqual(Box::new(acc), Box::new(exp)),
                _ => unreachable!(),
            }
        },
    )
}

fn equality<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        comparison,
        |t| *t == Operator::EqualEqual || *t == Operator::NotEqual,
        |acc, (op, exp)| {
            if *op == Operator::EqualEqual {
                Expression::Equal(Box::new(acc), Box::new(exp))
            } else {
                Expression::NotEqual(Box::new(acc), Box::new(exp))
            }
        },
    )
}

fn and<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        equality,
        |t| *t == Operator::LogicalAnd,
        |acc, (_op, exp)| Expression::And(Box::new(acc), Box::new(exp)),
    )
}

fn or<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        and,
        |t| *t == Operator::LogicalOr,
        |acc, (_op, exp)| Expression::Or(Box::new(acc), Box::new(exp)),
    )
}

fn pipe<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        or,
        |t| *t == Operator::ForwardPipe || *t == Operator::BackwardPipe,
        |acc, (op, exp)| {
            if *op == Operator::ForwardPipe {
                Expression::ForwardPipe(Box::new(acc), Box::new(exp))
            } else {
                Expression::BackwardPipe(Box::new(acc), Box::new(exp))
            }
        },
    )
}

pub(super) fn basic_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    pipe.parse_next(i)
}

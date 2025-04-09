use winnow::combinator::{
    alt, delimited, dispatch, eof, opt, peek, preceded, repeat, seq, terminated,
};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::array_expression::array_expression;
use super::block_expressions::block_like_expression;
use super::expressions::expression;
use super::helper::{literal_token, token_boxed, variable_token};
use super::patterns::pattern;
use super::record_helper::record_base;
use super::{Expression, Input, RecordElement, to_input};

fn record_like<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let (open, parts, close) = record_base(
        expression,
        variable_token(false, false),
        expression,
        expression,
    )
    .parse_next(i)?;
    let result = if parts.len() == 1 {
        let part = parts.into_iter().next().unwrap();
        if let RecordElement::Unnamed(exp, None) = part {
            Expression::Grouping(open, exp, close)
        } else {
            Expression::Record(vec![part])
        }
    } else {
        Expression::Record(parts)
    };
    Ok(result)
}

pub(super) fn interpolation<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let token = one_of(|t: &Token<'a>| matches!(&t.kind, &TokenKind::InterpolatedString(_, _)))
        .map(|t: &Token<'a>| t.to_owned())
        .parse_next(i)?;

    let TokenKind::InterpolatedString(_, e) = &token.kind else {
        unreachable!("Expected InterpolatedString");
    };
    let expressions: Vec<_> = e
        .iter()
        .map(|tokens| {
            let expr: ModalResult<Expression<'_>> = {
                let mut token_input = to_input(tokens.as_slice());
                terminated(expression, eof).parse_next(&mut token_input)
            };
            let expr = match expr {
                Ok(expr) => expr,
                Err(_) => {
                    let last_token = tokens.last().unwrap();
                    let error = if *last_token == TokenKind::Eof {
                        "Unterminated interpolation expression"
                    } else {
                        "Bad interpolation expression"
                    };
                    Expression::unknown(tokens.clone(), error)
                }
            };
            expr
        })
        .collect();
    Ok(Expression::InterpolatedString(Box::new(token), expressions))
}

pub(super) fn primary<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    (alt((
        block_like_expression,
        literal_token.map(Box::new).map(Expression::Literal),
        interpolation,
        variable_token(false, true)
            .map(Box::new)
            .map(Expression::Variable),
        record_like,
        array_expression,
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

pub(super) fn access_call<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
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

pub(super) fn unary<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus || *token == Operator::Minus|| *token == Operator::LogicalNot || *token == Keyword::TypeOf =>
            seq!(Expression::Unary(any.map(|t: &Token<'a>| Box::new(t.to_owned())), unary.map(Box::new))),
        &Token{..} => access_call,
    }
    .parse_next(i)
}

pub(super) fn exponentiation<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let first = unary.parse_next(i)?;
    let exponents: Vec<_> = repeat(0.., (token_boxed(Operator::Caret), unary))
        .fold(Vec::new, |mut v, t| {
            v.push(t);
            v
        })
        .parse_next(i)?;
    if exponents.is_empty() {
        return Ok(first);
    }
    // right-associative
    let mut iter = exponents.into_iter();
    let (mut op, mut acc) = iter.next().unwrap();
    for (next_op, next_exp) in iter {
        acc = Expression::Binary(Box::new(acc), op, Box::new(next_exp));
        op = next_op;
    }
    Ok(Expression::Binary(Box::new(first), op, Box::new(acc)))
}

fn left_associative_infix<'t, 'a: 't, O>(
    i: &mut Input<'t, 'a>,
    mut item0: impl Parser<Input<'t, 'a>, Expression<'a>, ErrMode<ContextError>>,
    item: impl Parser<Input<'t, 'a>, O, ErrMode<ContextError>>,
    filter: impl Fn(&Token<'a>) -> bool,
    accumulator: impl Fn(Expression<'a>, Token<'a>, O) -> Expression<'a>,
) -> ModalResult<Expression<'a>> {
    let first = item0.parse_next(i)?;
    let items: Vec<(&Token<'a>, O)> = repeat(0.., (one_of(filter), item))
        .fold(Vec::new, |mut v, t| {
            v.push(t);
            v
        })
        .parse_next(i)?;
    if items.is_empty() {
        return Ok(first);
    }
    Ok(items.into_iter().fold(
        first,
        |left: Expression<'a>, (op, right): (&Token<'a>, O)| {
            accumulator(left, op.to_owned(), right)
        },
    ))
}

fn left_associative_infix_simple<'t, 'a: 't>(
    i: &mut Input<'t, 'a>,
    item: impl Parser<Input<'t, 'a>, Expression<'a>, ErrMode<ContextError>> + Clone,
    filter: impl Fn(&Token<'a>) -> bool,
) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, item.clone(), item, filter, |left, op, right| {
        Expression::Binary(Box::new(left), Box::new(op), Box::new(right))
    })
}

pub(super) fn multiplicative<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, exponentiation, |t| {
        *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent
    })
}

pub(super) fn additive<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, multiplicative, |t| {
        *t == Operator::Plus || *t == Operator::Minus
    })
}

pub(super) fn matching<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(
        i,
        additive,
        pattern(false),
        |t| *t == Keyword::Is,
        |e, op, p| Expression::Is(Box::new(e), Box::new(op), Box::new(p)),
    )
}

pub(super) fn relational<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, matching, |t| {
        *t == Operator::Less
            || *t == Operator::LessEqual
            || *t == Operator::Greater
            || *t == Operator::GreaterEqual
            || *t == Keyword::In
    })
}

pub(super) fn equality<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, relational, |t| {
        *t == Operator::EqualEqual
            || *t == Operator::NotEqual
            || *t == Operator::TildeEqual
            || *t == Operator::NotTildeEqual
    })
}

pub(super) fn and<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, equality, |t| *t == Operator::LogicalAnd)
}

pub(super) fn or<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, and, |t| *t == Operator::LogicalOr)
}

pub(super) fn pipe<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix_simple(i, or, |t| {
        *t == Operator::ForwardPipe || *t == Operator::BackwardPipe
    })
}

pub(super) fn basic_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    pipe.parse_next(i)
}

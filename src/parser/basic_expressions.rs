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
use super::helper::{literal_boxed, literal_or_insert, literal_token, variable_token};
use super::record_like_expression::record_like_element;
use super::{Expression, Input, RecordLikeElement, to_input};

fn tuple_like<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let open = literal_boxed(Operator::OpenParen).parse_next(i)?;
    let parts: Vec<_> = repeat(0.., record_like_element).parse_next(i)?;
    let close = literal_or_insert(Operator::CloseParen, "Missing ')'")
        .map(Box::new)
        .parse_next(i)?;
    let result = if parts.is_empty() {
        Expression::Record(Vec::new())
    } else if parts.len() == 1 {
        let part = parts.into_iter().next().unwrap();
        if let RecordLikeElement::Unnamed(exp, None) = part {
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

fn primary<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    (alt((
        block_like_expression,
        literal_token.map(Box::new).map(Expression::Literal),
        interpolation,
        variable_token(false, true)
            .map(Box::new)
            .map(Expression::Variable),
        tuple_like,
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
        token if *token == Operator::Plus || *token == Operator::Minus|| *token == Operator::LogicalNot || *token == Keyword::TypeOf =>
            seq!(Expression::Unary(any.map(|t: &Token<'a>| Box::new(t.to_owned())), unary.map(Box::new))),
        &Token{..} => function,
    }
    .parse_next(i)
}

fn exponent<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let first = unary.parse_next(i)?;
    let exponents: Vec<_> = repeat(0.., (literal_boxed(Operator::Caret), unary))
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

fn left_associative_infix<'a, I, F>(
    i: &mut Input<'_, 'a>,
    mut item: I,
    filter: F,
) -> ModalResult<Expression<'a>>
where
    I: for<'f> Parser<Input<'f, 'a>, Expression<'a>, ErrMode<ContextError>>,
    F: Fn(&Token<'a>) -> bool,
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
    Ok(items.into_iter().fold(
        first,
        |left: Expression<'a>, (op, right): (&Token<'a>, Expression<'a>)| {
            Expression::Binary(Box::new(left), Box::new(op.to_owned()), Box::new(right))
        },
    ))
}

fn factor<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, exponent, |t| {
        *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent
    })
}

pub(super) fn term<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, factor, |t| *t == Operator::Plus || *t == Operator::Minus)
}

fn comparison<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, term, |t| {
        *t == Operator::Less
            || *t == Operator::LessEqual
            || *t == Operator::Greater
            || *t == Operator::GreaterEqual
    })
}

fn equality<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, comparison, |t| {
        *t == Operator::EqualEqual
            || *t == Operator::NotEqual
            || *t == Operator::TildeEqual
            || *t == Operator::NotTildeEqual
    })
}

fn and<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, equality, |t| *t == Operator::LogicalAnd)
}

fn or<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, and, |t| *t == Operator::LogicalOr)
}

fn pipe<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, or, |t| {
        *t == Operator::ForwardPipe || *t == Operator::BackwardPipe
    })
}

pub(super) fn basic_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    pipe.parse_next(i)
}

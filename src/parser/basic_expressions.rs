use winnow::combinator::{
    alt, dispatch, eof, opt, peek, repeat, separated_foldl1, separated_foldr1, separated_pair, seq,
    terminated,
};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenKind};
use crate::utils::SourceRange;

use super::array_helper::array_base;
use super::block_expressions::block_like_expression;
use super::expressions::expression;
use super::helper::{literal_token, token, token_boxed, token_or_insert, variable_token};
use super::patterns::pattern;
use super::ranges::range;
use super::record_helper::record_base;
use super::{Expression, Input, RecordElement, to_input};

fn record_like<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let omit_named = |i: &mut Input<'_, 'a>| -> ModalResult<Expression<'a>> {
        expression
            .with_taken()
            .map(|(e, t)| {
                if matches!(e, Expression::Access(..)) || matches!(e, Expression::Variable(..)) {
                    e
                } else {
                    Expression::unknown(t, "Can not infer key from expression")
                }
            })
            .parse_next(i)
    };
    let (open, parts, close) =
        record_base(expression, omit_named, expression, expression).parse_next(i)?;
    let result = if parts.len() == 1 {
        let part = parts.into_iter().next().unwrap();
        if let RecordElement::Unnamed(exp, None) = part {
            Expression::Grouping(open, exp, close)
        } else {
            Expression::Record(open, vec![part], close)
        }
    } else {
        Expression::Record(open, parts, close)
    };
    Ok(result)
}

fn array<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let spread = |i: &mut Input<'_, 'a>| {
        let pos = i.previous_token_end();
        opt(expression)
            .map(|e| {
                if let Some(e) = e {
                    e
                } else {
                    Expression::unknown_range(
                        [],
                        SourceRange {
                            start: pos,
                            end: pos,
                        },
                        "Expression expected after `..`",
                    )
                }
            })
            .parse_next(i)
    };
    array_base(expression, range, spread)
        .map(|(open, parts, close)| Expression::Array(open, parts, close))
        .parse_next(i)
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

fn pseudo_function<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let (kw_type, open, (mut args, close)) = (
        token_boxed(Keyword::Type),
        token_or_insert(
            Operator::OpenParen,
            "`type` is a function-like keyword, add `(` here",
        ),
        arg_list,
    )
        .parse_next(i)?;
    let exp = if args.len() != 1 {
        Expression::unknown_range(
            [],
            SourceRange { start: 0, end: 0 },
            "`type` call must have exactly one argument",
        )
    } else {
        args.pop().unwrap()
    };
    Ok(Expression::Type(kw_type, open.into(), exp.into(), close))
}

pub(super) fn primary<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    (alt((
        pseudo_function,
        block_like_expression,
        literal_token.map(Box::new).map(Expression::Literal),
        interpolation,
        variable_token(false, true)
            .map(Box::new)
            .map(Expression::Variable),
        record_like,
        array,
    )))
    .parse_next(i)
}

fn arg_list<'a>(i: &mut Input<'_, 'a>) -> ModalResult<(Vec<Expression<'a>>, Box<Token<'a>>)> {
    separated_pair(
        (
            repeat(0.., terminated(expression, token(Operator::Comma))),
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
        peek(one_of(|t: &Token<'a>| {
            *t == TokenKind::Eof
                || *t == Operator::CloseParen
                || *t == Operator::Semicolon
                || *t == Operator::CloseBrace
                || *t == Operator::CloseBracket
        })),
        token_or_insert(Operator::CloseParen, "Missing `)`").map(Box::new),
    )
    .parse_next(i)
}

fn access_token<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Box<Token<'a>>> {
    one_of(|t: &Token<'a>| matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_)))
        .map(|t: &Token<'a>| Box::new(t.to_owned()))
        .parse_next(i)
}

fn extension_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Box<Expression<'a>>> {
    alt((
        record_like.with_taken().map(|(r, t)| {
            if matches!(r, Expression::Record(..)) {
                r.wrap_as_unknown(t, "Grouping expected")
            } else {
                r
            }
        }),
        (
            variable_token(false, true),
            repeat(0.., (token_boxed(Operator::Dot), access_token)),
        )
            .map(|(first, rest): (_, Vec<_>)| {
                let mut acc = Expression::Variable(first.into());
                for (dot, token) in rest {
                    acc = Expression::Access(Box::new(acc), dot, token);
                }
                acc
            }),
    ))
    .map(Box::new)
    .parse_next(i)
}

pub(super) fn postfix<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    enum Function<'a> {
        Call(Box<Token<'a>>, Vec<Expression<'a>>, Box<Token<'a>>),
        Extension(
            Box<Token<'a>>,
            Box<Expression<'a>>,
            Box<Token<'a>>,
            Vec<Expression<'a>>,
            Box<Token<'a>>,
        ),
        Access(Box<Token<'a>>, Box<Token<'a>>),
        Index(Box<Token<'a>>, Expression<'a>, Box<Token<'a>>),
        NonNil(Box<Token<'a>>),
    }
    let first = primary.parse_next(i)?;
    let functions: Vec<Function<'a>> = repeat(
        0..,
        alt((
            token_boxed(Operator::Exclamation).map(Function::NonNil),
            (token_boxed(Operator::OpenParen), arg_list).map(|(o, (a, c))| Function::Call(o, a, c)),
            (token_boxed(Operator::Dot), access_token).map(|(d, i)| Function::Access(d, i)),
            (
                token_boxed(Operator::ColonColon),
                extension_expression,
                token_boxed(Operator::OpenParen),
                arg_list,
            )
                .map(|(kw, ex, o, (a, c))| Function::Extension(kw, ex, o, a, c)),
            (
                token_boxed(Operator::OpenBracket),
                expression,
                token_boxed(Operator::CloseBracket),
            )
                .map(|(o, e, c)| Function::Index(o, e, c)),
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
        Function::Call(o, args, c) => Expression::Call(Box::new(acc), o, args, c),
        Function::Extension(e, ex, o, arg, c) => {
            Expression::Extension(Box::new(acc), e, ex, o, arg, c)
        }
        Function::Access(dot, token) => Expression::Access(Box::new(acc), dot, token),
        Function::Index(l, index, r) => Expression::Index(Box::new(acc), l, Box::new(index), r),
        Function::NonNil(token) => Expression::NonNil(Box::new(acc), token),
    }))
}

pub(super) fn prefix<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus || *token == Operator::Minus|| *token == Operator::Exclamation =>
            seq!(Expression::Prefix(any.map(|t: &Token<'a>| Box::new(t.to_owned())), prefix.map(Box::new))),
        &Token{..} => postfix,
    }
    .parse_next(i)
}

pub(super) fn exponentiation<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    separated_foldr1(prefix, token_boxed(Operator::Caret), |l, op, r| {
        Expression::Infix(Box::new(l), op, Box::new(r))
    })
    .parse_next(i)
}

fn left_associative_infix<'t, 'a: 't>(
    i: &mut Input<'t, 'a>,
    item: impl Parser<Input<'t, 'a>, Expression<'a>, ErrMode<ContextError>>,
    filter: impl Fn(&Token<'a>) -> bool,
) -> ModalResult<Expression<'a>> {
    separated_foldl1(item, one_of(filter), |left, op, right| {
        Expression::Infix(Box::new(left), Box::new(op.to_owned()), Box::new(right))
    })
    .parse_next(i)
}

pub(super) fn multiplicative<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, exponentiation, |t| {
        *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent
    })
}

pub(super) fn additive<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, multiplicative, |t| {
        *t == Operator::Plus || *t == Operator::Minus
    })
}

pub(super) fn matching<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    let first = additive.parse_next(i)?;
    let items: Vec<(_, _)> = repeat(0.., (token_boxed(Keyword::Is), pattern(false)))
        .fold(Vec::new, |mut v, t| {
            v.push(t);
            v
        })
        .parse_next(i)?;
    if items.is_empty() {
        return Ok(first);
    }
    Ok(items.into_iter().fold(first, |e, (op, p)| {
        Expression::Is(Box::new(e), op, Box::new(p))
    }))
}

pub(super) fn relational<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, matching, |t| {
        *t == Operator::Less
            || *t == Operator::LessEqual
            || *t == Operator::Greater
            || *t == Operator::GreaterEqual
            || *t == Keyword::In
    })
}

pub(super) fn equality<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, relational, |t| {
        *t == Operator::EqualEqual
            || *t == Operator::NotEqual
            || *t == Operator::TildeEqual
            || *t == Operator::NotTildeEqual
    })
}

pub(super) fn and<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, equality, |t| *t == Operator::LogicalAnd)
}

pub(super) fn or<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, and, |t| *t == Operator::LogicalOr)
}

pub(super) fn null_coalescing<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    left_associative_infix(i, or, |t| *t == Operator::NullCoalescing)
}

pub(super) fn basic_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    null_coalescing.parse_next(i)
}

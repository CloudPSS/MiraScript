use winnow::combinator::{
    alt, dispatch, eof, opt, peek, repeat, separated_foldl1, separated_foldr1, separated_pair, seq,
    terminated,
};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, one_of};

use crate::diagnostic::{DiagnosticCode, SourceRange};
use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::array_helper::array_base;
use super::block_expressions::block_like_expression;
use super::expression::Callable;
use super::expressions::expression;
use super::helper::{literal_token, token, token_boxed, token_or_insert, variable_token};
use super::patterns::pattern;
use super::ranges::range;
use super::record_helper::record_base;
use super::scripts::script;
use super::{Expression, Input, RecordElement, to_input};

fn to_interpolate_expr(token: Token<'_>) -> Expression<'_> {
    let TokenKind::InterpolatedString(_, e) = &token.kind else {
        unreachable!("Expected InterpolatedString");
    };
    let expressions: Vec<_> = e
        .iter()
        .map(|tokens| {
            let expr: ModalResult<Expression<'_>> = {
                let len = tokens.len();
                if tokens.len() >= 2
                    && tokens[0].kind == Operator::OpenBrace
                    && tokens[len - 1].kind == Operator::CloseBrace
                {
                    let [op, tokens @ .., cp] = &tokens[..] else {
                        unreachable!();
                    };
                    let mut token_input = to_input(tokens);
                    script.parse_next(&mut token_input).map(|script| {
                        Expression::Block(
                            op.to_owned().into(),
                            script.0,
                            script.1,
                            cp.to_owned().into(),
                        )
                    })
                } else {
                    let mut token_input = to_input(tokens);
                    terminated(expression, eof).parse_next(&mut token_input)
                }
            };
            let expr = match expr {
                Ok(expr) => expr,
                Err(_) => {
                    let last_token = tokens.last().unwrap();
                    let error = if *last_token == TokenKind::Eof {
                        DiagnosticCode::UnterminatedInterpolation
                    } else {
                        DiagnosticCode::BadInterpolation
                    };
                    Expression::unknown(tokens.clone(), error)
                }
            };
            expr
        })
        .collect();
    Expression::InterpolatedString(Box::new(token), expressions)
}

fn record_like<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    let (open, parts, close) = record_base(
        expression,
        |t| to_interpolate_expr(t.to_owned()),
        expression,
        expression,
        expression,
    )
    .parse_next(i)?;
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

fn array<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    let spread = |i: &mut Input<'_, 's>| {
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
                        DiagnosticCode::BadArraySpread,
                    )
                }
            })
            .parse_next(i)
    };
    array_base(expression, range, spread)
        .map(|(open, parts, close)| Expression::Array(open, parts, close))
        .parse_next(i)
}

pub(super) fn interpolation<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    let token = one_of(|t: &Token<'s>| matches!(&t.kind, &TokenKind::InterpolatedString(_, _)))
        .map(|t: &Token<'s>| t.to_owned())
        .parse_next(i)?;
    Ok(to_interpolate_expr(token))
}

/// callable '(' ...args ')'
type Call<'s> = (
    Box<Callable<'s>>,
    Box<Token<'s>>,
    Vec<Expression<'s>>,
    Box<Token<'s>>,
);

fn pseudo_function<'t, 's: 't>(
    extension_call: bool,
) -> impl Parser<Input<'t, 's>, Call<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'_, 's>| {
        let provided = if extension_call { 1 } else { 0 };
        let (kw_type, open, (args, close)) = (
            token(Keyword::Type),
            token_or_insert(
                Operator::OpenParen,
                DiagnosticCode::MissingOpenParenAfterType,
            ),
            arg_list,
        )
            .parse_next(i)?;
        let exp = if args.len() != (1 - provided) {
            vec![Expression::unknown_range(
                [],
                SourceRange {
                    start: kw_type.range.start,
                    end: close.range.end,
                },
                DiagnosticCode::InvalidTypeCall,
            )]
        } else {
            args
        };
        Ok((Callable::Type(kw_type).into(), open.into(), exp, close))
    }
}

pub(super) fn primary<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    (alt((
        pseudo_function(false).map(|(e, o, a, c)| Expression::Call(e, o, a, c)),
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

fn arg_list<'s>(i: &mut Input<'_, 's>) -> ModalResult<(Vec<Expression<'s>>, Box<Token<'s>>)> {
    separated_pair(
        (
            repeat(0.., terminated(expression, token(Operator::Comma))),
            opt(expression),
        )
            .map(
                |(mut v, e): (Vec<Expression<'s>>, Option<Expression<'s>>)| {
                    if let Some(e) = e {
                        v.push(e);
                    }
                    v
                },
            ),
        peek(one_of(|t: &Token<'s>| {
            *t == TokenKind::Eof
                || *t == Operator::CloseParen
                || *t == Operator::Semicolon
                || *t == Operator::CloseBrace
                || *t == Operator::CloseBracket
        })),
        token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen).map(Box::new),
    )
    .parse_next(i)
}

enum AccessIndex<'s> {
    /// '.' identifier
    Access(Box<Token<'s>>, Box<Token<'s>>),
    /// '[' expression ']'
    Index(Box<Token<'s>>, Box<Expression<'s>>, Box<Token<'s>>),
}
fn access_index<'s>(i: &mut Input<'_, 's>) -> ModalResult<AccessIndex<'s>> {
    let access_token = |i: &mut Input<'_, 's>| {
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_)))
            .map(|t: &Token<'s>| Box::new(t.to_owned()))
            .parse_next(i)
    };
    alt((
        (token_boxed(Operator::Dot), access_token).map(|(d, i)| AccessIndex::Access(d, i)),
        (
            token_boxed(Operator::OpenBracket),
            expression,
            token_boxed(Operator::CloseBracket),
        )
            .map(|(o, e, c)| AccessIndex::Index(o, Box::new(e), c)),
    ))
    .parse_next(i)
}

fn extension_call<'s>(i: &mut Input<'_, 's>) -> ModalResult<Call<'s>> {
    let parenthesised = |i: &mut Input<'_, 's>| {
        record_like
            .with_taken()
            .map(|(r, t)| {
                if r.is_record() {
                    r.wrap_as_unknown(t, DiagnosticCode::RecordLiteralInExtensionCaller)
                } else {
                    r
                }
            })
            .parse_next(i)
    };
    let access_chain = |i: &mut Input<'_, 's>| {
        (variable_token(false, true), repeat(0.., access_index))
            .map(|(first, rest): (_, Vec<_>)| {
                let mut acc = Expression::Variable(first.into());
                for access_index in rest {
                    match access_index {
                        AccessIndex::Access(dot, token) => {
                            acc = Expression::Access(Box::new(acc), dot, token);
                        }
                        AccessIndex::Index(open, exp, close) => {
                            acc = Expression::Index(Box::new(acc), open, exp, close);
                        }
                    }
                }
                acc
            })
            .parse_next(i)
    };
    alt((
        (
            alt((parenthesised, access_chain)).map(Callable::Expression),
            token_boxed(Operator::OpenParen),
            arg_list,
        )
            .map(|(e, o, (a, c))| (Box::new(e), o, a, c)),
        pseudo_function(true),
    ))
    .parse_next(i)
}

pub(super) fn postfix<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    enum Function<'s> {
        Call(Box<Token<'s>>, Vec<Expression<'s>>, Box<Token<'s>>),
        Extension(
            Box<Token<'s>>,
            Box<Callable<'s>>,
            Box<Token<'s>>,
            Vec<Expression<'s>>,
            Box<Token<'s>>,
        ),
        Access(Box<Token<'s>>, Box<Token<'s>>),
        Index(Box<Token<'s>>, Box<Expression<'s>>, Box<Token<'s>>),
        NonNil(Box<Token<'s>>),
    }
    let first = primary.parse_next(i)?;
    let functions: Vec<Function<'s>> = repeat(
        0..,
        alt((
            token_boxed(Operator::Exclamation).map(Function::NonNil),
            (token_boxed(Operator::OpenParen), arg_list).map(|(o, (a, c))| Function::Call(o, a, c)),
            (token_boxed(Operator::ColonColon), extension_call)
                .map(|(kw, (ex, o, a, c))| Function::Extension(kw, ex, o, a, c)),
            access_index.map(|t| match t {
                AccessIndex::Access(dot, token) => Function::Access(dot, token),
                AccessIndex::Index(open, exp, close) => Function::Index(open, exp, close),
            }),
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
        Function::Call(o, args, c) => {
            Expression::Call(Box::new(Callable::Expression(acc)), o, args, c)
        }
        Function::Extension(e, ex, o, arg, c) => {
            Expression::Extension(Box::new(acc), e, ex, o, arg, c)
        }
        Function::Access(dot, token) => Expression::Access(Box::new(acc), dot, token),
        Function::Index(l, index, r) => Expression::Index(Box::new(acc), l, index, r),
        Function::NonNil(token) => Expression::NonNil(Box::new(acc), token),
    }))
}

pub(super) fn prefix<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus || *token == Operator::Minus|| *token == Operator::Exclamation =>
            seq!(Expression::Prefix(any.map(|t: &Token<'s>| Box::new(t.to_owned())), prefix.map(Box::new))),
        &Token{..} => postfix,
    }
    .parse_next(i)
}

pub(super) fn exponentiation<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    separated_foldr1(prefix, token_boxed(Operator::Caret), |l, op, r| {
        Expression::Infix(Box::new(l), op, Box::new(r))
    })
    .parse_next(i)
}

fn left_associative_infix<'t, 's: 't>(
    i: &mut Input<'t, 's>,
    item: impl Parser<Input<'t, 's>, Expression<'s>, ErrMode<ContextError>>,
    filter: impl Fn(&Token<'s>) -> bool,
) -> ModalResult<Expression<'s>> {
    separated_foldl1(item, one_of(filter), |left, op, right| {
        Expression::Infix(Box::new(left), Box::new(op.to_owned()), Box::new(right))
    })
    .parse_next(i)
}

pub(super) fn multiplicative<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, exponentiation, |t| {
        *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent
    })
}

pub(super) fn additive<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, multiplicative, |t| {
        *t == Operator::Plus || *t == Operator::Minus
    })
}

pub(super) fn matching<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
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

pub(super) fn relational<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, matching, |t| {
        *t == Operator::Less
            || *t == Operator::LessEqual
            || *t == Operator::Greater
            || *t == Operator::GreaterEqual
            || *t == Keyword::In
    })
}

pub(super) fn equality<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, relational, |t| {
        *t == Operator::EqualEqual
            || *t == Operator::NotEqual
            || *t == Operator::TildeEqual
            || *t == Operator::NotTildeEqual
    })
}

pub(super) fn and<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, equality, |t| *t == Operator::LogicalAnd)
}

pub(super) fn or<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, and, |t| *t == Operator::LogicalOr)
}

pub(super) fn null_coalescing<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    left_associative_infix(i, or, |t| *t == Operator::NullCoalescing)
}

pub(super) fn basic_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    null_coalescing.parse_next(i)
}

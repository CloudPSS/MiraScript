use winnow::{
    combinator::{
        alt, dispatch, eof, opt, peek, repeat, separated_foldl1, separated_foldr1, separated_pair,
        seq, terminated,
    },
    token::{any, one_of},
};

use super::array_helper::array_base;
use super::block_expressions::block_like_expression;
use super::expression::Callable;
use super::expressions::expression;
use super::helper::{literal_token, token, token_or_insert, variable_token};
use super::patterns::pattern;
use super::ranges::range;
use super::record_helper::record_base;
use super::scripts::script;
use super::{prelude::*, record_element::RecordElementBase, to_input};

fn to_interpolate_expr<'s>(token: &'s Token<'s>) -> Expression<'s> {
    let TokenKind::InterpolatedString(v) = &token.kind else {
        unreachable!("Expected InterpolatedString");
    };
    let expressions: Vec<Expression<'s>> = v[0..v.len() - 1]
        .iter()
        .map(|(_, tokens)| {
            let expr: Result<Expression<'s>> = {
                let tokens = match &tokens[..] {
                    // If the first token is an open brace and the last token is a close brace,
                    // we treat it as a block interpolation.
                    [op, inner @ .., ed]
                        if *op == Operator::OpenBrace && *ed == Operator::CloseBrace =>
                    {
                        inner
                    }
                    tokens => tokens,
                };
                let mut token_input = to_input(tokens);
                terminated(expression, eof).parse_next(&mut token_input)
            };
            match expr {
                Ok(expr) => expr,
                Err(_) => {
                    let last_token = tokens.last().unwrap();
                    let error = if *last_token == TokenKind::Eof {
                        DiagnosticCode::UnterminatedInterpolation
                    } else {
                        DiagnosticCode::BadInterpolation
                    };
                    Expression::unknown(
                        tokens.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                        error,
                    )
                }
            }
        })
        .collect();
    Expression::InterpolatedString(token, expressions)
}

fn record_like<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    let (open, parts, close) = record_base(
        expression,
        |t: &Token<'s>| to_interpolate_expr(t),
        expression,
        expression,
        expression,
    )
    .parse_next(i)?;
    let result = if parts.len() == 1 && !parts[0].has_tail_comma() && parts[0].is_unnamed() {
        let RecordElementBase::Unnamed(part) = parts.into_iter().next().unwrap().unwrap() else {
            unreachable!();
        };
        Expression::Grouping(open, part, close)
    } else {
        Expression::Record(open, parts, close)
    };
    Ok(result)
}

fn array<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    let spread = |i: &mut Input<'s>| {
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
    array_base(
        [Operator::OpenBracket, Operator::CloseBracket],
        expression,
        range,
        spread,
    )
    .map(|(open, parts, close)| Expression::Array(open, parts, close))
    .parse_next(i)
}

fn interpolation<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    let token = one_of(|t: &Token<'s>| matches!(&t.kind, &TokenKind::InterpolatedString(_)))
        .parse_next(i)?;
    Ok(to_interpolate_expr(token))
}

/// callable '(' ...args ')'
type Call<'s> = (
    Callable<'s>,
    TokenRef<'s>,
    Vec<Expression<'s>>,
    TokenRef<'s>,
);

fn pseudo_function<'t, 's: 't, const EXTENSION_CALL: bool>(i: &mut Input<'s>) -> Result<Call<'s>> {
    let provided: usize = if EXTENSION_CALL { 1 } else { 0 };
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
    Ok((Callable::Type(kw_type), open, exp, close))
}

fn primary<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    (alt((
        pseudo_function::<false>.map(|(e, o, a, c)| Expression::Call(e, o, a, c)),
        block_like_expression,
        literal_token.map(Expression::Literal),
        interpolation,
        variable_token(false, true).map(Expression::Variable),
        record_like,
        array,
    )))
    .parse_next(i)
}

fn arg_list<'s>(i: &mut Input<'s>) -> Result<(Vec<Expression<'s>>, TokenRef<'s>)> {
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
        token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen),
    )
    .parse_next(i)
}

enum AccessIndex<'s> {
    /// '.' identifier
    Access(TokenRef<'s>, TokenRef<'s>),
    /// '[' expression ']'
    Index(TokenRef<'s>, Box<Expression<'s>>, TokenRef<'s>),
}
fn access_index<'s>(i: &mut Input<'s>) -> Result<AccessIndex<'s>> {
    let access_token = |i: &mut Input<'s>| {
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_)))
            .map(TokenRef::borrow)
            .parse_next(i)
    };
    alt((
        (token(Operator::Dot), access_token).map(|(d, i)| AccessIndex::Access(d, i)),
        (
            token(Operator::OpenBracket),
            expression.map(Box::new),
            token(Operator::CloseBracket),
        )
            .map(|(o, e, c)| AccessIndex::Index(o, e, c)),
    ))
    .parse_next(i)
}

fn extension_call<'s>(i: &mut Input<'s>) -> Result<Call<'s>> {
    let parenthesised = |i: &mut Input<'s>| {
        record_like
            .with_taken()
            .map(|(r, t)| {
                if r.is_record() {
                    r.wrap_as_unknown(
                        t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                        DiagnosticCode::RecordLiteralInExtensionCaller,
                    )
                } else {
                    r
                }
            })
            .parse_next(i)
    };
    let access_chain = |i: &mut Input<'s>| {
        (variable_token(false, true), repeat(0.., access_index))
            .map(|(first, rest): (_, Vec<_>)| {
                let mut acc = Expression::Variable(first);
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
            alt((parenthesised, access_chain)).map(|e| Callable::Expression(Box::new(e))),
            token(Operator::OpenParen),
            arg_list,
        )
            .map(|(e, o, (a, c))| (e, o, a, c)),
        pseudo_function::<true>,
    ))
    .parse_next(i)
}

fn postfix<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    enum Function<'s> {
        Call(TokenRef<'s>, Vec<Expression<'s>>, TokenRef<'s>),
        Extension(
            TokenRef<'s>,
            Callable<'s>,
            TokenRef<'s>,
            Vec<Expression<'s>>,
            TokenRef<'s>,
        ),
        Access(TokenRef<'s>, TokenRef<'s>),
        Index(TokenRef<'s>, Box<Expression<'s>>, TokenRef<'s>),
        NonNil(TokenRef<'s>),
    }
    let first = primary.parse_next(i)?;
    let functions: Vec<Function<'s>> = repeat(
        0..,
        alt((
            token(Operator::Exclamation).map(Function::NonNil),
            (token(Operator::OpenParen), arg_list).map(|(o, (a, c))| Function::Call(o, a, c)),
            (token(Operator::ColonColon), extension_call)
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
            Expression::Call(Callable::Expression(Box::new(acc)), o, args, c)
        }
        Function::Extension(e, ex, o, arg, c) => {
            Expression::Extension(Box::new(acc), e, ex, o, arg, c)
        }
        Function::Access(dot, token) => Expression::Access(Box::new(acc), dot, token),
        Function::Index(l, index, r) => Expression::Index(Box::new(acc), l, index, r),
        Function::NonNil(token) => Expression::NonNil(Box::new(acc), token),
    }))
}

fn exponentiation<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    separated_foldr1(postfix, token(Operator::Caret), |l, op, r| {
        Expression::Infix(Box::new(l), op, Box::new(r))
    })
    .parse_next(i)
}

fn prefix<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    dispatch! {peek(any);
        token if *token == Operator::Plus || *token == Operator::Minus|| *token == Operator::Exclamation =>
            seq!(Expression::Prefix(any.map(TokenRef::borrow), prefix.map(Box::new))),
        &Token{..} => exponentiation,
    }
    .parse_next(i)
}

fn left_associative_infix<'s>(
    i: &mut Input<'s>,
    item: impl Parser<'s, Expression<'s>>,
    filter: impl Fn(&Token<'s>) -> bool,
) -> Result<Expression<'s>> {
    separated_foldl1(
        item,
        one_of(filter).map(TokenRef::borrow),
        |left, op, right| Expression::Infix(Box::new(left), op, Box::new(right)),
    )
    .parse_next(i)
}

fn multiplicative<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, prefix, |t| {
        *t == Operator::Asterisk || *t == Operator::Slash || *t == Operator::Percent
    })
}

pub(super) fn additive<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, multiplicative, |t| {
        *t == Operator::Plus || *t == Operator::Minus
    })
}

fn matching<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    let first = additive.parse_next(i)?;
    let items: Vec<(_, _)> = repeat(0.., (token(Keyword::Is), pattern(false)))
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

fn relational<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, matching, |t| {
        *t == Operator::Less
            || *t == Operator::LessEqual
            || *t == Operator::Greater
            || *t == Operator::GreaterEqual
            || *t == Keyword::In
    })
}

fn equality<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, relational, |t| {
        *t == Operator::EqualEqual
            || *t == Operator::NotEqual
            || *t == Operator::TildeEqual
            || *t == Operator::NotTildeEqual
    })
}

fn and<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, equality, |t| *t == Operator::LogicalAnd)
}

fn or<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, and, |t| *t == Operator::LogicalOr)
}

fn null_coalescing<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    left_associative_infix(i, or, |t| *t == Operator::NullCoalescing)
}

pub(super) fn basic_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    null_coalescing.parse_next(i)
}

use winnow::{
    combinator::{alt, eof, opt, peek, repeat},
    token::{any, one_of},
};

use super::{
    array_helper::array_base,
    block_expressions::block_like_expression,
    expressions::{expression, expression_expected},
    helper::{literal_token, token, token_or_insert, variable_token},
    patterns::pattern,
    prelude::*,
    record_helper::record_base,
    to_input,
};

fn to_interpolate_expr<'s>(token: &'s Token<'s>) -> Expression<'s> {
    let TokenKind::InterpolatedString(parts, _) = &token.kind else {
        unreachable!("Expected InterpolatedString");
    };
    let expressions: Vec<Expression<'s>> = parts[0..parts.len() - 1]
        .iter()
        .map(|(_, tokens, _)| {
            debug_assert!(
                !tokens.is_empty(),
                "Expected non-empty tokens in interpolation {token:?}"
            );
            if tokens.iter().all(|t| t.is_empty()) {
                return Expression::unknown(
                    tokens.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                    DiagnosticCode::EmptyInterpolation,
                );
            }
            let last_token = tokens.last().map_or(&TokenKind::Eof, |t| &t.kind);
            if *last_token == TokenKind::Eof {
                return Expression::unknown(
                    tokens.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                    DiagnosticCode::UnterminatedInterpolation,
                );
            }
            let mut token_input = to_input(tokens);
            let result = (expression, opt(eof.value(()))).parse_next(&mut token_input);
            match result {
                Ok((expr, Some(_))) => expr,
                Ok((expr, None)) => expr.wrap_as_unknown(
                    token_input
                        .peek_finish()
                        .iter()
                        .map(TokenRef::borrow)
                        .collect::<Vec<_>>(),
                    DiagnosticCode::BadInterpolation,
                ),
                Err(_) => Expression::unknown(
                    tokens.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
                    DiagnosticCode::BadInterpolation,
                ),
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
        expression_expected,
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
        token(Operator::OpenBracket),
        token_or_insert(Operator::CloseBracket, DiagnosticCode::MissingCloseBracket),
        iterable,
        spread,
        |pos| Iterable::Value(expression_expected(pos)),
    )
    .map(|(open, parts, close)| Expression::Array(open, parts, close))
    .parse_next(i)
}

pub(super) fn interpolation<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    let token = one_of(|t: &Token<'s>| matches!(&t.kind, &TokenKind::InterpolatedString(..)))
        .parse_next(i)?;
    Ok(to_interpolate_expr(token))
}

/// callable '(' ('..'? arg),* ')'
type Call<'s> = (
    Callable<'s>,
    TokenRef<'s>,
    Vec<ArgElement<'s>>,
    TokenRef<'s>,
);

fn pseudo_function<'t, 's: 't, const EXTENSION_CALL: bool>(i: &mut Input<'s>) -> Result<Call<'s>> {
    let provided: usize = if EXTENSION_CALL { 1 } else { 0 };
    let (kw_type, (open, args, close)) = (
        token(Keyword::Type),
        arg_list(token_or_insert(
            Operator::OpenParen,
            DiagnosticCode::MissingOpenParenAfterType,
        )),
    )
        .parse_next(i)?;
    let exp = if args.len() != (1 - provided) || args.first().is_some_and(|a| a.is_spread()) {
        vec![ListItem::new(ArrayElementBase::Element(
            Expression::unknown_range(
                [],
                SourceRange {
                    start: kw_type.range.start,
                    end: close.range.end,
                },
                DiagnosticCode::InvalidTypeCall,
            )
            .into(),
        ))]
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

fn arg_list<'s>(
    open: impl Parser<'s, TokenRef<'s>>,
) -> impl Parser<'s, (TokenRef<'s>, Vec<ArgElement<'s>>, TokenRef<'s>)> {
    move |i: &mut Input<'s>| {
        array_base(
            open,
            token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen),
            expression,
            expression,
            expression_expected,
        )
        .parse_next(i)
    }
}

enum AccessIndex<'s> {
    /// `.` identifier
    Access(TokenRef<'s>, TokenRef<'s>),
    /// `[` expression `]`
    Index(TokenRef<'s>, Box<Expression<'s>>, TokenRef<'s>),
    ///  `[` additive_expression? (`..` | `..<`) additive_expression? `]`
    Slice(
        TokenRef<'s>,
        Option<Box<Expression<'s>>>,
        TokenRef<'s>,
        Option<Box<Expression<'s>>>,
        TokenRef<'s>,
    ),
    /// `!`
    NonNil(TokenRef<'s>),
}
fn access_index<'s>(i: &mut Input<'s>) -> Result<AccessIndex<'s>> {
    fn access_token<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_)))
            .map(TokenRef::borrow)
            .parse_next(i)
    }
    fn additive<'s>(i: &mut Input<'s>) -> Result<Box<Expression<'s>>> {
        let mut precedence_additive = precedence_of(&TokenKind::Operator(Operator::SpreadRange));
        precedence_additive.value += 2;
        pratt(precedence_additive, false)
            .verify_map(verify_expr)
            .map(Box::new)
            .parse_next(i)
    }
    fn range_op<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
        one_of(|t: &Token<'s>| *t == Operator::SpreadRange || *t == Operator::HalfOpenRange)
            .map(TokenRef::borrow)
            .parse_next(i)
    }
    fn non_nil<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
        token(Operator::Exclamation).parse_next(i)
    }

    alt((
        // `!`
        non_nil.map(AccessIndex::NonNil),
        // `.` identifier
        (token(Operator::Dot), access_token).map(|(d, i)| AccessIndex::Access(d, i)),
        // `[` (`..` | `..<`) `]` | `[` (`..` | `..<`) additive `]`
        (
            token(Operator::OpenBracket),
            range_op,
            opt(additive),
            token(Operator::CloseBracket),
        )
            .map(|(l, op, end, r)| AccessIndex::Slice(l, None, op, end, r)),
        // `[` expression `]` | `[` additive (`..` | `..<`) additive `]`
        (
            token(Operator::OpenBracket),
            iterable,
            token(Operator::CloseBracket),
        )
            .map(|(o, e, c)| match e {
                Iterable::Range(r) => AccessIndex::Slice(o, Some(r.0), r.1, Some(r.2), c),
                Iterable::Value(expr) => AccessIndex::Index(o, Box::new(expr), c),
            }),
        // `[` additive (`..` | `..<`) `]`
        (
            token(Operator::OpenBracket),
            additive,
            range_op,
            token(Operator::CloseBracket),
        )
            .map(|(l, start, op, r)| AccessIndex::Slice(l, Some(start), op, None, r)),
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
                        AccessIndex::NonNil(token) => {
                            acc = Expression::NonNil(Box::new(acc), token);
                        }
                        AccessIndex::Access(dot, token) => {
                            acc = Expression::Access(Box::new(acc), dot, token);
                        }
                        AccessIndex::Index(open, exp, close) => {
                            acc = Expression::Index(Box::new(acc), open, exp, close);
                        }
                        AccessIndex::Slice(left, start, op, end, right) => {
                            acc = Expression::Slice(Box::new(acc), left, start, op, end, right);
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
            arg_list(token_or_insert(
                Operator::OpenParen,
                DiagnosticCode::MissingOpenParenAfterExtension,
            )),
        )
            .map(|(e, (o, a, c))| (e, o, a, c)),
        pseudo_function::<true>,
    ))
    .parse_next(i)
}

fn postfix<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    enum Function<'s> {
        Call(TokenRef<'s>, Vec<ArgElement<'s>>, TokenRef<'s>),
        TaggedString(Box<Expression<'s>>),
        Extension(
            TokenRef<'s>,
            Callable<'s>,
            TokenRef<'s>,
            Vec<ArgElement<'s>>,
            TokenRef<'s>,
        ),
        Access(TokenRef<'s>, TokenRef<'s>),
        Index(TokenRef<'s>, Box<Expression<'s>>, TokenRef<'s>),
        Slice(
            TokenRef<'s>,
            Option<Box<Expression<'s>>>,
            TokenRef<'s>,
            Option<Box<Expression<'s>>>,
            TokenRef<'s>,
        ),
        NonNil(TokenRef<'s>),
    }
    let first = primary.parse_next(i)?;
    let functions: Vec<Function<'s>> = repeat(
        0..,
        alt((
            (token(Operator::ColonColon), extension_call)
                .map(|(kw, (ex, o, a, c))| Function::Extension(kw, ex, o, a, c)),
            one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::String(..))).map(|token| {
                Function::TaggedString(Box::new(Expression::Literal(TokenRef::borrow(token))))
            }),
            interpolation.map(|ex| Function::TaggedString(ex.into())),
            access_index.map(|t| match t {
                AccessIndex::NonNil(token) => Function::NonNil(token),
                AccessIndex::Access(dot, token) => Function::Access(dot, token),
                AccessIndex::Index(open, exp, close) => Function::Index(open, exp, close),
                AccessIndex::Slice(left, start, op, end, right) => {
                    Function::Slice(left, start, op, end, right)
                }
            }),
            arg_list(token(Operator::OpenParen)).map(|(o, a, c)| Function::Call(o, a, c)),
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
        Function::TaggedString(ex) => Expression::TaggedString(Box::new(acc), ex),
        Function::Extension(e, ex, o, arg, c) => {
            Expression::Extension(Box::new(acc), e, ex, o, arg, c)
        }
        Function::Access(dot, token) => Expression::Access(Box::new(acc), dot, token),
        Function::Index(l, index, r) => Expression::Index(Box::new(acc), l, index, r),
        Function::Slice(left, start, op, end, right) => {
            Expression::Slice(Box::new(acc), left, start, op, end, right)
        }
        Function::NonNil(token) => Expression::NonNil(Box::new(acc), token),
    }))
}

#[derive(Default)]
struct PrecedenceResult {
    value: u8,
    can_be_prefix: bool,
    right_associative: bool,
}

fn precedence_of(t: &TokenKind<'_>) -> PrecedenceResult {
    use crate::lexer::{Keyword::*, Operator::*, TokenKind::*};
    let (precedence, can_be_prefix) = match t {
        Operator(Caret) => (200, false),
        Operator(Exclamation) | Keyword(Not) => (190, true),
        Operator(Asterisk) | Operator(Slash) | Operator(Percent) => (180, false),
        Operator(Plus) | Operator(Minus) => (170, true),
        Operator(SpreadRange) | Operator(HalfOpenRange) => (140, false),
        Keyword(Is) => (120, false),
        Keyword(In)
        | Keyword(NotIn)
        | Operator(Less)
        | Operator(LessEqual)
        | Operator(Greater)
        | Operator(GreaterEqual) => (100, false),
        Operator(Equal) | Operator(NotEqual) | Operator(TildeEqual) | Operator(TildeNotEqual) => {
            (90, false)
        }
        Operator(LogicalAnd) | Keyword(And) => (70, false),
        Operator(LogicalOr) | Keyword(Or) => (60, false),
        Operator(NullCoalescing) => (50, false),
        Operator(Question) => (30, true),
        _ => return PrecedenceResult::default(),
    };
    PrecedenceResult {
        value: precedence,
        can_be_prefix,
        right_associative: matches!(t, Operator(Caret) | Operator(Question)),
    }
}

fn pratt_prefix<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    let token = peek(any).parse_next(i)?;
    let precedence = precedence_of(token);
    if precedence.can_be_prefix {
        let op = any.parse_next(i)?;
        let expr = pratt(precedence, false)
            .verify_map(verify_expr)
            .parse_next(i)?;
        Ok(Expression::Prefix(op.into(), expr.into()))
    } else {
        postfix.parse_next(i)
    }
}

fn pratt_infix<'s>(
    left: Box<Expression<'s>>,
    op: &'s Token<'s>,
    mut precedence: PrecedenceResult,
    allow_range: bool,
    i: &mut Input<'s>,
) -> Result<Iterable<'s>> {
    if *op == Keyword::Is {
        let right = pattern(false).map(Box::new).parse_next(i)?;
        return Ok(Iterable::Value(Expression::Is(left, op.into(), right)));
    }
    // 调整优先级以实现右结合
    if precedence.right_associative {
        precedence.value -= 1;
    }
    let parse_right = |i: &mut Input<'s>| {
        let expr = pratt(precedence, false)
            .verify_map(verify_expr)
            .parse_next(i)?;
        Ok(Box::new(expr))
    };
    if *op == Operator::Question {
        let then_exp = expression.parse_next(i)?.into();
        let colon = token_or_insert(Operator::Colon, DiagnosticCode::MissingColon).parse_next(i)?;
        let else_exp = parse_right(i)?;
        return Ok(Iterable::Value(Expression::Cond(
            left,
            op.into(),
            then_exp,
            colon,
            else_exp,
        )));
    }
    let right = parse_right(i)?;
    if *op == Operator::SpreadRange || *op == Operator::HalfOpenRange {
        return if allow_range {
            Ok(Iterable::Range(Range(left, op.into(), right)))
        } else {
            Ok(Iterable::Value(Expression::Infix(
                left,
                Token::unknown(
                    op.range.clone(),
                    TokenKind::Operator(Operator::Plus),
                    DiagnosticCode::UnexpectedToken,
                )
                .into(),
                right,
            )))
        };
    }
    Ok(Iterable::Value(Expression::Infix(left, op.into(), right)))
}

fn pratt<'s>(precedence: PrecedenceResult, allow_range: bool) -> impl Parser<'s, Iterable<'s>> {
    move |i: &mut Input<'s>| {
        let mut left = pratt_prefix.parse_next(i)?;

        loop {
            if i.is_empty() {
                break;
            }

            let op = peek(any).parse_next(i)?;
            let op_precedence = precedence_of(op);
            if op_precedence.value <= precedence.value {
                break;
            }

            let op = any.parse_next(i)?;
            match pratt_infix(left.into(), op, op_precedence, allow_range, i)? {
                Iterable::Value(e) => left = e,
                Iterable::Range(r) => return Ok(Iterable::Range(r)),
            }
        }

        Ok(Iterable::Value(left))
    }
}

fn verify_expr<'s>(e: Iterable<'s>) -> Option<Expression<'s>> {
    match e {
        Iterable::Value(e) => Some(e),
        _ => None,
    }
}

pub(super) fn basic_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    pratt(PrecedenceResult::default(), false)
        .verify_map(verify_expr)
        .parse_next(i)
}

pub(super) fn iterable<'s>(i: &mut Input<'s>) -> Result<Iterable<'s>> {
    pratt(PrecedenceResult::default(), true).parse_next(i)
}

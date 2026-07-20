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

fn to_interpolate_expr<'s: 'a, 'a>(
    arena: &'a AstArena,
    token: &'s Token<'s>,
) -> Expression<'s, 'a> {
    let TokenKind::InterpolatedString(parts, _) = &token.kind else {
        unreachable!("Expected InterpolatedString");
    };
    let expressions: Vec<Expression<'s, 'a>> = parts[0..parts.len() - 1]
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
            let result = (|i: &mut Input<'s>| expression(arena, i), opt(eof.value(())))
                .parse_next(&mut token_input);
            match result {
                Ok((expr, Some(_))) => expr,
                Ok((expr, None)) => expr.wrap_as_unknown(
                    arena,
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

fn record_like<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Expression<'s, 'a>> {
    let (open, parts, close) = record_base(
        arena,
        |i: &mut Input<'s>| expression(arena, i),
        |t: &Token<'s>| to_interpolate_expr(arena, t),
        |i: &mut Input<'s>| expression(arena, i),
        |i: &mut Input<'s>| expression(arena, i),
        |i: &mut Input<'s>| expression(arena, i),
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

fn array<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Expression<'s, 'a>> {
    let spread = |i: &mut Input<'s>| {
        let pos = i.previous_token_end();
        opt(|i: &mut Input<'s>| expression(arena, i))
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
        arena,
        token(Operator::OpenBracket),
        token_or_insert(Operator::CloseBracket, DiagnosticCode::MissingCloseBracket),
        |i: &mut Input<'s>| iterable(arena, i),
        spread,
        |pos| Iterable::Value(expression_expected(pos)),
    )
    .map(|(open, parts, close)| Expression::Array(open, parts, close))
    .parse_next(i)
}

pub(super) fn interpolation<'s: 'a, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    let token = one_of(|t: &Token<'s>| matches!(&t.kind, &TokenKind::InterpolatedString(..)))
        .parse_next(i)?;
    Ok(to_interpolate_expr(arena, token))
}

/// callable '(' ('..'? arg),* ')'
type Call<'s, 'a> = (
    Callable<'s, 'a>,
    TokenRef<'s>,
    Vec<ArgElement<'s, 'a>>,
    TokenRef<'s>,
);

fn pseudo_function<'t, 's: 't + 'a, 'a, const EXTENSION_CALL: bool>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Call<'s, 'a>> {
    let provided: usize = if EXTENSION_CALL { 1 } else { 0 };
    let (kw_type, (open, args, close)) = (
        token(Keyword::Type),
        arg_list(
            arena,
            token_or_insert(
                Operator::OpenParen,
                DiagnosticCode::MissingOpenParenAfterType,
            ),
        ),
    )
        .parse_next(i)?;
    let exp = if args.len() != (1 - provided) || args.first().is_some_and(|a| a.is_spread()) {
        vec![ListItem::new(
            arena,
            ArrayElementBase::Element(arena.alloc(Expression::unknown_range(
                [],
                SourceRange {
                    start: kw_type.range.start,
                    end: close.range.end,
                },
                DiagnosticCode::InvalidTypeCall,
            ))),
        )]
    } else {
        args
    };
    Ok((Callable::Type(kw_type), open, exp, close))
}

fn primary<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Expression<'s, 'a>> {
    (alt((
        (|i: &mut Input<'s>| pseudo_function::<false>(arena, i))
            .map(|(e, o, a, c)| Expression::Call(e, o, a, c)),
        |i: &mut Input<'s>| block_like_expression(arena, i),
        literal_token.map(Expression::Literal),
        |i: &mut Input<'s>| interpolation(arena, i),
        variable_token(false, true).map(Expression::Variable),
        |i: &mut Input<'s>| record_like(arena, i),
        |i: &mut Input<'s>| array(arena, i),
    )))
    .parse_next(i)
}

fn arg_list<'s: 'a, 'a>(
    arena: &'a AstArena,
    open: impl Parser<'s, TokenRef<'s>>,
) -> impl Parser<'s, (TokenRef<'s>, Vec<ArgElement<'s, 'a>>, TokenRef<'s>)> {
    move |i: &mut Input<'s>| {
        array_base(
            arena,
            open,
            token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen),
            |i: &mut Input<'s>| expression(arena, i),
            |i: &mut Input<'s>| expression(arena, i),
            expression_expected,
        )
        .parse_next(i)
    }
}

enum AccessIndex<'s, 'a> {
    /// `.` identifier
    Access(TokenRef<'s>, TokenRef<'s>),
    /// `[` expression `]`
    Index(TokenRef<'s>, ABox<'a, Expression<'s, 'a>>, TokenRef<'s>),
    ///  `[` additive_expression? (`..` | `..<`) additive_expression? `]`
    Slice(
        TokenRef<'s>,
        Option<ABox<'a, Expression<'s, 'a>>>,
        TokenRef<'s>,
        Option<ABox<'a, Expression<'s, 'a>>>,
        TokenRef<'s>,
    ),
    /// `!`
    NonNil(TokenRef<'s>),
}
fn access_index<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<AccessIndex<'s, 'a>> {
    fn access_token<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
        one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_)))
            .map(TokenRef::borrow)
            .parse_next(i)
    }
    fn additive<'s: 'a, 'a>(
        arena: &'a AstArena,
        i: &mut Input<'s>,
    ) -> Result<ABox<'a, Expression<'s, 'a>>> {
        let mut precedence_additive = precedence_of(&TokenKind::Operator(Operator::SpreadRange));
        precedence_additive.value += 2;
        pratt(arena, precedence_additive, false)
            .verify_map(verify_expr)
            .map(|e| arena.alloc(e))
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
            opt(|i: &mut Input<'s>| additive(arena, i)),
            token(Operator::CloseBracket),
        )
            .map(|(l, op, end, r)| AccessIndex::Slice(l, None, op, end, r)),
        // `[` expression `]` | `[` additive (`..` | `..<`) additive `]`
        (
            token(Operator::OpenBracket),
            |i: &mut Input<'s>| iterable(arena, i),
            token(Operator::CloseBracket),
        )
            .map(|(o, e, c)| match e {
                Iterable::Range(r) => AccessIndex::Slice(o, Some(r.0), r.1, Some(r.2), c),
                Iterable::Value(expr) => AccessIndex::Index(o, arena.alloc(expr), c),
            }),
        // `[` additive (`..` | `..<`) `]`
        (
            token(Operator::OpenBracket),
            |i: &mut Input<'s>| additive(arena, i),
            range_op,
            token(Operator::CloseBracket),
        )
            .map(|(l, start, op, r)| AccessIndex::Slice(l, Some(start), op, None, r)),
    ))
    .parse_next(i)
}

fn extension_call<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Call<'s, 'a>> {
    let parenthesised = |i: &mut Input<'s>| {
        (|i: &mut Input<'s>| record_like(arena, i))
            .with_taken()
            .map(|(r, t)| {
                if r.is_record() {
                    r.wrap_as_unknown(
                        arena,
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
        (
            variable_token(false, true),
            repeat(0.., |i: &mut Input<'s>| access_index(arena, i)),
        )
            .map(|(first, rest): (_, Vec<_>)| {
                let mut acc = Expression::Variable(first);
                for access_index in rest {
                    match access_index {
                        AccessIndex::NonNil(token) => {
                            acc = Expression::NonNil(arena.alloc(acc), token);
                        }
                        AccessIndex::Access(dot, token) => {
                            acc = Expression::Access(arena.alloc(acc), dot, token);
                        }
                        AccessIndex::Index(open, exp, close) => {
                            acc = Expression::Index(arena.alloc(acc), open, exp, close);
                        }
                        AccessIndex::Slice(left, start, op, end, right) => {
                            acc = Expression::Slice(arena.alloc(acc), left, start, op, end, right);
                        }
                    }
                }
                acc
            })
            .parse_next(i)
    };
    alt((
        (
            alt((parenthesised, access_chain)).map(|e| Callable::Expression(arena.alloc(e))),
            arg_list(
                arena,
                token_or_insert(
                    Operator::OpenParen,
                    DiagnosticCode::MissingOpenParenAfterExtension,
                ),
            ),
        )
            .map(|(e, (o, a, c))| (e, o, a, c)),
        |i: &mut Input<'s>| pseudo_function::<true>(arena, i),
    ))
    .parse_next(i)
}

fn postfix<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Expression<'s, 'a>> {
    enum Function<'s, 'a> {
        Call(TokenRef<'s>, Vec<ArgElement<'s, 'a>>, TokenRef<'s>),
        TaggedString(ABox<'a, Expression<'s, 'a>>),
        Extension(
            TokenRef<'s>,
            Callable<'s, 'a>,
            TokenRef<'s>,
            Vec<ArgElement<'s, 'a>>,
            TokenRef<'s>,
        ),
        Access(TokenRef<'s>, TokenRef<'s>),
        Index(TokenRef<'s>, ABox<'a, Expression<'s, 'a>>, TokenRef<'s>),
        Slice(
            TokenRef<'s>,
            Option<ABox<'a, Expression<'s, 'a>>>,
            TokenRef<'s>,
            Option<ABox<'a, Expression<'s, 'a>>>,
            TokenRef<'s>,
        ),
        NonNil(TokenRef<'s>),
    }
    let first = primary(arena, i)?;
    let functions: Vec<Function<'s, 'a>> = repeat(
        0..,
        alt((
            (token(Operator::ColonColon), |i: &mut Input<'s>| {
                extension_call(arena, i)
            })
                .map(|(kw, (ex, o, a, c))| Function::Extension(kw, ex, o, a, c)),
            one_of(|t: &Token<'s>| matches!(t.kind, TokenKind::String(..))).map(|token| {
                Function::TaggedString(arena.alloc(Expression::Literal(TokenRef::borrow(token))))
            }),
            (|i: &mut Input<'s>| interpolation(arena, i))
                .map(|ex| Function::TaggedString(arena.alloc(ex))),
            (|i: &mut Input<'s>| access_index(arena, i)).map(|t| match t {
                AccessIndex::NonNil(token) => Function::NonNil(token),
                AccessIndex::Access(dot, token) => Function::Access(dot, token),
                AccessIndex::Index(open, exp, close) => Function::Index(open, exp, close),
                AccessIndex::Slice(left, start, op, end, right) => {
                    Function::Slice(left, start, op, end, right)
                }
            }),
            arg_list(arena, token(Operator::OpenParen)).map(|(o, a, c)| Function::Call(o, a, c)),
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
            Expression::Call(Callable::Expression(arena.alloc(acc)), o, args, c)
        }
        Function::TaggedString(ex) => Expression::TaggedString(arena.alloc(acc), ex),
        Function::Extension(e, ex, o, arg, c) => {
            Expression::Extension(arena.alloc(acc), e, ex, o, arg, c)
        }
        Function::Access(dot, token) => Expression::Access(arena.alloc(acc), dot, token),
        Function::Index(l, index, r) => Expression::Index(arena.alloc(acc), l, index, r),
        Function::Slice(left, start, op, end, right) => {
            Expression::Slice(arena.alloc(acc), left, start, op, end, right)
        }
        Function::NonNil(token) => Expression::NonNil(arena.alloc(acc), token),
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

fn pratt_prefix<'s: 'a, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Expression<'s, 'a>> {
    let token = peek(any).parse_next(i)?;
    let precedence = precedence_of(token);
    if precedence.can_be_prefix {
        let op = any.parse_next(i)?;
        let expr = pratt(arena, precedence, false)
            .verify_map(verify_expr)
            .parse_next(i)?;
        Ok(Expression::Prefix(op.into(), arena.alloc(expr)))
    } else {
        postfix(arena, i)
    }
}

fn pratt_infix<'s: 'a, 'a>(
    arena: &'a AstArena,
    left: ABox<'a, Expression<'s, 'a>>,
    op: &'s Token<'s>,
    mut precedence: PrecedenceResult,
    allow_range: bool,
    i: &mut Input<'s>,
) -> Result<Iterable<'s, 'a>> {
    if *op == Keyword::Is {
        let right = pattern(arena, false)
            .map(|p| arena.alloc(p))
            .parse_next(i)?;
        return Ok(Iterable::Value(Expression::Is(left, op.into(), right)));
    }
    // 调整优先级以实现右结合
    if precedence.right_associative {
        precedence.value -= 1;
    }
    let parse_right = |i: &mut Input<'s>| {
        let expr = pratt(arena, precedence, false)
            .verify_map(verify_expr)
            .parse_next(i)?;
        Ok(arena.alloc(expr))
    };
    if *op == Operator::Question {
        let then_exp = arena.alloc(expression(arena, i)?);
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

fn pratt<'s: 'a, 'a>(
    arena: &'a AstArena,
    precedence: PrecedenceResult,
    allow_range: bool,
) -> impl Parser<'s, Iterable<'s, 'a>> {
    move |i: &mut Input<'s>| {
        let mut left = pratt_prefix(arena, i)?;

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
            match pratt_infix(arena, arena.alloc(left), op, op_precedence, allow_range, i)? {
                Iterable::Value(e) => left = e,
                Iterable::Range(r) => return Ok(Iterable::Range(r)),
            }
        }

        Ok(Iterable::Value(left))
    }
}

fn verify_expr<'s, 'a>(e: Iterable<'s, 'a>) -> Option<Expression<'s, 'a>> {
    match e {
        Iterable::Value(e) => Some(e),
        _ => None,
    }
}

pub(super) fn basic_expression<'s: 'a, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    pratt(arena, PrecedenceResult::default(), false)
        .verify_map(verify_expr)
        .parse_next(i)
}

pub(super) fn iterable<'s: 'a, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Iterable<'s, 'a>> {
    pratt(arena, PrecedenceResult::default(), true).parse_next(i)
}

use winnow::{
    combinator::{alt, dispatch, fail, not, opt, peek, repeat, seq},
    token::any,
};

use super::{
    basic_expressions::iterable,
    expressions::{expression, expression_or_insert},
    helper::{statements_and_expression, token, token_or_insert},
    json_expressions::{json_expression, json_start},
    parameter_list::parameter_list,
    patterns::{pattern, pattern_or_insert},
    prelude::*,
};

fn optional_else<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Option<ElseBlock<'s, 'a>>> {
    let Some(kw_else) = opt(token(Keyword::Else)).parse_next(i)? else {
        return Ok(None);
    };

    let block = alt((
        |i: &mut Input<'s>| if_expression(arena, i),
        |i: &mut Input<'s>| block_expression(arena, i),
    ))
    .map(|e| arena.alloc(e))
    .parse_next(i)?;

    Ok(Some(ElseBlock(kw_else, block)))
}

pub(super) fn if_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    seq!(Expression::If(
        token(Keyword::If),
        expression_or_insert(arena, |t| *t == Operator::OpenBrace).map(|e| arena.alloc(e)),
        (|i: &mut Input<'s>| block_expression(arena, i)).map(|e| arena.alloc(e)),
        |i: &mut Input<'s>| optional_else(arena, i),
    ))
    .parse_next(i)
}

pub(super) fn block_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    not(json_start).parse_next(i)?;
    (
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace),
        |i: &mut Input<'s>| statements_and_expression(arena, i),
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace),
    )
        .map(|(open, (statements, expression), close)| {
            Expression::Block(open, statements, expression, close)
        })
        .parse_next(i)
}

pub(super) fn block_expression_no_expr<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    (
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace),
        |i: &mut Input<'s>| statements_and_expression(arena, i),
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace),
    )
        .map(|(open, (mut statements, expr), close)| {
            if let Some(expr) = expr {
                if expr.is_block_like() {
                    statements.push(Statement::BlockExpression(expr));
                } else {
                    let pos = expr.range();
                    statements.push(Statement::Expression(
                        expr,
                        TokenRef::new(Token::unknown_at(
                            pos.end,
                            Operator::Semicolon,
                            DiagnosticCode::MissingSemicolon,
                        )),
                    ));
                }
            }
            Expression::Block(open, statements, None, close)
        })
        .parse_next(i)
}

pub(super) fn fn_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    seq!(Expression::Function(
        token(Keyword::Fn),
        |i: &mut Input<'s>| parameter_list(arena, i),
        (|i: &mut Input<'s>| block_expression(arena, i)).map(|e| arena.alloc(e)),
    ))
    .parse_next(i)
}

pub(super) fn loop_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    seq!(Expression::Loop(
        token(Keyword::Loop),
        (|i: &mut Input<'s>| block_expression_no_expr(arena, i)).map(|e| arena.alloc(e)),
    ))
    .parse_next(i)
}

pub(super) fn while_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    seq!(Expression::While(
        token(Keyword::While),
        expression_or_insert(arena, |t| *t == Operator::OpenBrace).map(|e| arena.alloc(e)),
        (|i: &mut Input<'s>| block_expression_no_expr(arena, i)).map(|e| arena.alloc(e)),
        |i: &mut Input<'s>| optional_else(arena, i),
    ))
    .parse_next(i)
}

pub(super) fn match_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    fn branch_parser<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<MatchCase<'s, 'a>> {
        (
            alt((
                (
                    token(Keyword::Case),
                    pattern_or_insert(arena, false, |t| {
                        *t == Operator::OpenBrace || *t == Keyword::If
                    }),
                    opt((
                        token(Keyword::If),
                        expression_or_insert(arena, |t| *t == Operator::OpenBrace),
                    )),
                ),
                (
                    token_or_insert(Keyword::Case, DiagnosticCode::MissingCase),
                    pattern(arena, false),
                    opt((token(Keyword::If), |i: &mut Input<'s>| expression(arena, i))),
                ),
            )),
            |i: &mut Input<'s>| block_expression(arena, i),
        )
            .map(|((kw_case, pattern, guard), body)| MatchCase(kw_case, pattern, guard, body))
            .parse_next(i)
    }
    (
        token(Keyword::Match),
        expression_or_insert(arena, |t| *t == Operator::OpenBrace).map(|e| arena.alloc(e)),
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace),
        repeat(0.., |i: &mut Input<'s>| branch_parser(arena, i)),
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace),
    )
        .map(|(kw_match, expr, open, branches, close)| {
            Expression::Match(kw_match, expr, open, branches, close)
        })
        .parse_next(i)
}

pub(super) fn for_in_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    seq!(Expression::ForIn(
        token(Keyword::For),
        // 由后边的 `in` 定位，无条件插入
        pattern_or_insert(arena, false, |_| true).map(|p| arena.alloc(p)),
        token(Keyword::In),
        (|i: &mut Input<'s>| iterable(arena, i)).map(|e| arena.alloc(e)),
        (|i: &mut Input<'s>| block_expression_no_expr(arena, i)).map(|e| arena.alloc(e)),
        |i: &mut Input<'s>| optional_else(arena, i),
    ))
    .parse_next(i)
}

pub(super) fn block_like_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    dispatch! {peek(any);
        t if *t == Operator::OpenBrace => alt((
            |i: &mut Input<'s>| json_expression(arena, i),
            |i: &mut Input<'s>| block_expression(arena, i),
        )),
        t if *t == Keyword::If => |i: &mut Input<'s>| if_expression(arena, i),
        t if *t == Keyword::Fn => |i: &mut Input<'s>| fn_expression(arena, i),
        t if *t == Keyword::Loop => |i: &mut Input<'s>| loop_expression(arena, i),
        t if *t == Keyword::While => |i: &mut Input<'s>| while_expression(arena, i),
        t if *t == Keyword::Match => |i: &mut Input<'s>| match_expression(arena, i),
        t if *t == Keyword::For => |i: &mut Input<'s>| for_in_expression(arena, i),

        &Token{..} => fail,
    }
    .parse_next(i)
}

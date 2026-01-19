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

fn optional_else<'s>(i: &mut Input<'s>) -> Result<Option<ElseBlock<'s>>> {
    let Some(kw_else) = opt(token(Keyword::Else)).parse_next(i)? else {
        return Ok(None);
    };

    let block = alt((if_expression, block_expression))
        .map(Box::new)
        .parse_next(i)?;

    Ok(Some(ElseBlock(kw_else, block)))
}

pub(super) fn if_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    seq!(Expression::If(
        token(Keyword::If),
        expression_or_insert(|t| *t == Operator::OpenBrace).map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    not(json_start).parse_next(i)?;
    (
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace),
        statements_and_expression,
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace),
    )
        .map(|(open, (statements, expression), close)| {
            Expression::Block(open, statements, expression, close)
        })
        .parse_next(i)
}

pub(super) fn block_expression_no_expr<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    (
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace),
        statements_and_expression,
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
                        TokenRef::new(Token::unknown(
                            pos.end..pos.end,
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

pub(super) fn fn_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    seq!(Expression::Function(
        token(Keyword::Fn),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn loop_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    seq!(Expression::Loop(
        token(Keyword::Loop),
        block_expression_no_expr.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn while_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    seq!(Expression::While(
        token(Keyword::While),
        expression_or_insert(|t| *t == Operator::OpenBrace).map(Box::new),
        block_expression_no_expr.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn match_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    fn branch_parser<'s>(i: &mut Input<'s>) -> Result<MatchCase<'s>> {
        (
            alt((
                (
                    token(Keyword::Case),
                    pattern_or_insert(false, |t| *t == Operator::OpenBrace || *t == Keyword::If),
                    opt((
                        token(Keyword::If),
                        expression_or_insert(|t| *t == Operator::OpenBrace),
                    )),
                ),
                (
                    token_or_insert(Keyword::Case, DiagnosticCode::MissingCase),
                    pattern(false),
                    opt((token(Keyword::If), expression)),
                ),
            )),
            block_expression,
        )
            .map(|((kw_case, pattern, guard), body)| MatchCase(kw_case, pattern, guard, body))
            .parse_next(i)
    }
    (
        token(Keyword::Match),
        expression_or_insert(|t| *t == Operator::OpenBrace).map(Box::new),
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace),
        repeat(0.., branch_parser),
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace),
    )
        .map(|(kw_match, expr, open, branches, close)| {
            Expression::Match(kw_match, expr, open, branches, close)
        })
        .parse_next(i)
}

pub(super) fn for_in_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    seq!(Expression::ForIn(
        token(Keyword::For),
        // 由后边的 `in` 定位，无条件插入
        pattern_or_insert(false, |_| true).map(Box::new),
        token(Keyword::In),
        iterable.map(Box::new),
        block_expression_no_expr.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_like_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    dispatch! {peek(any);
        t if *t == Operator::OpenBrace => alt((
            json_expression,
            block_expression,
        )),
        t if *t == Keyword::If => if_expression,
        t if *t == Keyword::Fn => fn_expression,
        t if *t == Keyword::Loop => loop_expression,
        t if *t == Keyword::While => while_expression,
        t if *t == Keyword::Match => match_expression,
        t if *t == Keyword::For => for_in_expression,

        &Token{..} => fail,
    }
    .parse_next(i)
}

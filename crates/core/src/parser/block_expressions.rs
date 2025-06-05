use winnow::combinator::{alt, dispatch, fail, opt, peek, repeat, seq};
use winnow::prelude::*;
use winnow::token::any;

use crate::diagnostic::{DiagnosticCode, SourceRange};
use crate::lexer::{Keyword, Operator, Token};
use crate::parser::helper::statements_and_expression;

use super::expressions::expression;
use super::helper::{parameter_list, token, token_boxed, token_or_insert};
use super::iterables::iterable;
use super::patterns::{pattern, pattern_or_insert};
use super::{AstWalker, Expression, Input, Statement};

fn optional_else<'s>(
    i: &mut Input<'_, 's>,
) -> ModalResult<Option<(Box<Token<'s>>, Box<Expression<'s>>)>> {
    let Some(kw_else) = opt(token_boxed(Keyword::Else)).parse_next(i)? else {
        return Ok(None);
    };

    let block = alt((if_expression, block_expression))
        .map(Box::new)
        .parse_next(i)?;

    Ok(Some((kw_else, block)))
}

pub(super) fn if_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    seq!(Expression::If(
        token_boxed(Keyword::If),
        expression.map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    (
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace).map(Box::new),
        statements_and_expression,
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace).map(Box::new),
    )
        .map(|(open, (statements, expression), close)| {
            Expression::Block(open, statements, expression, close)
        })
        .parse_next(i)
}

pub(super) fn block_expression_no_expr<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    (
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace).map(Box::new),
        statements_and_expression,
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace).map(Box::new),
    )
        .map(|(open, (mut statements, expr), close)| {
            if let Some(expr) = expr {
                if expr.is_block_like() {
                    statements.push(Statement::BlockExpression(expr));
                } else {
                    let pos = expr.range();
                    let pos = SourceRange {
                        start: pos.end,
                        end: pos.end,
                    };
                    statements.push(Statement::Expression(
                        expr,
                        Box::new(Token::unknown(
                            pos.clone(),
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

pub(super) fn fn_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    seq!(Expression::Function(
        token_boxed(Keyword::Fn),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn loop_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    seq!(Expression::Loop(
        token_boxed(Keyword::Loop),
        block_expression_no_expr.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn while_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    seq!(Expression::While(
        token_boxed(Keyword::While),
        expression.map(Box::new),
        block_expression_no_expr.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn match_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    seq!(Expression::Match(
        token_boxed(Keyword::Match),
        expression.map(Box::new),
        token_or_insert(Operator::OpenBrace, DiagnosticCode::MissingOpenBrace).map(Box::new),
        repeat(
            0..,
            alt((
                // Avoid combine `token_or_insert` and `pattern_or_insert`
                // to prevent no consumption in `repeat`
                (
                    token(Keyword::Case),
                    pattern_or_insert(false),
                    block_expression,
                ),
                (
                    token_or_insert(Keyword::Case, DiagnosticCode::MissingCase),
                    pattern(false),
                    block_expression,
                ),
            ))
        ),
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingOpenBrace).map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn for_in_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    seq!(Expression::ForIn(
        token_boxed(Keyword::For),
        pattern_or_insert(false).map(Box::new),
        token_boxed(Keyword::In),
        iterable.map(Box::new),
        block_expression_no_expr.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_like_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    dispatch! {peek(any);
        t if *t == Operator::OpenBrace => block_expression,
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

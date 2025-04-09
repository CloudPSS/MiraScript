use winnow::combinator::{alt, dispatch, fail, opt, peek, repeat, seq};
use winnow::prelude::*;
use winnow::token::any;

use crate::lexer::{Keyword, Operator, Token};

use super::expressions::expression;
use super::helper::{parameter_list, token, token_boxed, token_or_insert};
use super::iterables::iterable;
use super::patterns::{pattern, pattern_or_insert};
use super::statements::statement;
use super::{Expression, Input};

fn optional_else<'a>(
    i: &mut Input<'_, 'a>,
) -> ModalResult<Option<(Box<Token<'a>>, Box<Expression<'a>>)>> {
    let Some(kw_else) = opt(token_boxed(Keyword::Else)).parse_next(i)? else {
        return Ok(None);
    };

    let block = alt((if_expression, block_expression))
        .map(Box::new)
        .parse_next(i)?;

    Ok(Some((kw_else, block)))
}

pub(super) fn if_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::If(
        token_boxed(Keyword::If),
        expression.map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Block(
        token_or_insert(Operator::OpenBrace, "Missing '{'").map(Box::new),
        repeat(0.., statement),
        opt(expression.map(Box::new)),
        token_or_insert(Operator::CloseBrace, "Missing '}'").map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn fn_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Function(
        token_boxed(Keyword::Fn),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn loop_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Loop(
        token_boxed(Keyword::Loop),
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn while_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::While(
        token_boxed(Keyword::While),
        expression.map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn match_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Match(
        token_boxed(Keyword::Match),
        expression.map(Box::new),
        token_or_insert(Operator::OpenBrace, "Missing '{'").map(Box::new),
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
                    token_or_insert(Keyword::Case, "Missing 'case'"),
                    pattern(false),
                    block_expression,
                ),
            ))
        ),
        token_or_insert(Operator::CloseBrace, "Missing '{'").map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn for_in_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::ForIn(
        token_boxed(Keyword::For),
        pattern_or_insert(false).map(Box::new),
        token_boxed(Keyword::In),
        iterable.map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_like_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
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

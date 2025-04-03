use winnow::combinator::{alt, dispatch, fail, opt, peek, repeat, seq};
use winnow::prelude::*;
use winnow::token::{any, one_of};

use super::expressions::expression;
use super::helper::{literal_boxed, literal_token, parameter_list, variable_token};
use super::iterables::iterable;
use super::statements::statement;
use super::{Expression, Input};
use crate::lexer::{Keyword, Operator, Token};
use crate::parser::helper::literal_or_insert;

fn optional_else<'a>(
    i: &mut Input<'_, 'a>,
) -> ModalResult<Option<(Box<Token<'a>>, Box<Expression<'a>>)>> {
    opt((
        literal_boxed(Keyword::Else),
        alt((if_expression, block_expression)).map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn if_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::If(
        literal_boxed(Keyword::If),
        expression.map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn block_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Block(
        literal_or_insert(Operator::OpenBrace, "Missing '{'").map(Box::new),
        repeat(0.., statement),
        opt(expression.map(Box::new)),
        literal_or_insert(Operator::CloseBrace, "Missing '}'").map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn fn_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Function(
        literal_boxed(Keyword::Fn),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn loop_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Loop(
        literal_boxed(Keyword::Loop),
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn while_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::While(
        literal_boxed(Keyword::While),
        expression.map(Box::new),
        block_expression.map(Box::new),
        optional_else,
    ))
    .parse_next(i)
}

pub(super) fn match_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Match(
        literal_boxed(Keyword::Match),
        expression.map(Box::new),
        literal_or_insert(Operator::OpenBrace, "Missing '{'").map(Box::new),
        repeat(
            0..,
            (
                literal_or_insert(Keyword::Case, "Missing 'case'"),
                alt((
                    literal_token,
                    one_of(|t: &Token<'a>| *t == Keyword::Underscore)
                        .map(|t: &Token<'a>| t.to_owned()),
                ))
                .map(|t: Token<'a>| Expression::Literal(Box::new(t))),
                block_expression,
            )
        ),
        literal_or_insert(Operator::CloseBrace, "Missing '{'").map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn for_in_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::ForIn(
        literal_boxed(Keyword::For),
        variable_token(true, false).map(Box::new),
        literal_boxed(Keyword::In),
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

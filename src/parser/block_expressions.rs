use winnow::combinator::{alt, dispatch, fail, opt, peek, preceded, repeat, seq};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use super::expressions::expression;
use super::helper::{literal_token, parameter_list, variable_token};
use super::statements::statement;
use super::{Expression, Input};
use crate::lexer::{Keyword, Operator, Token};

pub(super) fn if_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::If(
        _: literal(Keyword::If),
        expression.map(Box::new),
        block_expression.map(Box::new),
        opt(preceded(
            literal(Keyword::Else),
            alt((block_expression, if_expression)).map(Box::new)
        )),
    ))
    .parse_next(i)
}

pub(super) fn block_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Block(
        _: literal(Operator::OpenBrace),
        repeat(0.., statement),
        opt(expression.map(Box::new)),
        _: literal(Operator::CloseBrace),
    ))
    .parse_next(i)
}

pub(super) fn fn_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Function(
        _: literal(Keyword::Fn),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn loop_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Loop(
        _: literal(Keyword::Loop),
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn while_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::While(
        _: literal(Keyword::While),
        expression.map(Box::new),
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

pub(super) fn match_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Match(
        _: literal(Keyword::Match),
        expression.map(Box::new),
        _: literal(Operator::OpenBrace),
        repeat(
            0..,
            (
                alt((
                    literal_token,
                    one_of(|t: &Token<'a>| *t == Keyword::Underscore).map(|t: &Token<'a>| t.to_owned()),
                )).map(|t: Token<'a>| Expression::Literal(Box::new(t))),
                block_expression,
            )
        ),
        _: literal(Operator::CloseBrace),
    ))
    .parse_next(i)
}

pub(super) fn for_in_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::ForIn(
        _: literal(Keyword::For),
        variable_token(true, false).map(Box::new),
        _: literal(Keyword::In),
        expression.map(Box::new),
        block_expression.map(Box::new),
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

use winnow::combinator::{alt, opt, preceded, repeat, seq};
use winnow::prelude::*;
use winnow::token::literal;

use super::helper::parameter_list;
use super::statements::statement;
use crate::tokenizer::{Keyword, Operator};

use super::expressions::expression;
use super::{Expression, Input};

pub(super) fn if_expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
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

pub(super) fn block_expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Block(
        _: literal(Operator::OpenBrace),
        repeat(0.., statement),
        opt(expression.map(Box::new)),
        _: literal(Operator::CloseBrace),
    ))
    .parse_next(i)
}

pub(super) fn fn_expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    seq!(Expression::Function(
        _: literal(Keyword::Fn),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

use winnow::combinator::alt;
use winnow::prelude::*;

use super::{
    Expression, Input, basic_expressions::basic_expression,
    block_expressions::block_like_expression,
};

pub fn expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    alt((block_like_expression, basic_expression)).parse_next(i)
}

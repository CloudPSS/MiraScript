use winnow::combinator::{dispatch, fail, peek};
use winnow::prelude::*;
use winnow::token::any;

use crate::tokenizer::{Keyword, Operator, Token};

use super::{
    Expression, Input,
    basic_expressions::basic_expression,
    block_expressions::{block_expression, fn_expression, if_expression},
};

pub(super) fn expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(any);
        t if *t == Operator::OpenBrace => block_expression,
        t if *t == Keyword::If => if_expression,
        t if *t == Keyword::Fn => fn_expression,
        &Token{..}=> basic_expression,
    }
    .parse_next(i)
}

use winnow::combinator::{alt, dispatch, fail, opt, peek, preceded, repeat, seq, terminated};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use super::expressions::expression;
use crate::tokenizer::{Keyword, Operator, Range, Token, TokenError, TokenKind};

use super::{Input, Statement};

pub(super) fn statement<'a>(i: &mut Input<'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Expression(
        expression,
        _: literal(Operator::Semicolon),
    ))
    .parse_next(i)
}

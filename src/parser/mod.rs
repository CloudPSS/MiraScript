use winnow::combinator::{opt, repeat, seq};
use winnow::prelude::*;
use winnow::stream::TokenSlice;
use winnow::token::literal;

use crate::lexer::{Token, TokenKind};

mod basic_expressions;
mod block_expressions;
mod expression;
mod expressions;
mod helper;
mod iterable;
mod iterables;
mod range;
mod ranges;
mod script;
mod statement;
mod statements;

pub use expression::Expression;
pub use expressions::expression;
pub use iterable::Iterable;
pub use range::Range;
pub use script::Script;
pub use statement::Statement;

pub type Input<'t, 'a> = TokenSlice<'t, Token<'a>>;

pub fn to_input<'t, 'a>(tokens: &'t [Token<'a>]) -> Input<'t, 'a> {
    TokenSlice::new(tokens)
}

pub fn parse<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Script<'a>> {
    seq!(Script(
        repeat(0.., statements::statement),
        opt(expression.map(Box::new)),
        _: literal(TokenKind::Eof),
    ))
    .parse_next(i)
}

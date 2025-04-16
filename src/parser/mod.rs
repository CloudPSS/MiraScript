use winnow::prelude::*;
use winnow::stream::TokenSlice;

use crate::lexer::{Token, TokenKind};

mod array_element;
mod array_helper;
mod basic_expressions;
mod block_expressions;
mod expression;
mod expressions;
mod helper;
mod iterable;
mod iterables;
mod pattern;
mod patterns;
mod range;
mod ranges;
mod record_element;
mod record_helper;
mod script;
mod statement;
mod statements;

pub use array_element::{ArrayElement, ArrayPattern};
pub use expression::Expression;
pub use expressions::expression;
pub use iterable::Iterable;
pub use pattern::Pattern;
pub use range::Range;
pub use record_element::{RecordElement, RecordPattern};
pub use script::Script;
pub use statement::Statement;

pub type Input<'t, 'a> = TokenSlice<'t, Token<'a>>;

pub fn to_input<'t, 'a>(tokens: &'t [Token<'a>]) -> Input<'t, 'a> {
    TokenSlice::new(tokens)
}

pub fn parse<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Script<'a>> {
    (
        helper::statements_and_expression,
        helper::token_boxed(TokenKind::Eof),
    )
        .map(|((statements, expression), eof)| Script(statements, expression, eof))
        .parse_next(i)
}

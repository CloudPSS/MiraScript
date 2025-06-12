use winnow::prelude::*;
use winnow::stream::TokenSlice;

use crate::lexer::Token;

mod array_element;
mod array_helper;
mod ast_visitor;
mod basic_expressions;
mod block_expressions;
mod expression;
mod expressions;
mod helper;
mod iterable;
mod iterables;
mod list_item;
mod parameter_list;
mod pattern;
mod patterns;
mod range;
mod ranges;
mod record_element;
mod record_helper;
mod script;
mod scripts;
mod statement;
mod statements;

pub use array_element::{ArrayElement, ArrayElementBase, ArrayPattern};
pub(super) use ast_visitor::*;
pub use expression::{Callable, Expression};
pub use iterable::Iterable;
pub(super) use parameter_list::ParameterList;
pub use pattern::Pattern;
pub use range::Range;
pub use record_element::{RecordElement, RecordElementBase, RecordPattern};
pub use script::Script;
pub use statement::Statement;

pub type Input<'t, 's> = TokenSlice<'t, Token<'s>>;

pub fn to_input<'t, 's>(tokens: &'t [Token<'s>]) -> Input<'t, 's> {
    TokenSlice::new(tokens)
}

pub fn parse<'s>(i: &mut Input<'_, 's>) -> ModalResult<Script<'s>> {
    scripts::script.parse_next(i)
}

use winnow::{
    ModalResult, Parser as _,
    error::{EmptyError, ErrMode},
    stream::{Stateful, TokenSlice},
};

use bumpalo::Bump;

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
mod json_expressions;
mod list_item;
mod parameter_list;
mod pattern;
mod patterns;
mod range;
mod record_element;
mod record_helper;
mod script;
mod scripts;
mod statement;
mod statements;
mod token_ref;

pub use array_element::{ArgElement, ArrayElement, ArrayElementBase, ArrayPattern};
pub(super) use ast_visitor::*;
pub use expression::{Callable, ElseBlock, Expression, MatchCase};
pub use iterable::Iterable;
pub use list_item::ListItem;
pub use parameter_list::ParameterList;
pub use pattern::Pattern;
pub use range::Range;
pub use record_element::{RecordElement, RecordElementBase, RecordPattern};
pub use script::Script;
pub use statement::Statement;
pub use token_ref::TokenRef;

pub type AstBox<'s, T> = bumpalo::boxed::Box<'s, T>;
pub type Input<'s> = Stateful<TokenSlice<'s, Token<'s>>, &'s Bump>;
pub(crate) type Result<Output> = ModalResult<Output, EmptyError>;
trait Parser<'s, Output>: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>> + Copy {}

impl<'s, Output, F> Parser<'s, Output> for F where
    F: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>> + Copy
{
}

fn boxed<'s, Output: 's>(
    mut parser: impl Parser<'s, Output>,
) -> impl Parser<'s, AstBox<'s, Output>> {
    move |input: &mut Input<'s>| {
        let output = parser.parse_next(input)?;
        Ok(AstBox::new_in(output, input.state))
    }
}

mod prelude {
    pub(super) use super::{
        ArgElement, ArrayElement, ArrayElementBase, ArrayPattern, AstBox, AstWalker, Callable,
        ElseBlock, Expression, Input, Iterable, ListItem, MatchCase, ParameterList, Parser,
        Pattern, Range, RecordElement, RecordElementBase, RecordPattern, Result, Script, Statement,
        TokenRef, boxed,
    };
    pub(super) use crate::{
        diagnostic::{DiagnosticCode, DiagnosticsCollector, SourceDiagnostic, SourceRange},
        lexer::{Keyword, Operator, Token, TokenKind},
    };
    pub(super) use winnow::{
        Parser as _,
        stream::{Location as _, Stream as _},
    };
}

pub fn to_input<'s>(tokens: &'s [Token<'s>], arena: &'s Bump) -> Input<'s> {
    Stateful {
        input: TokenSlice::new(tokens),
        state: arena,
    }
}

pub fn parse<'s>(i: &mut Input<'s>) -> Result<Script<'s>> {
    scripts::script.parse_next(i)
}

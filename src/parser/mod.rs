use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::TokenSlice;
use winnow::token::any;

use crate::lexer::{Token, TokenKind};

mod basic_expressions;
mod block_expressions;
mod expression;
mod expressions;
mod helper;
mod script;
mod statement;
mod statements;

pub use expression::Expression;
pub use expressions::expression;
pub use script::Script;
pub use statement::Statement;

pub type Input<'t, 'a> = TokenSlice<'t, Token<'a>>;

pub fn to_input<'t, 'a>(tokens: &'t [Token<'a>]) -> Input<'t, 'a> {
    TokenSlice::new(tokens)
}

pub fn parse<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Script<'a>> {
    let mut statements = vec![];
    loop {
        let token = peek(any).parse_next(i)?;
        if *token == TokenKind::Eof {
            any.parse_next(i)?;
            break;
        }
        let statement = statements::statement.parse_next(i)?;
        statements.push(statement);
    }
    Ok(Script(statements))
}

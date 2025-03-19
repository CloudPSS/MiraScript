use std::fmt::Debug;

use winnow::Stateful;
use winnow::prelude::*;
use winnow::stream::TokenSlice;

use crate::tokenizer::Token;

mod basic_expression;
mod display;
mod expression;

#[derive(Debug)]
pub struct State<'a> {
    #[allow(clippy::vec_box)]
    // Boxed to ensure that the reference is stable
    resumed: Vec<Box<Token<'a>>>,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        Self {
            resumed: Vec::new(),
        }
    }

    pub fn add_token(&mut self, token: Token<'a>) -> TokenRef<'a> {
        let cell = Box::new(token);
        let ptr = &*cell as *const Token<'a>;
        self.resumed.push(cell);
        unsafe { &*ptr }
    }
}

pub type Input<'a> = Stateful<TokenSlice<'a, Token<'a>>, State<'a>>;

type TokenRef<'a> = &'a Token<'a>;

#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    // primary
    Value(TokenRef<'a>),
    Grouping(Box<Expression<'a>>),
    Tuple(Vec<Expression<'a>>),
    NamedTuple(Vec<(TokenRef<'a>, Expression<'a>)>),

    // function
    Call(Box<Expression<'a>>, Vec<Expression<'a>>),
    Access(Box<Expression<'a>>, TokenRef<'a>),

    // unary
    Not(Box<Expression<'a>>),
    Negate(Box<Expression<'a>>),
    Plus(Box<Expression<'a>>),

    // exponent
    Exponent(Box<Expression<'a>>, Box<Expression<'a>>),

    // factor
    Multiply(Box<Expression<'a>>, Box<Expression<'a>>),
    Divide(Box<Expression<'a>>, Box<Expression<'a>>),
    Modulo(Box<Expression<'a>>, Box<Expression<'a>>),

    // term
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    Subtract(Box<Expression<'a>>, Box<Expression<'a>>),

    // and
    And(Box<Expression<'a>>, Box<Expression<'a>>),

    // or
    Or(Box<Expression<'a>>, Box<Expression<'a>>),
}

pub fn to_input<'a>(tokens: &'a [Token<'a>]) -> Input<'a> {
    Stateful {
        input: TokenSlice::new(tokens),
        state: State::new(),
    }
}

pub fn lex<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    expression::expression.parse_next(i)
}

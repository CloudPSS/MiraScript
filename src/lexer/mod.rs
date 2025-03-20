use std::fmt::Debug;

use winnow::{LocatingSlice, ModalResult, Stateful};

mod comment;
mod keyword;
mod operator;
mod string;
mod token;
mod token_error;
mod token_kind;
mod tokens;

pub use comment::Comment;
pub use keyword::Keyword;
pub use operator::Operator;
pub use token::Token;
pub use token_error::TokenError;
pub use token_kind::TokenKind;

pub type Input<'a> = Stateful<LocatingSlice<&'a str>, State<'a>>;
pub type Range = std::ops::Range<usize>;

pub struct State<'a> {
    tokens: Vec<Box<[Token<'a>]>>,
}

impl Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}

impl<'a> State<'a> {
    pub fn add_tokens(&mut self, tokens: Vec<Token<'a>>) -> &'a [Token<'a>] {
        let cell = tokens.into_boxed_slice();
        let ptr = &*cell as *const [Token<'a>];
        self.tokens.push(cell);
        unsafe { &*ptr }
    }
}

pub fn to_input(text: &str) -> Input<'_> {
    Stateful {
        input: LocatingSlice::new(text),
        state: State { tokens: vec![] },
    }
}

pub fn lex<'a>(input: &mut Input<'a>, ignore_comments: bool) -> ModalResult<Vec<Token<'a>>> {
    let mut tokens = vec![];
    loop {
        let prev_token = &tokens.last();
        let token = tokens::token(input, prev_token)?;
        if ignore_comments && matches!(token.kind, TokenKind::Comment(_)) {
            continue;
        }
        let eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

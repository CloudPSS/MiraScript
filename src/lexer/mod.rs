use winnow::{LocatingSlice, ModalResult};

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

pub type Input<'a> = LocatingSlice<&'a str>;
pub type Range = std::ops::Range<usize>;

pub fn to_input(text: &str) -> Input<'_> {
    LocatingSlice::new(text)
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

pub fn lex_balanced<'a>(
    input: &mut Input<'a>,
    ignore_comments: bool,
    open: Operator,
    close: Operator,
) -> ModalResult<Vec<Token<'a>>> {
    let mut tokens = vec![];
    let mut depth = 0;
    while depth >= 0 {
        let prev_token = &tokens.last();
        let token = tokens::token(input, prev_token)?;
        if ignore_comments && matches!(token.kind, TokenKind::Comment(_)) {
            continue;
        }
        let eof = token.kind == TokenKind::Eof;
        if token == open {
            depth += 1;
        } else if token.kind == close {
            depth -= 1;
        }
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

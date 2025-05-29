use winnow::{LocatingSlice, ModalResult, Parser, error::ContextError};

mod identifier;
mod keyword;
mod numeric;
mod operator;
mod string;
mod token;
mod token_kind;
mod tokens;
mod trivia;

pub use keyword::Keyword;
pub use operator::Operator;
pub use token::Token;
pub use token_kind::TokenKind;
pub use trivia::Trivia;

pub type Input<'s> = LocatingSlice<&'s str>;

pub fn to_input(text: &str) -> Input<'_> {
    LocatingSlice::new(text)
}

fn lex_balanced_impl<'s, OP: PartialEq<TokenKind<'s>>, CP: PartialEq<TokenKind<'s>>>(
    input: &mut Input<'s>,
    mut depth: i32,
    open: OP,
    close: CP,
) -> ModalResult<Vec<Token<'s>>> {
    let mut tokens = vec![];
    while tokens.is_empty() || depth > 0 {
        let t = trivia::trivia_list(input)?;
        let prev_token = tokens.last_mut();
        let mut token = tokens::token(input, prev_token)?;
        token.leading_trivia = t;
        let eof = token.kind == TokenKind::Eof;
        if open == token.kind {
            depth += 1;
        } else if close == token.kind {
            depth -= 1;
        }
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

pub fn lex<'s>(input: &mut Input<'s>) -> ModalResult<Vec<Token<'s>>> {
    lex_balanced_impl(input, 1, TokenKind::Eof, TokenKind::Eof)
}

pub(crate) fn lex_balanced<'s, OP: PartialEq<TokenKind<'s>>, CP: PartialEq<TokenKind<'s>>>(
    input: &mut Input<'s>,
    open: OP,
    close: CP,
) -> ModalResult<Vec<Token<'s>>> {
    lex_balanced_impl(input, 0, open, close)
}

pub fn lex_string<'s>(input: &mut Input<'s>) -> ModalResult<Vec<Token<'s>>> {
    let mut str = string::string_content(None, 0)
        .with_span()
        .map(|(s, range)| Token {
            kind: s,
            range,
            leading_trivia: vec![],
            trailing_trivia: vec![],
        })
        .parse_next(input)?;
    let eof = tokens::token(input, Some(&mut str))?;

    if eof != TokenKind::Eof {
        return Err(winnow::error::ErrMode::Backtrack(ContextError::new()));
    }

    Ok(vec![str, eof])
}

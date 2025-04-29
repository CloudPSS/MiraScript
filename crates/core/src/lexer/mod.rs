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

pub fn lex<'s>(input: &mut Input<'s>) -> ModalResult<Vec<Token<'s>>> {
    let mut tokens = vec![];
    // let mut trivia = vec![];
    loop {
        let t = trivia::trivia_list(input)?;
        //   trivia.push(t);
        let prev_token = &tokens.last();
        let mut token = tokens::token(input, prev_token)?;
        token.leading_trivia = t;
        let eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

pub fn lex_string<'s>(input: &mut Input<'s>) -> ModalResult<Vec<Token<'s>>> {
    let str = string::string_content(None, 1)
        .with_span()
        .map(|(s, range)| Token {
            kind: s,
            range,
            leading_trivia: vec![],
            trailing_trivia: vec![],
        })
        .parse_next(input)?;
    let eof = tokens::token(input, &None)?;

    if eof != TokenKind::Eof {
        return Err(winnow::error::ErrMode::Backtrack(ContextError::new()));
    }

    Ok(vec![str, eof])
}

pub fn lex_balanced<'s>(
    input: &mut Input<'s>,
    open: Operator,
    close: Operator,
) -> ModalResult<Vec<Token<'s>>> {
    let mut tokens = vec![];
    let mut depth = 0;
    while tokens.is_empty() || depth > 0 {
        let t = trivia::trivia_list(input)?;
        let prev_token = &tokens.last();
        let mut token = tokens::token(input, prev_token)?;
        token.leading_trivia = t;
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

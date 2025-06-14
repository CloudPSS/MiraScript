use winnow::{
    LocatingSlice, ModalResult, Parser as _,
    error::{EmptyError, ErrMode},
};

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
pub(crate) type Result<Output> = ModalResult<Output, EmptyError>;
trait Parser<'s, Output>: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>> {}

impl<'s, Output, F> Parser<'s, Output> for F where
    F: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>>
{
}

pub fn to_input(text: &str) -> Input<'_> {
    LocatingSlice::new(text)
}

fn lex_balanced_impl<'s, OP: PartialEq<TokenKind<'s>>, CP: PartialEq<TokenKind<'s>>>(
    input: &mut Input<'s>,
    mut depth: i32,
    open: OP,
    close: CP,
) -> Result<Vec<Token<'s>>> {
    let mut tokens = vec![];
    while tokens.is_empty() || depth > 0 {
        let t = trivia::trivia_list(input)?;
        let prev_token = tokens.last_mut();
        let mut token = tokens::token(input, prev_token)?;
        #[cfg(feature = "trivia")]
        {
            token.leading_trivia = t;
        }
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

pub fn lex<'s>(input: &mut Input<'s>) -> Result<Vec<Token<'s>>> {
    lex_balanced_impl(input, 1, TokenKind::Eof, TokenKind::Eof)
}

pub(crate) fn lex_balanced<'s, OP: PartialEq<TokenKind<'s>>, CP: PartialEq<TokenKind<'s>>>(
    input: &mut Input<'s>,
    open: OP,
    close: CP,
) -> Result<Vec<Token<'s>>> {
    lex_balanced_impl(input, 0, open, close)
}

pub fn lex_string<'s>(input: &mut Input<'s>) -> Result<Vec<Token<'s>>> {
    let mut str = string::string_content(None, 1)
        .with_span()
        .map(|(s, range)| Token {
            kind: s,
            range,
            #[cfg(feature = "trivia")]
            leading_trivia: vec![],
            #[cfg(feature = "trivia")]
            trailing_trivia: vec![],
        })
        .parse_next(input)?;
    let eof = tokens::token(input, Some(&mut str))?;

    if eof != TokenKind::Eof {
        return Err(winnow::error::ErrMode::Backtrack(EmptyError));
    }

    Ok(vec![str, eof])
}

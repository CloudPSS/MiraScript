use char_count::{count_chars, count_from_start};
use string::apply_interpolation_offset;
use winnow::{LocatingSlice, ModalResult, Parser, error::ContextError};

mod char_count;
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

use crate::error::SourceRange;

pub type Input<'s> = LocatingSlice<&'s str>;

pub fn to_input(text: &str) -> Input<'_> {
    LocatingSlice::new(text)
}

fn lex_balanced_impl<'s, OP: PartialEq<TokenKind<'s>>, CP: PartialEq<TokenKind<'s>>>(
    input: &mut Input<'s>,
    mut index: usize,
    mut depth: i32,
    open: OP,
    close: CP,
) -> ModalResult<Vec<Token<'s>>> {
    let mut tokens = vec![];
    while tokens.is_empty() || depth > 0 {
        let (t, t_str) = trivia::trivia_list(input)?;
        let prev_token = &tokens.last();
        index += count_chars(t_str);
        let mut token = tokens::token(input, index, prev_token)?;
        token.leading_trivia = t;
        let eof = token.kind == TokenKind::Eof;
        if open == token.kind {
            depth += 1;
        } else if close == token.kind {
            depth -= 1;
        }
        index = token.range.end;
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

pub fn lex<'s>(input: &mut Input<'s>) -> ModalResult<Vec<Token<'s>>> {
    let start = count_from_start(input);
    lex_balanced_impl(input, start, 1, TokenKind::Eof, TokenKind::Eof)
}

pub(crate) fn lex_balanced<'s, OP: PartialEq<TokenKind<'s>>, CP: PartialEq<TokenKind<'s>>>(
    input: &mut Input<'s>,
    index: usize,
    open: OP,
    close: CP,
) -> ModalResult<Vec<Token<'s>>> {
    lex_balanced_impl(input, index, 0, open, close)
}

pub fn lex_string<'s>(input: &mut Input<'s>) -> ModalResult<Vec<Token<'s>>> {
    let start = count_from_start(input);
    let mut str = string::string_content(None, 0)
        .with_taken()
        .map(|(s, taken)| {
            let len = count_chars(taken);
            Token {
                kind: s,
                range: SourceRange {
                    start,
                    end: start + len,
                },
                leading_trivia: vec![],
                trailing_trivia: vec![],
            }
        })
        .parse_next(input)?;
    apply_interpolation_offset(&mut str, start);
    let eof = tokens::token(input, str.range.end, &Some(&str))?;

    if eof != TokenKind::Eof {
        return Err(winnow::error::ErrMode::Backtrack(ContextError::new()));
    }

    Ok(vec![str, eof])
}

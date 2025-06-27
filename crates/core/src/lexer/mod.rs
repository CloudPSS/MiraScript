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

pub(crate) use self::trivia::{Trivia, TriviaList};

pub(crate) use self::{
    numeric::NumberInfo,
    string::{StringFragment, StringInfo},
};

pub type Input<'s> = LocatingSlice<&'s str>;
pub(crate) type Result<Output> = ModalResult<Output, EmptyError>;
trait Parser<'s, Output>: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>> {}

impl<'s, Output, F> Parser<'s, Output> for F where
    F: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>>
{
}

mod prelude {
    pub(super) use super::{
        Input, Keyword, Operator, Parser, Result, Token, TokenKind, Trivia, TriviaList,
    };
    pub(super) use crate::diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
    pub(super) use winnow::{
        Parser as _,
        stream::{Location as _, Stream as _},
    };
}

pub fn to_input(text: &str) -> Input<'_> {
    LocatingSlice::new(text)
}

fn lex_impl<'s, const BALANCED: bool>(
    input: &mut Input<'s>,
    mut depth: i32,
    open: Operator,
    close: Operator,
) -> Result<Vec<Token<'s>>> {
    let mut tokens = vec![];
    while !BALANCED || tokens.is_empty() || depth > 0 {
        let leading_trivia = trivia::leading_trivia(input)?;
        let prev_token = tokens.last_mut();
        #[cfg_attr(not(feature = "formatter"), allow(unused_mut))]
        let mut token = tokens::token(input, prev_token)?;
        let tailing_trivia = trivia::tailing_trivia(input)?;

        #[cfg(feature = "formatter")]
        if crate::config::trivia() {
            token.leading_trivia = leading_trivia;
            token.tailing_trivia = tailing_trivia;
        }
        #[cfg(not(feature = "formatter"))]
        {
            let _ = leading_trivia;
            let _ = tailing_trivia;
        }

        let eof = matches!(token.kind, TokenKind::Eof);
        if BALANCED {
            if open == token.kind {
                depth += 1;
            } else if close == token.kind {
                depth -= 1;
            }
        }
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

pub fn lex<'s>(input: &mut Input<'s>) -> Result<Vec<Token<'s>>> {
    lex_impl::<false>(input, 1, Operator::Unknown, Operator::Unknown)
}

pub(crate) fn lex_balanced<'s>(
    input: &mut Input<'s>,
    open: Operator,
    close: Operator,
) -> Result<Vec<Token<'s>>> {
    lex_impl::<true>(input, 0, open, close)
}

pub fn lex_string<'s>(input: &mut Input<'s>) -> Result<Vec<Token<'s>>> {
    let info = StringInfo {
        leading_range: 0..0,
        trailing_range: input.len()..input.len(),
        ats: 1,
        quote: None,
        content: vec![],
    };
    let mut str = string::string_content(info)
        .with_span()
        .map(|(s, range)| Token::new(s, range))
        .parse_next(input)?;
    let eof = tokens::token(input, Some(&mut str))?;

    if !matches!(eof.kind, TokenKind::Eof) {
        return Err(winnow::error::ErrMode::Backtrack(EmptyError));
    }

    Ok(vec![str, eof])
}

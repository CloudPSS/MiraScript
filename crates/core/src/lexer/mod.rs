use winnow::{
    LocatingSlice, ModalResult, Parser as _, Stateful,
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

use crate::Config;

#[allow(unused)]
pub(crate) use self::{
    numeric::NumberInfo,
    string::{StringFragment, StringInfo},
    trivia::{Trivia, TriviaList},
};

pub type Input<'s> = Stateful<LocatingSlice<&'s str>, &'s Config>;
pub(crate) type Result<Output> = ModalResult<Output, EmptyError>;
trait Parser<'s, Output>: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>> {}

impl<'s, Output, F> Parser<'s, Output> for F where
    F: winnow::Parser<Input<'s>, Output, ErrMode<EmptyError>>
{
}

mod prelude {
    pub(super) use super::*;
    pub(super) use crate::diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
    pub(super) use winnow::stream::{Location as _, Stream as _};
}

pub fn to_input<'s>(text: &'s str, config: &'s Config) -> Input<'s> {
    Input {
        input: LocatingSlice::new(text),
        state: config,
    }
}

fn lex_impl<'s, const BALANCED: bool>(
    input: &mut Input<'s>,
    mut depth: i32,
    open: Operator,
    close: Operator,
) -> Result<Vec<Token<'s>>> {
    let mut tokens = vec![];
    loop {
        #[cfg_attr(not(feature = "formatter"), allow(unused))]
        let leading_trivia = trivia::leading_trivia(input)?;
        let prev_token = tokens.last_mut();
        #[cfg_attr(not(feature = "formatter"), allow(unused_mut))]
        let mut token = tokens::token(input, prev_token)?;

        if BALANCED {
            if open == token.kind {
                depth += 1;
            } else if close == token.kind {
                depth -= 1;
            }
        }

        let eof = matches!(token.kind, TokenKind::Eof);
        if BALANCED && depth <= 0 || eof {
            #[cfg(feature = "formatter")]
            if input.state.trivia {
                token.leading_trivia = leading_trivia;
            }
            tokens.push(token);
            break;
        }

        #[cfg_attr(not(feature = "formatter"), allow(unused))]
        let tailing_trivia = trivia::tailing_trivia(input)?;

        #[cfg(feature = "formatter")]
        if input.state.trivia {
            token.leading_trivia = leading_trivia;
            token.tailing_trivia = tailing_trivia;
        }
        tokens.push(token);
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

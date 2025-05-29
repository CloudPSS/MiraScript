use winnow::ascii::space0;
use winnow::combinator::{alt, dispatch, eof, fail, opt, peek, preceded};
use winnow::prelude::*;
use winnow::token::{any, take};

use crate::error::{ErrorCode, SourceError};

use super::identifier::{identifier, is_identifier_special, is_identifier_start};
use super::numeric::{number, ordinal};
use super::{Input, Operator, Token, TokenKind, string};

pub(super) fn token<'s>(
    input: &mut Input<'s>,
    prev_token: Option<&mut Token<'s>>,
) -> ModalResult<Token<'s>> {
    let dot = prev_token.as_deref().is_some_and(|t| *t == Operator::Dot);
    let token_parser = |i: &mut Input<'s>| {
        dispatch! {peek(any);
            '0'..='9' => if dot {
                ordinal
            } else {
                number
            },
            '@' => alt((
                string::string,
                identifier(!dot),
            )),
            '"' | '\'' | '`' => string::string,
            '+' => alt((
                "+=".value(TokenKind::Operator(Operator::PlusEqual)),
                any.value(TokenKind::Operator(Operator::Plus)),
            )),
            '-' => alt((
                "-=".value(TokenKind::Operator(Operator::MinusEqual)),
                "->".value(TokenKind::Operator(Operator::Arrow)),
                any.value(TokenKind::Operator(Operator::Minus)),
            )),
            '*' => alt((
                "*=".value(TokenKind::Operator(Operator::AsteriskEqual)),
                any.value(TokenKind::Operator(Operator::Asterisk)),
            )),
            '/' => dispatch! {peek(opt(take(2usize)));
                // Some("/*") => block_comment,
                // Some("//") => line_comment,
                Some("/=") => take(2usize).value(TokenKind::Operator(Operator::SlashEqual)),
                _ => any.value(TokenKind::Operator(Operator::Slash)),
            },
            '%' => alt((
                "%=".value(TokenKind::Operator(Operator::PercentEqual)),
                any.value(TokenKind::Operator(Operator::Percent)),
            )),
            '^' => alt((
                "^=".value(TokenKind::Operator(Operator::CaretEqual)),
                any.value(TokenKind::Operator(Operator::Caret)),
            )),
            '=' => alt((
                "==".value(TokenKind::Operator(Operator::EqualEqual)),
                any.value(TokenKind::Operator(Operator::Equal)),
            )),
            '~' => "~=".value(TokenKind::Operator(Operator::TildeEqual)),
            '!' => alt((
                ("!", peek("==")).value(TokenKind::Operator(Operator::Exclamation)),
                ("!", peek("::")).value(TokenKind::Operator(Operator::Exclamation)),
                "!:".value(TokenKind::Operator(Operator::ExclamationColon)),
                "!=".value(TokenKind::Operator(Operator::NotEqual)),
                "!~=".value(TokenKind::Operator(Operator::NotTildeEqual)),
                any.value(TokenKind::Operator(Operator::Exclamation)),
            )),
            '>' => alt((
                ">=".value(TokenKind::Operator(Operator::GreaterEqual)),
                ">".value(TokenKind::Operator(Operator::Greater)),
            )),
            '<' => alt((
                "<=".value(TokenKind::Operator(Operator::LessEqual)),
                "<".value(TokenKind::Operator(Operator::Less)),
            )),
            '&' => alt((
                "&&=".value(TokenKind::Operator(Operator::LogicalAndEqual)),
                "&&".value(TokenKind::Operator(Operator::LogicalAnd)),
            )),
            '|' => alt((
                "||=".value(TokenKind::Operator(Operator::LogicalOrEqual)),
                "||".value(TokenKind::Operator(Operator::LogicalOr)),
            )),
            '?' => alt((
                "?:".value(TokenKind::Operator(Operator::QuestionColon)),
                "??=".value(TokenKind::Operator(Operator::NullCoalescingEqual)),
                "??".value(TokenKind::Operator(Operator::NullCoalescing)),
            )),
            '(' => any.value(TokenKind::Operator(Operator::OpenParen)),
            ')' => any.value(TokenKind::Operator(Operator::CloseParen)),
            '[' => any.value(TokenKind::Operator(Operator::OpenBracket)),
            ']' => any.value(TokenKind::Operator(Operator::CloseBracket)),
            ':' => alt((
                "::".value(TokenKind::Operator(Operator::ColonColon)),
                any.value(TokenKind::Operator(Operator::Colon)),
            )),
            ',' => any.value(TokenKind::Operator(Operator::Comma)),
            '.' => alt((
                "..<".value(TokenKind::Operator(Operator::HalfOpenRange)),
                "..".value(TokenKind::Operator(Operator::SpreadRange)),
                ".".value(TokenKind::Operator(Operator::Dot)),
            )),

            ';' => any.value(TokenKind::Operator(Operator::Semicolon)),
            '{' => any.value(TokenKind::Operator(Operator::OpenBrace)),
            '}' => any.value(TokenKind::Operator(Operator::CloseBrace)),

            c if is_identifier_start(c) || is_identifier_special(c) => identifier(!dot),

            _ => fail,
        }
        .parse_next(i)
    };
    let cur_token = preceded(
        space0,
        alt((
            token_parser,
            eof.map(|_| TokenKind::Eof),
            any.span().map(|range| TokenKind::Unknown {
                recovered: None,
                errors: vec![SourceError::new(range, ErrorCode::UnknownToken)],
            }),
        ))
        .with_span()
        .map(|(kind, range)| Token {
            range,
            kind,
            leading_trivia: vec![],
            trailing_trivia: vec![],
        }),
    )
    .parse_next(input)?;
    if (cur_token == Operator::Colon
        || cur_token == Operator::QuestionColon
        || cur_token == Operator::ExclamationColon)
        && matches!(prev_token.as_deref(), Some(prev_token) if prev_token.is_keyword())
    {
        let Some(prev_token) = prev_token else {
            unreachable!();
        };
        let TokenKind::Keyword(kw) = prev_token.kind else {
            unreachable!();
        };
        prev_token.kind = TokenKind::Identifier(kw.to_string().into());
    }
    Ok(cur_token)
}

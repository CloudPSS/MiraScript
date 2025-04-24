use std::borrow::Cow;
use std::str::FromStr;

use winnow::ascii::space0;
use winnow::combinator::{alt, dispatch, eof, fail, opt, peek, preceded, trace};
use winnow::prelude::*;
use winnow::token::{any, one_of, take, take_while};

use crate::error::{ErrorCode, SourceError};

use super::helper::{is_identifier_continue, is_identifier_start};
use super::numeric::{number, ordinal};
use super::{Input, Keyword, Operator, Token, TokenKind, string};

pub(super) fn identifier<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    trace(
        "identifier",
        (
            one_of(is_identifier_start),
            take_while(0.., is_identifier_continue),
        )
            .take()
            .map(|s| {
                if let Ok(kw) = Keyword::from_str(s) {
                    TokenKind::Keyword(kw)
                } else {
                    TokenKind::Identifier(Cow::Borrowed(s))
                }
            }),
    )
    .parse_next(i)
}

pub(super) fn token<'a>(
    input: &mut Input<'a>,
    prev_token: &Option<&Token<'a>>,
) -> ModalResult<Token<'a>> {
    let token = |i: &mut Input<'a>| {
        dispatch!{peek(any);
            '0'..='9' => if prev_token.map(|t| &t.kind) == Some(&TokenKind::Operator(Operator::Dot)) {
                ordinal
            } else {
                number
            },
            '@' => alt((
                string::string,
                identifier,
            )),
            '"' | '\'' | '`' => string::string,
            '+' => alt((
                "+=".value(TokenKind::Operator(Operator::PlusEqual)),
                any.value(TokenKind::Operator(Operator::Plus)),
            )),
            '-' => alt((
                "-=".value(TokenKind::Operator(Operator::MinusEqual)),
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
            '&' => "&&".value(TokenKind::Operator(Operator::LogicalAnd)),
            '|' =>"||".value(TokenKind::Operator(Operator::LogicalOr)),
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

            c if is_identifier_start(c) => identifier,

            _ => fail,
        }.parse_next(i)
    };
    preceded(
        space0,
        alt((
            token,
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
    .parse_next(input)
}

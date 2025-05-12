use winnow::ascii::space0;
use winnow::combinator::{alt, dispatch, eof, fail, opt, peek};
use winnow::prelude::*;
use winnow::token::{any, take};

use crate::error::{ErrorCode, SourceError, SourceRange};

use super::char_count::count_chars;
use super::identifier::{identifier, is_identifier_special, is_identifier_start};
use super::numeric::{number, ordinal};
use super::string::apply_interpolation_offset;
use super::{Input, Operator, Token, TokenKind, string};

pub(super) fn token<'s>(
    input: &mut Input<'s>,
    prev_index: usize,
    prev_token: &Option<&Token<'s>>,
) -> ModalResult<Token<'s>> {
    let token_parser = |i: &mut Input<'s>| {
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

            c if is_identifier_start(c) || is_identifier_special(c) => identifier,

            _ => fail,
        }.parse_next(i)
    };
    let (sp, token) = (
        space0,
        alt((
            token_parser,
            eof.map(|_| TokenKind::Eof),
            any.span().map(|range| TokenKind::Unknown {
                recovered: None,
                errors: vec![SourceError::new(range, ErrorCode::UnknownToken)],
            }),
        ))
        .with_taken(),
    )
        .parse_next(input)?;
    let start = prev_index + count_chars(sp);
    let end = start + count_chars(token.1);
    let mut t = Token {
        kind: token.0,
        range: SourceRange { start, end },
        leading_trivia: vec![],
        trailing_trivia: vec![],
    };
    apply_interpolation_offset(&mut t, start);
    Ok(t)
}

use winnow::{
    ascii::space0,
    combinator::{alt, dispatch, eof, fail, opt, peek, preceded},
    token::{any, take},
};

use super::{
    identifier::{identifier, is_identifier_special, is_identifier_start},
    numeric::{number, ordinal},
    prelude::*,
    string,
};

pub(super) fn token<'s>(
    input: &mut Input<'s>,
    prev_token: Option<&mut Token<'s>>,
) -> Result<Token<'s>> {
    let dot = prev_token.as_deref().is_some_and(|t| *t == Operator::Dot);
    let token_parser = |i: &mut Input<'s>| {
        dispatch! {peek(any);
            '0'..='9' => if dot {
                ordinal
            } else {
                number
            },
            '#' => any.value(TokenKind::Operator(Operator::Format)),
            '@' => alt((
                string::string,
                identifier(!dot),
            )),
            '"' | '\'' | '`' => string::string,
            '+' => alt((
                "+=".value(TokenKind::Operator(Operator::PlusAssign)),
                any.value(TokenKind::Operator(Operator::Plus)),
            )),
            '-' => alt((
                "-=".value(TokenKind::Operator(Operator::MinusAssign)),
                "->".value(TokenKind::Operator(Operator::Arrow)),
                any.value(TokenKind::Operator(Operator::Minus)),
            )),
            '*' => alt((
                "*=".value(TokenKind::Operator(Operator::AsteriskAssign)),
                any.value(TokenKind::Operator(Operator::Asterisk)),
            )),
            '/' => dispatch! {peek(opt(take(2usize)));
                // Some("/*") => block_comment,
                // Some("//") => line_comment,
                Some("/=") => take(2usize).value(TokenKind::Operator(Operator::SlashAssign)),
                _ => any.value(TokenKind::Operator(Operator::Slash)),
            },
            '%' => alt((
                "%=".value(TokenKind::Operator(Operator::PercentAssign)),
                any.value(TokenKind::Operator(Operator::Percent)),
            )),
            '^' => alt((
                "^=".value(TokenKind::Operator(Operator::CaretAssign)),
                any.value(TokenKind::Operator(Operator::Caret)),
            )),
            '=' => alt((
                "==".value(TokenKind::Operator(Operator::Equal)),
                "=~".value(TokenKind::Operator(Operator::TildeEqual)),
                any.value(TokenKind::Operator(Operator::Assign)),
            )),
            '!' => alt((
                ("!", peek(alt(("==", "=~")))).value(TokenKind::Operator(Operator::Exclamation)),
                "!=".value(TokenKind::Operator(Operator::NotEqual)),
                "!~".value(TokenKind::Operator(Operator::TildeNotEqual)),
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
                "&&=".value(TokenKind::Operator(Operator::LogicalAndAssign)),
                "&&".value(TokenKind::Operator(Operator::LogicalAnd)),
            )),
            '|' => alt((
                "||=".value(TokenKind::Operator(Operator::LogicalOrAssign)),
                "||".value(TokenKind::Operator(Operator::LogicalOr)),
            )),
            '?' => alt((
                "?:".value(TokenKind::Operator(Operator::QuestionColon)),
                "??=".value(TokenKind::Operator(Operator::NullCoalescingAssign)),
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
            any.span().map(|range| {
                TokenKind::unknown(TokenKind::Empty, range, DiagnosticCode::UnknownToken)
            }),
        ))
        .with_span()
        .map(|(kind, range)| Token::new(kind, range)),
    )
    .parse_next(input)?;

    if matches!(prev_token.as_deref(), Some(prev_token) if prev_token.is_keyword()) {
        let Some(prev_token) = prev_token else {
            unreachable!();
        };
        let TokenKind::Keyword(kw) = prev_token.kind else {
            unreachable!();
        };
        if cur_token == Operator::Colon || cur_token == Operator::QuestionColon {
            prev_token.kind = TokenKind::Identifier(kw.into());
        }
    }
    Ok(cur_token)
}

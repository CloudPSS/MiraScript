use std::borrow::Cow;
use std::str::FromStr;

use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use winnow::ascii::{digit0, digit1, line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, dispatch, eof, fail, opt, peek, preceded, repeat, trace};
use winnow::error::{StrContext, StrContextValue};
use winnow::prelude::*;
use winnow::token::{any, one_of, take, take_until, take_while};

use crate::utils::{SourceError, SourceRange};

use super::helper::{is_identifier_continue, is_identifier_start};
use super::{Comment, Input, Keyword, Operator, Token, TokenKind, string};

fn line_comment<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    ("//", till_line_ending, opt(line_ending))
        .value(TokenKind::Comment(Comment::Line))
        .parse_next(i)
}

fn block_comment<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    preceded(
        "/*",
        alt((
            (take_until(0.., "*/"), "*/").value(TokenKind::Comment(Comment::Block)),
            repeat(0.., any)
                .fold(|| (), |_: (), _| ())
                .span()
                .map(|mut r: SourceRange| {
                    r.start = r.end;
                    TokenKind::unknown_range(
                        TokenKind::Comment(Comment::Block),
                        r,
                        "Unterminated block comment",
                    )
                }),
        )),
    )
    .parse_next(i)
}

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

fn number<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    trace("number", move |i: &mut Input<'a>| {
        let initial = (peek(opt((any, any)))).parse_next(i)?;
        match initial {
            Some(('0', ch1))
                if is_identifier_continue(ch1) && ch1 != '_' && ch1 != 'e' && ch1 != 'E' =>
            {
                (take_while(2.., is_identifier_continue).with_span().map(
                    |(s, r): (&str, SourceRange)| {
                        let ends_with_underscore = s.ends_with("_");
                        let radix;
                        let is_valid_char = if s.starts_with("0x") {
                            radix = 16;
                            |c: char| c.is_ascii_hexdigit()
                        } else if s.starts_with("0o") {
                            radix = 8;
                            |c: char| ('0'..='7').contains(&c)
                        } else if s.starts_with("0b") {
                            radix = 2;
                            |c: char| c == '0' || c == '1'
                        } else {
                            return TokenKind::unknown_range(
                                TokenKind::Number(0.0),
                                r,
                                "Invalid base prefix for number literal",
                            );
                        };

                        let s = &s[2..];
                        let mut errors = vec![];
                        let num = if s.chars().all(is_valid_char) {
                            if s.is_empty() {
                                errors.push(SourceError::new(
                                    r.clone(),
                                    "No valid digits for number literal",
                                ));
                            }
                            BigUint::parse_bytes(s.as_bytes(), radix)
                        } else {
                            let mut num = String::new();
                            for (i, c) in s.char_indices() {
                                if c == '_' {
                                    continue;
                                } else if is_valid_char(c) {
                                    num.push(c);
                                } else {
                                    let mut range = r.clone();
                                    range.start += i + 2;
                                    range.end = range.start + c.len_utf8();
                                    errors.push(SourceError::new(
                                        range,
                                        "Invalid character in number literal",
                                    ));
                                }
                            }
                            if num.is_empty() {
                                errors.push(SourceError::new(
                                    r.clone(),
                                    "No valid digits for number literal",
                                ));
                            }
                            BigUint::parse_bytes(num.as_bytes(), radix)
                        }
                        .unwrap_or_default();
                        let float = num.to_f64().unwrap();
                        if ends_with_underscore {
                            errors.push(SourceError::new(
                                r.clone(),
                                "Number literal cannot end with underscore",
                            ));
                        }
                        if float.is_infinite() {
                            errors.push(SourceError::new(r, "Number literal is too large"));
                        }
                        if !errors.is_empty() {
                            TokenKind::unknown_errors(TokenKind::Number(float), errors)
                        } else {
                            TokenKind::Number(float)
                        }
                    },
                ))
                .parse_next(i)
            }
            _ => (cut_err(
                (
                    digit0,
                    opt(('.', digit1)),
                    opt((one_of(['e', 'E']), opt(one_of(['+', '-'])), digit1)),
                )
                    .take()
                    .map(|s: &str| TokenKind::Number(s.parse().expect("valid float"))),
            ))
            .parse_next(i),
        }
    })
    .parse_next(i)
}

fn ordinal<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    // must be integer with NO leading zeros
    trace(
        "ordinal",
        cut_err(
            take_while(1.., is_identifier_continue)
                .verify(|s: &str| {
                    s == "0" || !s.starts_with('0') && s.chars().all(|c| c.is_ascii_digit())
                })
                .context(StrContext::Label("ordinal"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "integer",
                ))),
        )
        .map(|s: &str| TokenKind::Ordinal(s.parse().unwrap())),
    )
    .parse_next(i)
}

pub(super) fn token<'a>(
    input: &mut Input<'a>,
    prev_token: &Option<&Token<'a>>,
) -> ModalResult<Token<'a>> {
    preceded(take_while(0.., |c: char| c.is_ascii_whitespace()),  alt((
        dispatch! {peek(any);
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
            '+' => any.value(TokenKind::Operator(Operator::Plus)),
            '-' => any.value(TokenKind::Operator(Operator::Minus)),
            '^' => any.value(TokenKind::Operator(Operator::Caret)),
            '*' => any.value(TokenKind::Operator(Operator::Asterisk)),
            '%' => any.value(TokenKind::Operator(Operator::Percent)),
            '=' => dispatch! {peek(opt(take(2usize)));
                Some("==") => take(2usize).value(TokenKind::Operator(Operator::EqualEqual)),
                _ => any.value(TokenKind::Operator(Operator::Equal)),
            },
            '!' => dispatch! {peek(opt(take(2usize)));
                Some("!=") => take(2usize).value(TokenKind::Operator(Operator::NotEqual)),
                _ => any.value(TokenKind::Operator(Operator::LogicalNot)),
            },
            '>' => dispatch! {peek(opt(take(2usize)));
                Some(">=") => take(2usize).value(TokenKind::Operator(Operator::GreaterEqual)),
                _ => any.value(TokenKind::Operator(Operator::Greater)),
            },
            '<' => dispatch! {peek(opt(take(2usize)));
                Some("<=") => take(2usize).value(TokenKind::Operator(Operator::LessEqual)),
                Some("<|") => take(2usize).value(TokenKind::Operator(Operator::BackwardPipe)),
                _ => any.value(TokenKind::Operator(Operator::Less)),
            },
            '&' => dispatch! {peek(opt(take(2usize)));
                Some("&&") => take(2usize).value(TokenKind::Operator(Operator::LogicalAnd)),
                _ => fail,
            },
            '|' => dispatch! {peek(opt(take(2usize)));
                Some("||") => take(2usize).value(TokenKind::Operator(Operator::LogicalOr)),
                Some("|>") => take(2usize).value(TokenKind::Operator(Operator::ForwardPipe)),
                _ => fail,
            },
            '(' => any.value(TokenKind::Operator(Operator::OpenParen)),
            ')' => any.value(TokenKind::Operator(Operator::CloseParen)),
            '[' => any.value(TokenKind::Operator(Operator::OpenBracket)),
            ']' => any.value(TokenKind::Operator(Operator::CloseBracket)),
            ':' => any.value(TokenKind::Operator(Operator::Colon)),
            ',' => any.value(TokenKind::Operator(Operator::Comma)),
            '.' => dispatch! {peek(opt(take(2usize)));
                Some("..") => dispatch! {peek(opt(take(3usize)));
                    Some("..<") => take(3usize).value(TokenKind::Operator(Operator::HalfOpenRange)),
                    _ => take(2usize).value(TokenKind::Operator(Operator::SpreadRange)),
                },
                _ => any.value(TokenKind::Operator(Operator::Dot)),
            },

            '/' => dispatch! {peek(opt(take(2usize)));
                Some("/*") => block_comment,
                Some("//") => line_comment,
                _ => any.value(TokenKind::Operator(Operator::Slash)),
            },
            ';' => any.value(TokenKind::Operator(Operator::Semicolon)),
            '{' => any.value(TokenKind::Operator(Operator::OpenBrace)),
            '}' => any.value(TokenKind::Operator(Operator::CloseBrace)),
            c if is_identifier_start(c) => identifier,
            _ => fail,
        },
        eof.map(|_|TokenKind::Eof),
        any.span().map(|range| TokenKind::Unknown {
            recovered: None,
            errors: vec![SourceError::new(range, "Unknown token")],
        }),
    ))
    .with_span()
    .map(|(kind, range)| Token { range, kind })
    )
    .parse_next(input)
}

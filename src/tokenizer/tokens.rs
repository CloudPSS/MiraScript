use std::borrow::Cow;

use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use unicode_ident::{is_xid_continue, is_xid_start};
use winnow::ascii::{digit0, digit1, line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, dispatch, eof, fail, opt, peek, trace};
use winnow::error::{StrContext, StrContextValue};
use winnow::prelude::*;
use winnow::token::{any, one_of, take, take_until, take_while};

use super::{Input, Keyword, Operator, Range, Token, TokenError, TokenKind, Whitespace, string};

fn line_comment(i: &mut Input<'_>) -> ModalResult<()> {
    ("//", till_line_ending, opt(line_ending))
        .void()
        .parse_next(i)
}

fn block_comment(i: &mut Input<'_>) -> ModalResult<()> {
    (
        "/*",
        take_until(0.., "*/").context(StrContext::Expected(StrContextValue::StringLiteral("*/"))),
        "*/",
    )
        .context(StrContext::Label("block comment"))
        .void()
        .parse_next(i)
}

fn identifier<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    trace(
        "identifier",
        (
            one_of(|c: char| c == '_' || c == '$' || is_xid_start(c)),
            take_while(0.., |c: char| c == '$' || is_xid_continue(c)),
        )
            .take()
            .map(|s| match s {
                "and" => TokenKind::Keyword(Keyword::and),
                "or" => TokenKind::Keyword(Keyword::or),
                "not" => TokenKind::Keyword(Keyword::not),

                "if" => TokenKind::Keyword(Keyword::r#if),
                "else" => TokenKind::Keyword(Keyword::r#else),
                "match" => TokenKind::Keyword(Keyword::r#match),
                "for" => TokenKind::Keyword(Keyword::r#for),
                "in" => TokenKind::Keyword(Keyword::r#in),
                "while" => TokenKind::Keyword(Keyword::r#while),
                "loop" => TokenKind::Keyword(Keyword::r#loop),
                "break" => TokenKind::Keyword(Keyword::r#break),
                "continue" => TokenKind::Keyword(Keyword::r#continue),
                "return" => TokenKind::Keyword(Keyword::r#return),

                "fn" => TokenKind::Keyword(Keyword::r#fn),
                "op" => TokenKind::Keyword(Keyword::op),

                s => TokenKind::Identifier(Cow::Borrowed(s)),
            }),
    )
    .parse_next(i)
}

fn number<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    trace("number", move |i: &mut Input<'a>| {
        let initial = (peek(opt(take(2usize)))).parse_next(i)?;
        match initial {
            Some(prefix)
                if (prefix.starts_with('0')
                    && !prefix.ends_with('_')
                    && !prefix.ends_with('.')
                    && !prefix.ends_with('e')
                    && !prefix.ends_with('E')) =>
            {
                (take_while(2.., is_xid_continue)
                    .with_span()
                    .map(|(s, r): (&str, Range)| {
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
                            return TokenKind::unknown(
                                TokenKind::Number(0.0),
                                vec![TokenError::new(r, "Invalid base prefix for number literal")],
                            );
                        };

                        let s = &s[2..];
                        let mut errors = vec![];
                        let num = if s.chars().all(is_valid_char) {
                            if s.is_empty() {
                                errors.push(TokenError::new(
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
                                    errors.push(TokenError::new(
                                        range,
                                        "Invalid character in number literal",
                                    ));
                                }
                            }
                            if num.is_empty() {
                                errors.push(TokenError::new(
                                    r.clone(),
                                    "No valid digits for number literal",
                                ));
                            }
                            BigUint::parse_bytes(num.as_bytes(), radix)
                        }
                        .unwrap_or_default();
                        let float = num.to_f64().unwrap();
                        if ends_with_underscore {
                            errors.push(TokenError::new(
                                r.clone(),
                                "Number literal cannot end with underscore",
                            ));
                        }
                        if float.is_infinite() {
                            errors.push(TokenError::new(r, "Number literal is too large"));
                        }
                        if !errors.is_empty() {
                            TokenKind::unknown(TokenKind::Number(float), errors)
                        } else {
                            TokenKind::Number(float)
                        }
                    }))
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
            take_while(1.., |c: char| c == '$' || is_xid_continue(c))
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
    alt((
        dispatch! {peek(any);
            '0'..='9' => if prev_token.map(|t| &t.kind) == Some(&TokenKind::Operator(Operator::Dot)) {
                ordinal
            } else {
                number
            },
            '"' | '\'' | '`' => string::string,
            '+' => any.map(|_| TokenKind::Operator(Operator::Plus)),
            '-' => any.map(|_| TokenKind::Operator(Operator::Minus)),
            '^' => any.map(|_| TokenKind::Operator(Operator::Caret)),
            '*' => any.map(|_| TokenKind::Operator(Operator::Asterisk)),
            '%' => any.map(|_| TokenKind::Operator(Operator::Percent)),
            '=' => dispatch! {peek(opt(take(2usize)));
                Some("==") => take(2usize).map(|_| TokenKind::Operator(Operator::EqualEqual)),
                _ => any.map(|_| TokenKind::Operator(Operator::Equal)),
            },
            '!' => dispatch! {peek(opt(take(2usize)));
                Some("!=") => take(2usize).map(|_| TokenKind::Operator(Operator::NotEqual)),
                _ => fail,
            },
            '>' => dispatch! {peek(opt(take(2usize)));
                Some(">=") => take(2usize).map(|_| TokenKind::Operator(Operator::GreaterEqual)),
                _ => any.map(|_| TokenKind::Operator(Operator::Greater)),
            },
            '<' => dispatch! {peek(opt(take(2usize)));
                Some("<=") => take(2usize).map(|_| TokenKind::Operator(Operator::LessEqual)),
                _ => any.map(|_| TokenKind::Operator(Operator::Less)),
            },
            '(' => any.map(|_| TokenKind::Operator(Operator::OpenParen)),
            ')' => any.map(|_| TokenKind::Operator(Operator::CloseParen)),
            ':' => any.map(|_| TokenKind::Operator(Operator::Colon)),
            ',' => any.map(|_| TokenKind::Operator(Operator::Comma)),
            '.' => any.map(|_| TokenKind::Operator(Operator::Dot)),

            '/' => dispatch! {peek(opt(take(2usize)));
                Some("/*") => block_comment.map(|_| TokenKind::Whitespace(Whitespace::BlockComment)),
                Some("//") => line_comment.map(|_| TokenKind::Whitespace(Whitespace::LineComment)),
                _ => any.map(|_| TokenKind::Operator(Operator::Slash)),
            },
            ';' => any.map(|_| TokenKind::Operator(Operator::Semicolon)),
            '{' => any.map(|_| TokenKind::Operator(Operator::OpenBrace)),
            '}' => any.map(|_| TokenKind::Operator(Operator::CloseBrace)),
            c if c.is_ascii_whitespace() => trace("spaces", take_while(1.., |c: char| c.is_ascii_whitespace()).map(|_| TokenKind::Whitespace(Whitespace::Spaces))),
            c if is_xid_start(c) => identifier,
            _ => fail,
        },
        eof.map(|_|TokenKind::Eof),
        any.span().map(|range| TokenKind::Unknown {
            recovered: None,
            errors: vec![TokenError::new(range, "Unknown token")],
        }),
    ))
    .with_span()
    .map(|(kind, range)| Token { range, kind })
    .parse_next(input)
}

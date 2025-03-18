use core::num;
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::process::Command;

use unicode_ident::{is_xid_continue, is_xid_start};
use winnow::ascii::{line_ending, multispace1, till_line_ending};
use winnow::combinator::{cut_err, repeat_till, seq, trace};
use winnow::error::{AddContext, ErrMode, ParserError, StrContext, StrContextValue};
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{none_of, take, take_until, take_while};
use winnow::{LocatingSlice, Result};
use winnow::{
    ascii::{digit1 as digits, multispace0},
    combinator::alt,
    combinator::dispatch,
    combinator::eof,
    combinator::fail,
    combinator::opt,
    combinator::peek,
    combinator::repeat,
    combinator::{delimited, preceded, terminated},
    error::ContextError,
    stream::TokenSlice,
    token::any,
    token::literal,
    token::one_of,
    token::take_till,
};

pub(crate) type Input<'a> = LocatingSlice<&'a str>;
pub(crate) type Range = std::ops::Range<usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whitespace {
    LineComment,
    BlockComment,
    Spaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    OpenParen = '(' as isize,
    CloseParen = ')' as isize,
    Colon = ':' as isize,
    Comma = ',' as isize,

    Dot = '.' as isize,

    Plus = '+' as isize,
    Minus = '-' as isize,

    Caret = '^' as isize,

    Star = '*' as isize,
    Slash = '/' as isize,
    Percent = '%' as isize,

    Equal = '=' as isize,
    EqualEqual = (('=' as isize) << 8) + ('=' as isize),
    NotEqual = (('!' as isize) << 8) + ('=' as isize),
    Greater = ('>' as isize),
    GreaterEqual = ((('>' as isize) << 8) + ('=' as isize)),
    Less = ('<' as isize),
    LessEqual = ((('<' as isize) << 8) + ('=' as isize)),

    Semicolon = ';' as isize,
    OpenBrace = '{' as isize,
    CloseBrace = '}' as isize,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    and,
    or,
    not,

    r#if,
    r#else,
    r#match,
    r#for,
    r#in,
    r#while,
    r#loop,
    r#break,
    r#continue,
    r#return,

    r#fn,
    op,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenError<'a> {
    pub range: Range,
    pub error: Cow<'a, str>,
}

impl<'a> TokenError<'a> {
    pub fn new<E: Into<Cow<'a, str>>>(range: Range, error: E) -> Self {
        TokenError {
            range,
            error: error.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    Eof,
    Whitespace(Whitespace),
    Identifier(Cow<'a, str>),
    Ordinal(u64),
    Number(f64),
    String(Cow<'a, str>),
    Operator(Operator),
    Keyword(Keyword),

    Unknown {
        recovered: Option<Box<TokenKind<'a>>>,
        errors: Vec<TokenError<'a>>,
    },
}

impl<'a> TokenKind<'a> {
    pub(crate) fn unknown(recovered: TokenKind<'a>, errors: Vec<TokenError<'a>>) -> Self {
        TokenKind::Unknown {
            recovered: Some(Box::new(recovered)),
            errors,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub range: Range,
    pub kind: TokenKind<'a>,
}

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
    dispatch! {peek(opt(take(2usize)));
        Some("0x") => cut_err(take_while(1.., is_xid_continue).verify(|s: &str| s.chars().all(|c| c.is_ascii_hexdigit())))
            .context(StrContext::Label("digit"))
            .context(StrContext::Expected(StrContextValue::Description("hexadecimal"))),
        Some("0o") => cut_err(take_while(1.., is_xid_continue).verify(|s: &str| s.chars().all(|c| ('0'..='7').contains(&c))))
            .context(StrContext::Label("digit"))
            .context(StrContext::Expected(StrContextValue::Description("octal"))),
        Some("0b") => cut_err(take_while(1.., is_xid_continue).verify(|s: &str| s.chars().all(|c| c == '0' || c == '1')))
            .context(StrContext::Label("digit"))
            .context(StrContext::Expected(StrContextValue::Description("binary"))),
        _ => cut_err((
                 digits,
                 opt(('.', digits)),
                 opt((one_of(['e', 'E']), opt(one_of(['+', '-'])), digits)),
             )
            .take()),
    }
    .context(StrContext::Label("number"))
    .map(|s: &str| TokenKind::Number(s.parse().unwrap()))
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

pub(crate) fn tokenizer<'a>(
    input: &mut Input<'a>,
    prev_token: &Option<Token<'a>>,
) -> ModalResult<Token<'a>> {
    alt((
        dispatch! {peek(any);
            '0'..='9' => if prev_token.as_ref().map(|t| &t.kind) == Some(&TokenKind::Operator(Operator::Dot)) {
                ordinal
            } else {
                number
            },
            '"' | '\'' | '`' => crate::string_parser::string,
            '+' => any.map(|_| TokenKind::Operator(Operator::Plus)),
            '-' => any.map(|_| TokenKind::Operator(Operator::Minus)),
            '^' => any.map(|_| TokenKind::Operator(Operator::Caret)),
            '*' => any.map(|_| TokenKind::Operator(Operator::Star)),
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
            _ => any.span().map(|range| TokenKind::Unknown {
                recovered: None,
                errors: vec![TokenError::new(range, "Unknown token")],
            }),
        },
        eof.map(|_|TokenKind::Eof),
    ))
    .with_span()
    .map(|(kind, range)| Token { range, kind })
    .parse_next(input)
}

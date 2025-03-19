use std::borrow::Cow;

use winnow::{LocatingSlice, ModalResult};

mod display;
mod string;
mod tokens;

pub type Input<'a> = LocatingSlice<&'a str>;
pub type Range = std::ops::Range<usize>;

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

    Asterisk = '*' as isize,
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

impl<'a> Token<'a> {
    pub(crate) fn unknown(
        range: Range,
        recovered: TokenKind<'a>,
        errors: Vec<TokenError<'a>>,
    ) -> Self {
        Token {
            range,
            kind: TokenKind::unknown(recovered, errors),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub range: Range,
    pub kind: TokenKind<'a>,
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl PartialEq<Token<'_>> for TokenKind<'_> {
    fn eq(&self, other: &Token<'_>) -> bool {
        self == &other.kind
    }
}

impl PartialEq<TokenKind<'_>> for Token<'_> {
    fn eq(&self, other: &TokenKind<'_>) -> bool {
        self.kind == *other
    }
}

impl PartialEq<Token<'_>> for Operator {
    fn eq(&self, other: &Token<'_>) -> bool {
        matches!(other.kind, TokenKind::Operator(op) if op == *self)
    }
}

impl PartialEq<Operator> for Token<'_> {
    fn eq(&self, other: &Operator) -> bool {
        matches!(self.kind, TokenKind::Operator(op) if op == *other)
    }
}

impl PartialEq<Token<'_>> for Keyword {
    fn eq(&self, other: &Token<'_>) -> bool {
        matches!(other.kind, TokenKind::Keyword(kw) if kw == *self)
    }
}

impl PartialEq<Keyword> for Token<'_> {
    fn eq(&self, other: &Keyword) -> bool {
        matches!(self.kind, TokenKind::Keyword(kw) if kw == *other)
    }
}

pub fn to_input(text: &str) -> Input<'_> {
    LocatingSlice::new(text)
}

pub fn tokenize<'a>(
    input: &mut Input<'a>,
    ignore_whitespaces: bool,
) -> ModalResult<Vec<Token<'a>>> {
    let mut tokens = vec![];
    loop {
        let prev_token = &tokens.last();
        let token = tokens::token(input, prev_token)?;
        if ignore_whitespaces && matches!(token.kind, TokenKind::Whitespace(_)) {
            continue;
        }
        let eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if eof {
            break;
        }
    }
    Ok(tokens)
}

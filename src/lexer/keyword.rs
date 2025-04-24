use strum::{Display, EnumString, VariantNames};

use super::{Token, TokenKind};
use crate::ansi::{DisplayIdent, KEYWORD, NUMBER, RESET};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum Keyword {
    // constants
    True,
    False,
    Nil,
    Nan,
    Inf,

    // pseudo variable
    #[strum(to_string = "_")]
    Underscore,
    Global,

    // operators
    In,
    Is,
    And,
    Or,
    Not,

    // pseudo function
    Type,

    // control flow
    If,
    Else,
    Match,
    Case,
    For,
    While,
    Loop,
    Break,
    Continue,
    Return,

    // declaration
    Fn,
    Op, // Reserved for future use
    Let,
    Mut,
    // Type,  // Reserved for future use
    Where, // Reserved for future use

    // type Date(string) {
    //   if it |> matches(`(\d{4})-(\d{2})-(\d{2})`) { it }
    //   else if it |> matches(`(\d{4})/(\d{2})/(\d{2})`) { it |> replace("/", "-") }
    //   else { nil }
    // }
    // type Complex(number, number);
    // type Color(red: number, green: number, blue: number) { red > 0 && red < 256 && green > 0 && green < 256 && blue > 0 && blue < 256 }
    // Date("2021-01-01") -> "2021-01-01"
    // Complex(1, 2) -> { 0: 1, 1: 2 }
    // Color(1, 2, 3) -> { red: 1, green: 2, blue: 3 }
    // Color(-1, 2, 3) -> nil

    // module
    Import, // Reserved for future use
    Export, // Reserved for future use

    // algebraic effects
    Effect,  // Reserved for future use
    Try,     // Reserved for future use
    Handle,  // Reserved for future use
    Finally, // Reserved for future use
    Perform, // Reserved for future use
    Resume,  // Reserved for future use
}

impl From<Keyword> for TokenKind<'_> {
    fn from(kw: Keyword) -> Self {
        TokenKind::Keyword(kw)
    }
}

impl PartialEq<Token<'_>> for Keyword {
    fn eq(&self, other: &Token<'_>) -> bool {
        matches!(&other.kind, TokenKind::Keyword(kw) if kw == self)
    }
}

impl PartialEq<Keyword> for Token<'_> {
    fn eq(&self, other: &Keyword) -> bool {
        matches!(&self.kind, TokenKind::Keyword(kw) if kw == other)
    }
}

impl PartialEq<TokenKind<'_>> for Keyword {
    fn eq(&self, other: &TokenKind<'_>) -> bool {
        matches!(other, TokenKind::Keyword(kw) if kw == self)
    }
}

impl PartialEq<Keyword> for TokenKind<'_> {
    fn eq(&self, other: &Keyword) -> bool {
        matches!(self, TokenKind::Keyword(kw) if kw == other)
    }
}

impl DisplayIdent for Keyword {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, _ident: usize) -> std::fmt::Result {
        use Keyword::*;
        match self {
            Inf | Nan => write!(f, "{NUMBER}{self}{RESET}"),
            _ => write!(f, "{KEYWORD}{self}{RESET}"),
        }
    }
}

#[test]
fn test_keyword() {
    use std::str::FromStr;

    assert_eq!(Keyword::from_str("true"), Ok(Keyword::True));
    assert_eq!(Keyword::from_str("false"), Ok(Keyword::False));
    assert_eq!(Keyword::from_str("nil"), Ok(Keyword::Nil));
    assert_eq!(Keyword::from_str("nan"), Ok(Keyword::Nan));
    assert_eq!(Keyword::from_str("inf"), Ok(Keyword::Inf));

    assert_eq!(Keyword::from_str("_"), Ok(Keyword::Underscore));
    assert_eq!(Keyword::from_str("global"), Ok(Keyword::Global));

    assert_eq!(Keyword::from_str("in"), Ok(Keyword::In));
    assert_eq!(Keyword::from_str("is"), Ok(Keyword::Is));

    assert_eq!(
        Keyword::from_str("Is"),
        Err(strum::ParseError::VariantNotFound)
    );
    assert_eq!(
        Keyword::from_str("Underscore"),
        Err(strum::ParseError::VariantNotFound)
    );
    assert_eq!(
        Keyword::from_str("underscore"),
        Err(strum::ParseError::VariantNotFound)
    );
}

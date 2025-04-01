use std::{fmt::Display, str::FromStr};

use super::{Token, TokenKind};
use crate::ansi::{KEYWORD, RESET};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // constants
    True,
    False,
    Nil,
    Nan,
    Inf,

    // pseudo variable
    Underscore,
    Global,

    // control flow
    If,
    Else,
    Match,
    Case,
    For,
    In,
    While,
    Loop,
    Break,
    Continue,
    Return,

    // declaration
    Fn,
    Op, // Reserved for future use
    Var,
    Val,
    Record, // Reserved for future use

    // record Date(string) {
    //   if it |> matches(`(\d{4})-(\d{2})-(\d{2})`) { it }
    //   else if it |> matches(`(\d{4})/(\d{2})/(\d{2})`) { it |> replace("/", "-") }
    //   else { nil }
    // }
    // record Complex(number, number);
    // record Color(red: number, green: number, blue: number) { red > 0 && red < 256 && green > 0 && green < 256 && blue > 0 && blue < 256 }
    // Date("2021-01-01") -> "2021-01-01"
    // Complex(1, 2) -> { 0: 1, 1: 2 }
    // Color(1, 2, 3) -> { red: 1, green: 2, blue: 3 }
    // Color(-1, 2, 3) -> nil

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

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Keyword::Underscore {
            write!(f, "{KEYWORD}_{RESET}")
        } else {
            let mut d = format!("{:?}", self);
            d.make_ascii_lowercase();
            write!(f, "{KEYWORD}{d}{RESET}")
        }
    }
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(Keyword::True),
            "false" => Ok(Keyword::False),
            "nil" => Ok(Keyword::Nil),
            "nan" => Ok(Keyword::Nan),
            "inf" => Ok(Keyword::Inf),

            "_" => Ok(Keyword::Underscore),
            "global" => Ok(Keyword::Global),

            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "match" => Ok(Keyword::Match),
            "case" => Ok(Keyword::Case),
            "for" => Ok(Keyword::For),
            "in" => Ok(Keyword::In),
            "while" => Ok(Keyword::While),
            "loop" => Ok(Keyword::Loop),
            "break" => Ok(Keyword::Break),
            "continue" => Ok(Keyword::Continue),
            "return" => Ok(Keyword::Return),

            "fn" => Ok(Keyword::Fn),
            "op" => Ok(Keyword::Op),
            "var" => Ok(Keyword::Var),
            "val" => Ok(Keyword::Val),
            "record" => Ok(Keyword::Record),

            "effect" => Ok(Keyword::Effect),
            "try" => Ok(Keyword::Try),
            "handle" => Ok(Keyword::Handle),
            "finally" => Ok(Keyword::Finally),
            "perform" => Ok(Keyword::Perform),
            "resume" => Ok(Keyword::Resume),

            _ => Err(()),
        }
    }
}

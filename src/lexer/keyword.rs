use std::{fmt::Display, str::FromStr};

use super::{Token, TokenKind};
use crate::ansi::{KEYWORD, RESET};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // constants
    True,
    False,
    Nil,

    // logical operators
    And,
    Or,
    Not,

    // pseudo variable
    Underscore,
    Global,

    // control flow
    If,
    Else,
    Match,
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

    // algebraic effects
    Effect,  // Reserved for future use
    Try,     // Reserved for future use
    Handle,  // Reserved for future use
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
        let mut d = format!("{KEYWORD}{:?}{RESET}", self);
        d.make_ascii_lowercase();
        f.write_str(&d)
    }
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(Keyword::True),
            "false" => Ok(Keyword::False),
            "nil" => Ok(Keyword::Nil),

            "_" => Ok(Keyword::Underscore),
            "global" => Ok(Keyword::Global),

            "and" => Ok(Keyword::And),
            "or" => Ok(Keyword::Or),
            "not" => Ok(Keyword::Not),

            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "match" => Ok(Keyword::Match),
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
            "perform" => Ok(Keyword::Perform),
            "resume" => Ok(Keyword::Resume),

            _ => Err(()),
        }
    }
}

use strum::{Display, EnumProperty, EnumString, IntoStaticStr, VariantArray};

use super::prelude::*;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumString,
    Display,
    VariantArray,
    EnumProperty,
    IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Keyword {
    // constants
    #[strum(props(constant = true))]
    True,
    #[strum(props(constant = true))]
    False,
    #[strum(props(constant = true))]
    Nil,
    #[strum(props(constant = true, numeric = true))]
    Nan,
    #[strum(props(constant = true, numeric = true))]
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
    #[strum(props(control = true))]
    If,
    #[strum(props(control = true))]
    Else,
    #[strum(props(control = true))]
    Match,
    #[strum(props(control = true))]
    Case,
    #[strum(props(control = true))]
    For,
    #[strum(props(control = true))]
    While,
    #[strum(props(control = true))]
    Loop,
    #[strum(props(control = true))]
    Break,
    #[strum(props(control = true))]
    Continue,
    #[strum(props(control = true))]
    Return,

    // declaration
    Fn,
    #[strum(props(reserved = true))]
    Op,
    Let,
    Mut,
    #[strum(props(reserved = true))]
    Const,
    // #[strum(props(reserved = true))]
    // Type,
    #[strum(props(reserved = true))]
    Where,

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
    #[strum(props(control = true, reserved = true))]
    Import,
    #[strum(props(control = true, reserved = true))]
    Export,

    // algebraic effects
    #[strum(props(reserved = true))]
    Effect,
    #[strum(props(reserved = true))]
    Try,
    #[strum(props(reserved = true))]
    Handle,
    #[strum(props(reserved = true))]
    Finally,
    #[strum(props(reserved = true))]
    Perform,
    #[strum(props(reserved = true))]
    Resume,
}

impl Keyword {
    /// Returns `true` if the keyword is a control flow keyword.
    pub fn is_control(&self) -> bool {
        self.get_bool("control").unwrap_or(false)
    }

    /// Returns `true` if the keyword is a constant.
    pub fn is_constant(&self) -> bool {
        self.get_bool("constant").unwrap_or(false)
    }

    /// Returns `true` if the keyword is reserved and cannot be used as an identifier.
    pub fn is_reserved(&self) -> bool {
        self.get_bool("reserved").unwrap_or(false)
    }

    /// Returns `true` if the keyword is numeric.
    pub fn is_numeric(&self) -> bool {
        self.get_bool("numeric").unwrap_or(false)
    }
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

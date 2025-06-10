use std::fmt::Display;

use strum::{EnumProperty, VariantArray};

use super::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, EnumProperty)]
pub enum Operator {
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `:`
    Colon,
    /// `?:`
    QuestionColon,
    /// `::`
    ColonColon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `->`
    Arrow,

    /// `..`
    SpreadRange,
    /// `..<`
    HalfOpenRange,

    /// `+`
    Plus,
    /// `+=`
    PlusEqual,
    /// `-`
    Minus,
    /// `-=`
    MinusEqual,
    /// `*`
    Asterisk,
    /// `*=`
    AsteriskEqual,
    /// `/`
    Slash,
    /// `/=`
    SlashEqual,
    /// `%`
    Percent,
    /// `%=`
    PercentEqual,

    /// `^`
    Caret,
    /// `^=`
    CaretEqual,

    /// `!`
    Exclamation,
    /// `&&`
    LogicalAnd,
    /// `&&=`
    LogicalAndEqual,
    /// `||`
    LogicalOr,
    /// `||=`
    LogicalOrEqual,
    /// `??`
    NullCoalescing,
    /// `??=`
    NullCoalescingEqual,

    /// `=`
    Equal,
    /// `==`
    EqualEqual,
    /// `!=`
    NotEqual,
    /// `~=`
    TildeEqual,
    /// `!~=`
    NotTildeEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,

    /// `;`
    Semicolon,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
}

impl From<Operator> for TokenKind<'_> {
    fn from(op: Operator) -> Self {
        TokenKind::Operator(op)
    }
}

impl PartialEq<Token<'_>> for Operator {
    fn eq(&self, other: &Token<'_>) -> bool {
        matches!(&other.kind, TokenKind::Operator(op) if op == self)
    }
}

impl PartialEq<Operator> for Token<'_> {
    fn eq(&self, other: &Operator) -> bool {
        matches!(&self.kind, TokenKind::Operator(op) if op == other)
    }
}

impl PartialEq<TokenKind<'_>> for Operator {
    fn eq(&self, other: &TokenKind<'_>) -> bool {
        matches!(other, TokenKind::Operator(op) if op == self)
    }
}

impl PartialEq<Operator> for TokenKind<'_> {
    fn eq(&self, other: &Operator) -> bool {
        matches!(self, TokenKind::Operator(op) if op == other)
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operator::*;
        match self {
            OpenParen => f.write_str("("),
            CloseParen => f.write_str(")"),
            OpenBracket => f.write_str("["),
            CloseBracket => f.write_str("]"),
            Colon => f.write_str(":"),
            QuestionColon => f.write_str("?:"),
            ColonColon => f.write_str("::"),
            Comma => f.write_str(","),
            Dot => f.write_str("."),
            Arrow => f.write_str("->"),
            SpreadRange => f.write_str(".."),
            HalfOpenRange => f.write_str("..<"),
            Plus => f.write_str("+"),
            PlusEqual => f.write_str("+="),
            Minus => f.write_str("-"),
            MinusEqual => f.write_str("-="),
            Asterisk => f.write_str("*"),
            AsteriskEqual => f.write_str("*="),
            Slash => f.write_str("/"),
            SlashEqual => f.write_str("/="),
            Percent => f.write_str("%"),
            PercentEqual => f.write_str("%="),
            Caret => f.write_str("^"),
            CaretEqual => f.write_str("^="),
            Exclamation => f.write_str("!"),
            LogicalAnd => f.write_str("&&"),
            LogicalAndEqual => f.write_str("&&="),
            LogicalOr => f.write_str("||"),
            LogicalOrEqual => f.write_str("||="),
            NullCoalescing => f.write_str("??"),
            NullCoalescingEqual => f.write_str("??="),
            Equal => f.write_str("="),
            EqualEqual => f.write_str("=="),
            NotEqual => f.write_str("!="),
            TildeEqual => f.write_str("~="),
            NotTildeEqual => f.write_str("!~="),
            Greater => f.write_str(">"),
            GreaterEqual => f.write_str(">="),
            Less => f.write_str("<"),
            LessEqual => f.write_str("<="),
            Semicolon => f.write_str(";"),
            OpenBrace => f.write_str("{"),
            CloseBrace => f.write_str("}"),
        }
    }
}

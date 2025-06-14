use std::fmt::Display;

use strum::{EnumProperty, VariantArray};

use crate::emitter::OpCode;

use super::prelude::*;

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
    #[strum(props(arithmetic = true, infix = true, prefix = true))]
    Plus,
    /// `+=`
    #[strum(props(arithmetic = true, compound = true))]
    PlusEqual,
    /// `-`
    #[strum(props(arithmetic = true, infix = true, prefix = true))]
    Minus,
    /// `-=`
    #[strum(props(arithmetic = true, compound = true))]
    MinusEqual,
    /// `*`
    #[strum(props(arithmetic = true, infix = true))]
    Asterisk,
    /// `*=`
    #[strum(props(arithmetic = true, compound = true))]
    AsteriskEqual,
    /// `/`
    #[strum(props(arithmetic = true, infix = true))]
    Slash,
    /// `/=`
    #[strum(props(arithmetic = true, compound = true))]
    SlashEqual,
    /// `%`
    #[strum(props(arithmetic = true, infix = true))]
    Percent,
    /// `%=`
    #[strum(props(arithmetic = true, compound = true))]
    PercentEqual,
    /// `^`
    #[strum(props(arithmetic = true, infix = true))]
    Caret,
    /// `^=`
    #[strum(props(arithmetic = true, compound = true))]
    CaretEqual,

    /// `!`
    #[strum(props(logical = true, prefix = true, postfix = true))]
    Exclamation,
    /// `&&`
    #[strum(props(logical = true, infix = true))]
    LogicalAnd,
    /// `&&=`
    #[strum(props(logical = true, compound = true))]
    LogicalAndEqual,
    /// `||`
    #[strum(props(logical = true, infix = true))]
    LogicalOr,
    /// `||=`
    #[strum(props(logical = true, compound = true))]
    LogicalOrEqual,
    /// `??`
    #[strum(props(logical = true, infix = true))]
    NullCoalescing,
    /// `??=`
    #[strum(props(logical = true, compound = true))]
    NullCoalescingEqual,

    /// `=`
    Equal,

    /// `==`
    #[strum(props(relation = true, infix = true))]
    EqualEqual,
    /// `!=`
    #[strum(props(relation = true, infix = true))]
    NotEqual,
    /// `~=`
    #[strum(props(relation = true, infix = true))]
    TildeEqual,
    /// `!~=`
    #[strum(props(relation = true, infix = true))]
    NotTildeEqual,
    /// `>`
    #[strum(props(relation = true, infix = true))]
    Greater,
    /// `>=`
    #[strum(props(relation = true, infix = true))]
    GreaterEqual,
    /// `<`
    #[strum(props(relation = true, infix = true))]
    Less,
    /// `<=`
    #[strum(props(relation = true, infix = true))]
    LessEqual,

    /// `;`
    Semicolon,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
}

impl Operator {
    pub fn to_infix_op(&self) -> Option<OpCode> {
        use Operator::*;
        match self {
            Caret => Some(OpCode::Pow),

            Asterisk => Some(OpCode::Mul),
            Slash => Some(OpCode::Div),
            Percent => Some(OpCode::Mod),

            Plus => Some(OpCode::Add),
            Minus => Some(OpCode::Sub),

            Greater => Some(OpCode::Gt),
            GreaterEqual => Some(OpCode::Gte),
            Less => Some(OpCode::Lt),
            LessEqual => Some(OpCode::Lte),

            EqualEqual => Some(OpCode::Eq),
            NotEqual => Some(OpCode::Neq),
            TildeEqual => Some(OpCode::Aeq),
            NotTildeEqual => Some(OpCode::Naeq),

            _ => None,
        }
    }

    pub fn to_compound_op(&self) -> Option<OpCode> {
        use Operator::*;
        match self {
            CaretEqual => Some(OpCode::Pow),

            AsteriskEqual => Some(OpCode::Mul),
            SlashEqual => Some(OpCode::Div),
            PercentEqual => Some(OpCode::Mod),

            PlusEqual => Some(OpCode::Add),
            MinusEqual => Some(OpCode::Sub),

            _ => None,
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        self.get_bool("arithmetic").unwrap_or(false)
    }

    pub fn is_logical(&self) -> bool {
        self.get_bool("logical").unwrap_or(false)
    }

    pub fn is_relation(&self) -> bool {
        self.get_bool("relation").unwrap_or(false)
    }
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

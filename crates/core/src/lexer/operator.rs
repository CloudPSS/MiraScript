use std::fmt::Display;

use strum::{EnumProperty, VariantArray};

use crate::emitter::OpCode;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, EnumProperty)]
pub enum Operator {
    /// Unknown operator
    Unknown,
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
    /// `#`
    Format,

    /// `..`
    SpreadRange,
    /// `..<`
    HalfOpenRange,

    /// `+`
    #[strum(props(arithmetic = true, infix = true, prefix = true))]
    Plus,
    /// `+=`
    #[strum(props(arithmetic = true, compound = true))]
    PlusAssign,
    /// `-`
    #[strum(props(arithmetic = true, infix = true, prefix = true))]
    Minus,
    /// `-=`
    #[strum(props(arithmetic = true, compound = true))]
    MinusAssign,
    /// `*`
    #[strum(props(arithmetic = true, infix = true))]
    Asterisk,
    /// `*=`
    #[strum(props(arithmetic = true, compound = true))]
    AsteriskAssign,
    /// `/`
    #[strum(props(arithmetic = true, infix = true))]
    Slash,
    /// `/=`
    #[strum(props(arithmetic = true, compound = true))]
    SlashAssign,
    /// `%`
    #[strum(props(arithmetic = true, infix = true))]
    Percent,
    /// `%=`
    #[strum(props(arithmetic = true, compound = true))]
    PercentAssign,
    /// `^`
    #[strum(props(arithmetic = true, infix = true))]
    Caret,
    /// `^=`
    #[strum(props(arithmetic = true, compound = true))]
    CaretAssign,

    /// `!`
    #[strum(props(logical = true, prefix = true, postfix = true))]
    Exclamation,
    /// `&&`
    #[strum(props(logical = true, infix = true))]
    LogicalAnd,
    /// `&&=`
    #[strum(props(logical = true, compound = true))]
    LogicalAndAssign,
    /// `||`
    #[strum(props(logical = true, infix = true))]
    LogicalOr,
    /// `||=`
    #[strum(props(logical = true, compound = true))]
    LogicalOrAssign,
    /// `??`
    #[strum(props(null_coalescing = true, infix = true))]
    NullCoalescing,
    /// `??=`
    #[strum(props(null_coalescing = true, compound = true))]
    NullCoalescingAssign,

    /// `=`
    Assign,

    /// `==`
    #[strum(props(equality = true, infix = true))]
    Equal,
    /// `!=`
    #[strum(props(equality = true, infix = true))]
    NotEqual,
    /// `=~`
    #[strum(props(comparison = true, infix = true))]
    TildeEqual,
    /// `!~`
    #[strum(props(comparison = true, infix = true))]
    TildeNotEqual,
    /// `>`
    #[strum(props(comparison = true, infix = true))]
    Greater,
    /// `>=`
    #[strum(props(comparison = true, infix = true))]
    GreaterEqual,
    /// `<`
    #[strum(props(comparison = true, infix = true))]
    Less,
    /// `<=`
    #[strum(props(comparison = true, infix = true))]
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

            Equal => Some(OpCode::Eq),
            NotEqual => Some(OpCode::Neq),
            TildeEqual => Some(OpCode::Aeq),
            TildeNotEqual => Some(OpCode::Naeq),

            _ => None,
        }
    }

    pub fn to_compound_op(&self) -> Option<OpCode> {
        use Operator::*;
        match self {
            CaretAssign => Some(OpCode::Pow),

            AsteriskAssign => Some(OpCode::Mul),
            SlashAssign => Some(OpCode::Div),
            PercentAssign => Some(OpCode::Mod),

            PlusAssign => Some(OpCode::Add),
            MinusAssign => Some(OpCode::Sub),

            _ => None,
        }
    }

    pub(crate) fn is_arithmetic(&self) -> bool {
        self.get_bool("arithmetic").unwrap_or(false)
    }
    pub(crate) fn is_logical(&self) -> bool {
        self.get_bool("logical").unwrap_or(false)
    }
    pub(crate) fn is_equality(&self) -> bool {
        self.get_bool("equality").unwrap_or(false)
    }
    pub(crate) fn is_comparison(&self) -> bool {
        self.get_bool("comparison").unwrap_or(false)
    }
    pub(crate) fn is_relation(&self) -> bool {
        self.is_equality() || self.is_comparison()
    }

    pub(crate) fn is_infix(&self) -> bool {
        self.get_bool("infix").unwrap_or(false)
    }
    pub(crate) fn is_prefix(&self) -> bool {
        self.get_bool("prefix").unwrap_or(false)
    }
    pub(crate) fn is_postfix(&self) -> bool {
        self.get_bool("postfix").unwrap_or(false)
    }
    pub(crate) fn is_compound(&self) -> bool {
        self.get_bool("compound").unwrap_or(false)
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
            Unknown => Ok(()),
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
            Format => f.write_str("#"),
            SpreadRange => f.write_str(".."),
            HalfOpenRange => f.write_str("..<"),
            Plus => f.write_str("+"),
            PlusAssign => f.write_str("+="),
            Minus => f.write_str("-"),
            MinusAssign => f.write_str("-="),
            Asterisk => f.write_str("*"),
            AsteriskAssign => f.write_str("*="),
            Slash => f.write_str("/"),
            SlashAssign => f.write_str("/="),
            Percent => f.write_str("%"),
            PercentAssign => f.write_str("%="),
            Caret => f.write_str("^"),
            CaretAssign => f.write_str("^="),
            Exclamation => f.write_str("!"),
            LogicalAnd => f.write_str("&&"),
            LogicalAndAssign => f.write_str("&&="),
            LogicalOr => f.write_str("||"),
            LogicalOrAssign => f.write_str("||="),
            NullCoalescing => f.write_str("??"),
            NullCoalescingAssign => f.write_str("??="),
            Assign => f.write_str("="),
            Equal => f.write_str("=="),
            NotEqual => f.write_str("!="),
            TildeEqual => f.write_str("=~"),
            TildeNotEqual => f.write_str("!~"),
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

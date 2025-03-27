use std::fmt::{Display, Write};

use super::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    OpenParen = '(' as isize,
    CloseParen = ')' as isize,
    OpenBracket = '[' as isize,
    CloseBracket = ']' as isize,
    Colon = ':' as isize,
    Comma = ',' as isize,
    Dot = '.' as isize,

    Spread = (('.' as isize) << 16) + (('.' as isize) << 8) + ('.' as isize),

    InclusiveRange = (('.' as isize) << 8) + ('.' as isize),
    LeftExclusiveRange = (('>' as isize) << 16) + (('.' as isize) << 8) + ('.' as isize),
    RightExclusiveRange = (('.' as isize) << 16) + (('.' as isize) << 8) + ('<' as isize),
    ExclusiveRange =
        (('>' as isize) << 24) + (('.' as isize) << 16) + (('.' as isize) << 8) + ('<' as isize),

    Plus = '+' as isize,
    Minus = '-' as isize,

    Caret = '^' as isize,

    Asterisk = '*' as isize,
    Slash = '/' as isize,
    Percent = '%' as isize,

    LogicalNot = '!' as isize,
    LogicalAnd = ((('&' as isize) << 8) + ('&' as isize)),
    LogicalOr = ((('|' as isize) << 8) + ('|' as isize)),

    ForwardPipe = ((('|' as isize) << 8) + ('>' as isize)),
    BackwardPipe = ((('<' as isize) << 8) + ('|' as isize)),

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
        let v = *self as u32;
        if v <= 0xFF {
            let c = v as u8 as char;
            return f.write_char(c);
        }
        if v <= 0xFFFF {
            let c1 = (v >> 8) as u8 as char;
            let c2 = v as u8 as char;
            return write!(f, "{}{}", c1, c2);
        }
        if v <= 0xFFFFFF {
            let c1 = (v >> 16) as u8 as char;
            let c2 = (v >> 8) as u8 as char;
            let c3 = v as u8 as char;
            return write!(f, "{}{}{}", c1, c2, c3);
        }
        let c1 = (v >> 24) as u8 as char;
        let c2 = (v >> 16) as u8 as char;
        let c3 = (v >> 8) as u8 as char;
        let c4 = v as u8 as char;
        write!(f, "{}{}{}{}", c1, c2, c3, c4)
    }
}

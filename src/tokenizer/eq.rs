use std::mem;

use super::{Keyword, Operator, Token, TokenKind};

impl Eq for TokenKind<'_> {}

impl PartialEq for TokenKind<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Whitespace(l0), Self::Whitespace(r0)) => l0 == r0,
            (Self::Identifier(l0), Self::Identifier(r0)) => l0 == r0,
            (Self::Ordinal(l0), Self::Ordinal(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => unsafe {
                mem::transmute::<f64, u64>(*l0) == mem::transmute::<f64, u64>(*r0)
            },
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Operator(l0), Self::Operator(r0)) => l0 == r0,
            (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (
                Self::Unknown {
                    recovered: l_recovered,
                    ..
                },
                Self::Unknown {
                    recovered: r_recovered,
                    ..
                },
            ) => l_recovered == r_recovered,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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

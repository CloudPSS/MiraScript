use std::{borrow::Cow, fmt::Display};

use crate::ansi::{INTERPOLATED, RECOVER, RESET, STRING};
use crate::parser::Expression;
use crate::utils::{SourceError, SourceRange};

use super::{Comment, Keyword, Operator, Token};

#[derive(Debug, Clone)]
pub enum TokenKind<'a> {
    Eof,
    Comment(Comment),
    Identifier(Cow<'a, str>),
    Ordinal(u64),
    Number(f64),
    String(Cow<'a, str>),
    InterpolatedString(Vec<Cow<'a, str>>, Vec<Expression<'a>>),
    Operator(Operator),
    Keyword(Keyword),
    Unknown {
        recovered: Option<Box<TokenKind<'a>>>,
        errors: Vec<SourceError>,
    },
}

impl<'a> TokenKind<'a> {
    pub(crate) fn unknown_range<E: Into<Cow<'static, str>>, R: Into<TokenKind<'a>>>(
        recovered: R,
        error_range: SourceRange,
        error: E,
    ) -> Self {
        TokenKind::Unknown {
            recovered: Some(Box::new(recovered.into())),
            errors: vec![SourceError::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<E: Into<Vec<SourceError>>, R: Into<TokenKind<'a>>>(
        recovered: R,
        errors: E,
    ) -> Self {
        TokenKind::Unknown {
            recovered: Some(Box::new(recovered.into())),
            errors: errors.into(),
        }
    }
}

impl Eq for TokenKind<'_> {}

impl PartialEq for TokenKind<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Comment(l0), Self::Comment(r0)) => l0 == r0,
            (Self::Identifier(l0), Self::Identifier(r0)) => l0 == r0,
            (Self::Ordinal(l0), Self::Ordinal(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => (*l0).to_bits() == (*r0).to_bits(),
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::InterpolatedString(l0, l1), Self::InterpolatedString(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
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

impl PartialEq<Token<'_>> for TokenKind<'_> {
    fn eq(&self, other: &Token<'_>) -> bool {
        self == &other.kind
    }
}

impl PartialEq<TokenKind<'_>> for Token<'_> {
    fn eq(&self, other: &TokenKind<'_>) -> bool {
        &self.kind == other
    }
}

impl Display for TokenKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "␀"),
            Self::Comment(Comment::Line) => writeln!(f, " //"),
            Self::Comment(Comment::Block) => write!(f, " /* */ "),
            Self::Identifier(s) => write!(f, "{}", s),
            Self::Ordinal(n) => write!(f, "{}", n),
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => {
                write!(f, "{STRING}\"{}\"{RESET}", s.escape_debug())
            }
            Self::InterpolatedString(s, e) => {
                write!(f, "{STRING}\"")?;
                assert_eq!(s.len(), e.len() + 1, "Invalid string interpolation");
                let mut s_iter = s.iter();
                let first = s_iter.next().ok_or(std::fmt::Error)?;
                write!(f, "{}", first.escape_debug())?;
                for (s, e) in s_iter.zip(e.iter()) {
                    write!(
                        f,
                        "{RESET}{INTERPOLATED}${{{RESET}{}{INTERPOLATED}}}{RESET}{STRING}",
                        e
                    )?;
                    write!(f, "{}", s.escape_debug())?;
                }
                write!(f, "\"{RESET}")
            }
            Self::Operator(op) => write!(f, "{}", op),
            Self::Keyword(kw) => write!(f, "{}", kw),
            Self::Unknown { recovered, .. } => {
                if let Some(recovered) = recovered {
                    write!(f, "{RECOVER}{}{RESET}", recovered)
                } else {
                    write!(f, "{RECOVER}<?>{RESET}")
                }
            }
        }
    }
}

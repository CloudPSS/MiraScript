use std::{borrow::Cow, fmt::Display};

use crate::ansi::{DisplayIdent, INTERPOLATED, NUMBER, ORDINAL, RECOVER, RESET, STRING, VARIABLE};

use super::prelude::*;

#[derive(Debug, Clone, strum::EnumIs)]
pub enum TokenKind<'s> {
    Eof,
    Identifier(&'s str),
    Ordinal(i32),
    Number(f64),
    String(Cow<'s, str>),
    InterpolatedString(Vec<Cow<'s, str>>, Vec<Vec<Token<'s>>>),
    Operator(Operator),
    Keyword(Keyword, Option<&'s str>),
    Unknown {
        recovered: Option<Box<TokenKind<'s>>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s> TokenKind<'s> {
    pub(crate) fn unknown(error_range: SourceRange, error: DiagnosticCode) -> Self {
        TokenKind::Unknown {
            recovered: None,
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_range<R: Into<TokenKind<'s>>>(
        recovered: R,
        error_range: SourceRange,
        error: DiagnosticCode,
    ) -> Self {
        TokenKind::Unknown {
            recovered: Some(Box::new(recovered.into())),
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<E: Into<Vec<SourceDiagnostic>>, R: Into<TokenKind<'s>>>(
        recovered: R,
        errors: E,
    ) -> Self {
        TokenKind::Unknown {
            recovered: Some(Box::new(recovered.into())),
            errors: errors.into(),
        }
    }

    pub(crate) fn to_field_name(&'s self) -> Option<(DiagnosticCode, Cow<'s, str>)> {
        match self {
            Self::Keyword(_, Some(name)) | Self::Identifier(name) => Some((
                DiagnosticCode::RecordFieldIdName,
                std::borrow::Cow::Borrowed(name),
            )),
            Self::Keyword(kw, None) => Some((
                DiagnosticCode::RecordFieldIdName,
                std::borrow::Cow::Owned(kw.to_string()),
            )),
            Self::Ordinal(n) => {
                Some((DiagnosticCode::RecordFieldOrdinalName, n.to_string().into()))
            }
            Self::String(s) => Some((DiagnosticCode::RecordFieldStringName, Cow::Borrowed(s))),
            _ => None,
        }
    }

    pub(crate) fn to_id_name(&'s self) -> Option<&'s str> {
        match self {
            Self::Identifier(name) => Some(name.as_ref()),
            _ => None,
        }
    }
}

impl PartialEq for TokenKind<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(l0), Self::Identifier(r0)) => l0 == r0,
            (Self::Ordinal(l0), Self::Ordinal(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => (*l0).to_bits() == (*r0).to_bits(),
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::InterpolatedString(l0, l1), Self::InterpolatedString(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::Operator(l0), Self::Operator(r0)) => l0 == r0,
            (Self::Keyword(l0, _), Self::Keyword(r0, _)) => l0 == r0,
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
            Self::Eof => write!(f, "␃"),
            Self::Identifier(s) => write!(f, "{s}"),
            Self::Ordinal(n) => write!(f, "{n}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => {
                write!(f, "\"{}\"", s.escape_debug())
            }
            Self::InterpolatedString(s, e) => {
                write!(f, "\"")?;
                assert_eq!(s.len(), e.len() + 1, "Invalid string interpolation");
                let mut s_iter = s.iter();
                let first = s_iter.next().ok_or(std::fmt::Error)?;
                write!(f, "{}", first.escape_debug())?;
                for (s, e) in s_iter.zip(e.iter()) {
                    write!(f, "$")?;
                    for token in e {
                        write!(f, "{token}")?;
                    }
                    write!(f, "{}", s.escape_debug())?;
                }
                write!(f, "\"")
            }
            Self::Operator(op) => write!(f, "{}", op),
            Self::Keyword(kw, _) => write!(f, "{}", kw),
            Self::Unknown { recovered, .. } => {
                if let Some(recovered) = recovered {
                    write!(f, "{recovered}")
                } else {
                    write!(f, "<?>")
                }
            }
        }
    }
}

impl DisplayIdent for TokenKind<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "␃"),
            Self::Identifier(s) => write!(f, "{VARIABLE}{s}{RESET}"),
            Self::Ordinal(n) => write!(f, "{ORDINAL}{n}{RESET}"),
            Self::Number(n) => write!(f, "{NUMBER}{n}{RESET}"),
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
                    write!(f, "{RESET}{INTERPOLATED}${RESET}")?;
                    for token in e {
                        write!(f, "{token}")?;
                    }
                    write!(f, "{STRING}")?;
                    write!(f, "{}", s.escape_debug())?;
                }
                write!(f, "\"{RESET}")
            }
            Self::Operator(op) => write!(f, "{}", op),
            Self::Keyword(kw, _) => kw.fmt_ident(f, ident),
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

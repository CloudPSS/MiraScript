use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use crate::{
    ansi::{RECOVER, RESET},
    lexer::Token,
    utils::{SourceError, SourceRange},
};

use super::display_ident::DisplayIdent;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<'a> {
    /// literal
    ///
    /// Matches against a literal value.
    Literal(Box<Token<'a>>),
    /// `_`
    ///
    /// Matches and discards a value.
    Discard(Box<Token<'a>>),
    /// `mut`? identifier
    ///
    /// Matches and binds a value to a variable.
    Bind(Option<Box<Token<'a>>>, Box<Token<'a>>),

    /// Unknown pattern.
    Unknown {
        tokens: Vec<Token<'a>>,
        errors: Vec<SourceError>,
    },
}

impl<'a> Pattern<'a> {
    pub(crate) fn is_unknown(&self) -> bool {
        matches!(self, Pattern::Unknown { .. })
    }

    pub(crate) fn unknown<T: Into<Vec<Token<'a>>>, E: Into<Cow<'static, str>>>(
        tokens: T,
        error: E,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }
    pub(crate) fn unknown_range<T: Into<Vec<Token<'a>>>, E: Into<Cow<'static, str>>>(
        tokens: T,
        error_range: SourceRange,
        error: E,
    ) -> Self {
        Pattern::Unknown {
            tokens: tokens.into(),
            errors: vec![SourceError::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'a>>>, E: Into<Vec<SourceError>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Pattern::Unknown {
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl Display for Pattern<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Pattern<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, _level: usize) -> fmt::Result {
        use Pattern::*;
        match self {
            Literal(token) => write!(f, "{token}"),
            Discard(token) => write!(f, "{token}"),
            Bind(None, token) => write!(f, "{token}"),
            Bind(Some(kw_mut), token) => write!(f, "{kw_mut} {token}"),
            Unknown { tokens, .. } => {
                write!(f, "{RECOVER}<pattern{RESET}")?;
                for token in tokens {
                    write!(f, " {token}")?;
                }
                write!(f, "{RECOVER}>{RESET}")
            }
        }
    }
}

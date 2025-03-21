use std::{borrow::Cow, fmt::Display};

use crate::utils::{Range, SourceError};

use super::TokenKind;

#[derive(Debug, Clone, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub range: Range,
}

impl<'a> Token<'a> {
    pub(crate) fn unknown<E: Into<Cow<'static, str>>>(
        range: Range,
        recovered: TokenKind<'a>,
        error: E,
    ) -> Self {
        Token {
            range: range.clone(),
            kind: TokenKind::unknown_range(recovered, range, error),
        }
    }
    pub(crate) fn unknown_range<E: Into<Cow<'static, str>>>(
        token_range: Range,
        recovered: TokenKind<'a>,
        error_range: Range,
        error: E,
    ) -> Self {
        Token {
            range: token_range,
            kind: TokenKind::unknown_range(recovered, error_range, error),
        }
    }

    pub(crate) fn unknown_errors<E: Into<Vec<SourceError>>>(
        range: Range,
        recovered: TokenKind<'a>,
        errors: E,
    ) -> Self {
        Token {
            range,
            kind: TokenKind::unknown_errors(recovered, errors),
        }
    }
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <TokenKind as Display>::fmt(&self.kind, f)
    }
}

use std::{borrow::Cow, fmt::Display};

use winnow::stream::Location;

use crate::utils::{SourceError, SourceRange};

use super::TokenKind;

#[derive(Debug, Clone, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub range: SourceRange,
}

impl Location for Token<'_> {
    fn current_token_start(&self) -> usize {
        self.range.start
    }

    fn previous_token_end(&self) -> usize {
        self.range.end
    }
}

impl<'a> Token<'a> {
    pub(crate) fn unknown<E: Into<Cow<'static, str>>, R: Into<TokenKind<'a>>>(
        range: SourceRange,
        recovered: R,
        error: E,
    ) -> Self {
        Token {
            range: range.clone(),
            kind: TokenKind::unknown_range(recovered, range, error),
        }
    }
    pub(crate) fn unknown_range<E: Into<Cow<'static, str>>, R: Into<TokenKind<'a>>>(
        token_range: SourceRange,
        recovered: R,
        error_range: SourceRange,
        error: E,
    ) -> Self {
        Token {
            range: token_range,
            kind: TokenKind::unknown_range(recovered, error_range, error),
        }
    }

    pub(crate) fn unknown_errors<E: Into<Vec<SourceError>>, R: Into<TokenKind<'a>>>(
        range: SourceRange,
        recovered: R,
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

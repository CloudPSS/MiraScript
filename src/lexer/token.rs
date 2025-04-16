use std::{borrow::Cow, fmt::Display};

use winnow::stream::Location;

use crate::{
    ansi::DisplayIdent,
    utils::{SourceError, SourceRange},
};

use super::{TokenKind, Trivia};

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub range: SourceRange,
    pub leading_trivia: Vec<Trivia<'a>>,
    pub trailing_trivia: Vec<Trivia<'a>>,
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
    pub(crate) fn is_unknown(&self) -> bool {
        matches!(self.kind, TokenKind::Unknown { .. })
    }

    pub(crate) fn wrap_as_unknown<E: Into<Cow<'static, str>>>(self, error: E) -> Self {
        Token {
            kind: TokenKind::unknown_range(self.kind, self.range.clone(), error),
            range: self.range,
            leading_trivia: self.leading_trivia,
            trailing_trivia: self.trailing_trivia,
        }
    }

    pub(crate) fn unknown<E: Into<Cow<'static, str>>, R: Into<TokenKind<'a>>>(
        range: SourceRange,
        recovered: R,
        error: E,
    ) -> Self {
        Token {
            range: range.clone(),
            kind: TokenKind::unknown_range(recovered, range, error),
            leading_trivia: vec![],
            trailing_trivia: vec![],
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
            leading_trivia: vec![],
            trailing_trivia: vec![],
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
            leading_trivia: vec![],
            trailing_trivia: vec![],
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
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Token<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        for trivia in &self.leading_trivia {
            trivia.fmt_ident(f, ident)?;
        }
        <TokenKind as Display>::fmt(&self.kind, f)?;
        for trivia in &self.trailing_trivia {
            trivia.fmt_ident(f, ident)?;
        }
        Ok(())
    }
}

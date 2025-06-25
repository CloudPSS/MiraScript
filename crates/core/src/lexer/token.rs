use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::ansi::DisplayIdent;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Token<'s> {
    pub kind: TokenKind<'s>,
    pub range: SourceRange,
    #[cfg(feature = "trivia")]
    pub leading_trivia: Box<[Trivia<'s>]>,
    #[cfg(feature = "trivia")]
    pub trailing_trivia: Box<[Trivia<'s>]>,
}

impl winnow::stream::Location for Token<'_> {
    fn current_token_start(&self) -> usize {
        self.range.start
    }

    fn previous_token_end(&self) -> usize {
        self.range.end
    }
}

impl<'s> Deref for Token<'s> {
    type Target = TokenKind<'s>;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl DerefMut for Token<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl<'s> Token<'s> {
    pub(crate) fn new(kind: TokenKind<'s>, range: SourceRange) -> Self {
        Token {
            kind,
            range,
            #[cfg(feature = "trivia")]
            leading_trivia: Box::new([]),
            #[cfg(feature = "trivia")]
            trailing_trivia: Box::new([]),
        }
    }

    pub(crate) fn wrap_as_unknown(self, error: DiagnosticCode) -> Self {
        Token {
            kind: TokenKind::unknown_range(self.kind, self.range.clone(), error),
            range: self.range,
            #[cfg(feature = "trivia")]
            leading_trivia: self.leading_trivia,
            #[cfg(feature = "trivia")]
            trailing_trivia: self.trailing_trivia,
        }
    }

    pub(crate) fn empty(pos: usize) -> Self {
        Self::new(TokenKind::Empty, pos..pos)
    }

    pub(crate) fn unknown<R: Into<TokenKind<'s>>>(
        range: SourceRange,
        recovered: R,
        error: DiagnosticCode,
    ) -> Self {
        Self::new(
            TokenKind::unknown_range(recovered, range.clone(), error),
            range,
        )
    }
    pub(crate) fn unknown_range<R: Into<TokenKind<'s>>>(
        token_range: SourceRange,
        recovered: R,
        error_range: SourceRange,
        error: DiagnosticCode,
    ) -> Self {
        Self::new(
            TokenKind::unknown_range(recovered, error_range, error),
            token_range,
        )
    }

    pub(crate) fn unknown_errors<E: Into<Vec<SourceDiagnostic>>, R: Into<TokenKind<'s>>>(
        range: SourceRange,
        recovered: R,
        errors: E,
    ) -> Self {
        Self::new(TokenKind::unknown_errors(recovered, errors), range)
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
        #[cfg(feature = "trivia")]
        for trivia in &self.leading_trivia {
            trivia.fmt_ident(f, ident)?;
        }
        self.kind.fmt_ident(f, ident)?;
        #[cfg(feature = "trivia")]
        for trivia in &self.trailing_trivia {
            trivia.fmt_ident(f, ident)?;
        }
        Ok(())
    }
}

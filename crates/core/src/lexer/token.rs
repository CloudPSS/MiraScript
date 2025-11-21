use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Token<'s> {
    pub kind: TokenKind<'s>,
    pub range: SourceRange,
    pub leading_trivia: TriviaList<'s>,
    pub tailing_trivia: TriviaList<'s>,
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
            leading_trivia: Default::default(),
            tailing_trivia: Default::default(),
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
        Self::new(TokenKind::unknown(recovered, range.clone(), error), range)
    }

    pub(crate) fn wrap_as_unknown(&mut self, error: DiagnosticCode) {
        let kind = std::mem::replace(&mut self.kind, TokenKind::Empty);
        self.kind = TokenKind::unknown(kind, self.range.clone(), error);
    }
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

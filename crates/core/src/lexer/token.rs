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
    pub(crate) fn empty_range(range: SourceRange) -> Self {
        Self::new(TokenKind::Empty, range)
    }

    pub(crate) fn unknown<R: Into<TokenKind<'s>>>(
        range: SourceRange,
        recovered: R,
        error: DiagnosticCode,
    ) -> Self {
        Self::new(TokenKind::unknown(recovered, range.clone(), error), range)
    }

    pub(crate) fn unknown_at<R: Into<TokenKind<'s>>>(
        pos: usize,
        recovered: R,
        error: DiagnosticCode,
    ) -> Self {
        Self::unknown(pos..pos, recovered, error)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenId(pub usize);

#[derive(Debug, Clone, Default)]
pub struct TokenArena<'s> {
    tokens: Vec<Token<'s>>,
}

impl<'s> TokenArena<'s> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tokens: Vec::with_capacity(capacity),
        }
    }

    pub fn from_tokens(tokens: Vec<Token<'s>>) -> Self {
        Self { tokens }
    }

    pub fn alloc(&mut self, token: Token<'s>) -> TokenId {
        let id = TokenId(self.tokens.len());
        self.tokens.push(token);
        id
    }

    pub fn get(&self, id: TokenId) -> Option<&Token<'s>> {
        self.tokens.get(id.0)
    }

    pub fn get_mut(&mut self, id: TokenId) -> Option<&mut Token<'s>> {
        self.tokens.get_mut(id.0)
    }

    pub fn as_slice(&self) -> &[Token<'s>] {
        &self.tokens
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

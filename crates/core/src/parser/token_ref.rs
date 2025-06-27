use std::{fmt::Display, ops::Deref};

use super::prelude::*;

#[derive(Debug)]
pub enum TokenRef<'s> {
    /// A reference to a token that is owned by the parser.
    Owned(Box<Token<'s>>),
    /// A reference to a token that is borrowed from the input.
    Borrowed(&'s Token<'s>),
}

impl<'s> TokenRef<'s> {
    pub(crate) fn new(token: Token<'s>) -> Self {
        Self::Owned(Box::new(token))
    }

    pub(crate) fn borrow(token: &'s Token<'s>) -> Self {
        Self::Borrowed(token)
    }

    pub(crate) fn wrap_as_unknown(self, error: DiagnosticCode) -> Self {
        match self {
            Self::Owned(token) => Self::new(token.wrap_as_unknown(error)),
            Self::Borrowed(token) => Self::new(token.clone().wrap_as_unknown(error)),
        }
    }
}

impl Display for TokenRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'s> Clone for TokenRef<'s> {
    fn clone(&self) -> Self {
        match self {
            // Owned 的生存期不一定长于 's，所以不能直接转为 Borrowed
            Self::Owned(token) => Self::Owned(token.clone()),
            Self::Borrowed(token) => Self::Borrowed(token),
        }
    }
}

impl<'s> From<Token<'s>> for TokenRef<'s> {
    fn from(token: Token<'s>) -> Self {
        Self::new(token)
    }
}

impl<'s> From<&'s Token<'s>> for TokenRef<'s> {
    fn from(token: &'s Token<'s>) -> Self {
        Self::borrow(token)
    }
}

impl<'s> PartialEq for TokenRef<'s> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<'s> Deref for TokenRef<'s> {
    type Target = Token<'s>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(token) => token,
            Self::Borrowed(token) => token,
        }
    }
}

impl<'s> AsRef<Token<'s>> for TokenRef<'s> {
    fn as_ref(&self) -> &Token<'s> {
        self.deref()
    }
}

impl<'s> AstWalker<'s> for TokenRef<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        if let Self::Owned(token) = self {
            token.collect_diagnostics(collector);
        }
    }

    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        self.as_ref().walk(visitor);
    }

    fn range(&self) -> SourceRange {
        self.as_ref().range()
    }
}

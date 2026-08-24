use std::{fmt::Display, ops::Deref, ptr};

use super::prelude::*;

#[derive(Debug)]
pub(crate) struct TokenRef<'s>(*const Token<'s>);

const _: () = assert!(std::mem::align_of::<Token<'_>>() > 1);

const OWNED: usize = 0x1;
const BORROWED: usize = 0x0;
const FLAG_MASK: usize = OWNED;

impl<'s> TokenRef<'s> {
    pub(crate) fn new(token: Token<'s>) -> Self {
        let boxed = Box::new(token);
        let ptr = Box::into_raw(boxed).map_addr(|addr| addr | OWNED);
        Self(ptr)
    }

    pub(crate) fn borrow(token: &'s Token<'s>) -> Self {
        Self(ptr::from_ref(token).map_addr(|addr| addr | BORROWED))
    }

    pub(crate) fn is_owned(&self) -> bool {
        (self.0.addr() & FLAG_MASK) == OWNED
    }

    pub(crate) fn is_borrowed(&self) -> bool {
        (self.0.addr() & FLAG_MASK) == BORROWED
    }

    fn ptr(&self) -> *const Token<'s> {
        self.0.map_addr(|addr| addr & !FLAG_MASK)
    }

    fn inner<'t>(&'t self) -> &'s Token<'s>
    where
        's: 't,
    {
        // SAFETY: The inner pointer is constructed in a way that it is guaranteed to be non-null and properly aligned. The FLAG_MASK is used to store ownership information in the least significant bit of the pointer, which does not affect the validity of the pointer itself.
        unsafe {
            self.ptr()
                .as_ref()
                .expect("Inner token pointer should not be null")
        }
    }

    fn inner_borrowed(&self) -> Option<&'s Token<'s>> {
        if !self.is_borrowed() {
            return None;
        }
        // SAFETY: Only borrowed tokens can be accessed, and the inner pointer is guaranteed to be valid and properly aligned.
        unsafe { self.ptr().as_ref() }
    }

    fn inner_owned(&mut self) -> Option<&mut Token<'s>> {
        if !self.is_owned() {
            return None;
        }
        // SAFETY: Only owned tokens can be mutated, and the inner pointer is guaranteed to be valid and properly aligned.
        unsafe { self.ptr().cast_mut().as_mut() }
    }

    pub(crate) fn wrap_as_unknown(&mut self, error: DiagnosticCode) {
        if let Some(inner) = self.inner_owned() {
            inner.wrap_as_unknown(error);
        } else {
            let mut cloned = self.inner().clone();
            cloned.wrap_as_unknown(error);
            *self = Self::new(cloned);
        }
    }
}

impl Display for TokenRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl Drop for TokenRef<'_> {
    fn drop(&mut self) {
        if self.is_owned() {
            // SAFETY: The inner pointer is guaranteed to be valid and properly aligned, and it was allocated with Box::new, so it can be safely converted back to a Box and dropped.
            std::mem::drop(unsafe { Box::from_raw(self.ptr().cast_mut()) });
        }
    }
}

impl<'s> Clone for TokenRef<'s> {
    fn clone(&self) -> Self {
        if let Some(inner) = self.inner_borrowed() {
            Self::borrow(inner)
        } else {
            // Owned 的生存期为 'self，不一定长于 's，所以不能直接转为 Borrowed
            let cloned = self.inner().clone();
            Self::new(cloned)
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
        self.inner()
    }
}

impl<'s> AsRef<Token<'s>> for TokenRef<'s> {
    fn as_ref(&self) -> &Token<'s> {
        self.deref()
    }
}

impl<'s> AstWalker<'s> for TokenRef<'s> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        if let Some(token) = self.inner_owned() {
            token.collect_diagnostics(collector);
        }
    }
    fn range(&self) -> SourceRange {
        self.as_ref().range()
    }
}

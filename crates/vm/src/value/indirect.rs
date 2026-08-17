use std::fmt;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use indexmap::IndexMap;

/// A shared indirection used by the non-scalar [`MiraAny`](super::MiraAny) variants.
///
/// This type is public only because it appears in the fields of the public
/// `MiraAny` enum. Its representation is intentionally opaque.
#[doc(hidden)]
#[repr(transparent)]
pub struct MiraIndirect<T>(Rc<T>);

impl<T> MiraIndirect<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Rc::new(value))
    }

    pub(crate) fn into_inner(self) -> T
    where
        T: Clone,
    {
        Rc::unwrap_or_clone(self.0)
    }
}

impl<T> Clone for MiraIndirect<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Deref for MiraIndirect<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> DerefMut for MiraIndirect<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::make_mut(&mut self.0)
    }
}

impl<T> AsRef<T> for MiraIndirect<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> From<T> for MiraIndirect<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl From<&str> for MiraIndirect<String> {
    fn from(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

impl<T: fmt::Debug> fmt::Debug for MiraIndirect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T: PartialEq> PartialEq for MiraIndirect<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<T: Eq> Eq for MiraIndirect<T> {}

impl<T> FromIterator<T> for MiraIndirect<Vec<T>> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<K, V> FromIterator<(K, V)> for MiraIndirect<IndexMap<K, V>>
where
    K: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

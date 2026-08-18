use std::any::type_name;

use crate::{MiraError, Result};

use super::{MiraAny, MiraShared};

/// A live Rust value that appears as a read-only MiraScript record.
pub trait MiraRecord: 'static {
    /// Return the record's visible field names in iteration order.
    fn keys(&self) -> Vec<String>;

    /// Read a field, returning `None` when the field is absent.
    fn get(&self, key: &str) -> Result<Option<MiraAny>>;
}

/// A live Rust value that appears as a read-only MiraScript array.
pub trait MiraArray: 'static {
    /// Return the current array length.
    fn len(&self) -> usize;

    /// Read an element, returning `None` when the index is out of bounds.
    fn get(&self, index: usize) -> Result<Option<MiraAny>>;

    /// Return whether this array currently contains no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
#[doc(hidden)]
pub trait MiraBridge: Sized + 'static {
    fn into_mira_shared(value: MiraShared<Self>) -> MiraAny;
}

#[doc(hidden)]
pub trait RecordObject {
    fn identity(&self) -> usize;
    fn tag(&self) -> &'static str;
    fn keys(&self) -> Result<Vec<String>>;
    fn get(&self, key: &str) -> Result<Option<MiraAny>>;
}

impl<T: MiraRecord> RecordObject for MiraShared<T> {
    fn identity(&self) -> usize {
        self.identity()
    }

    fn tag(&self) -> &'static str {
        type_name::<T>()
    }

    fn keys(&self) -> Result<Vec<String>> {
        self.inner
            .try_borrow()
            .map(|value| value.keys())
            .map_err(|_| {
                MiraError::BorrowConflict {
                    operation: "read",
                    tag: type_name::<T>().into(),
                }
                .into()
            })
    }

    fn get(&self, key: &str) -> Result<Option<MiraAny>> {
        self.inner
            .try_borrow()
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })?
            .get(key)
    }
}

#[doc(hidden)]
pub trait ArrayObject {
    fn identity(&self) -> usize;
    fn tag(&self) -> &'static str;
    fn len(&self) -> Result<usize>;
    fn get(&self, index: usize) -> Result<Option<MiraAny>>;
}

impl<T: MiraArray> ArrayObject for MiraShared<T> {
    fn identity(&self) -> usize {
        self.identity()
    }

    fn tag(&self) -> &'static str {
        type_name::<T>()
    }

    fn len(&self) -> Result<usize> {
        self.inner
            .try_borrow()
            .map(|value| value.len())
            .map_err(|_| {
                MiraError::BorrowConflict {
                    operation: "read",
                    tag: type_name::<T>().into(),
                }
                .into()
            })
    }

    fn get(&self, index: usize) -> Result<Option<MiraAny>> {
        self.inner
            .try_borrow()
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })?
            .get(index)
    }
}

mod handle;
mod index_map;
mod map;
mod tuple;

use std::any::Any;

use crate::{
    Result, Runtime,
    value::{MiraHandle, MiraManageable},
};

use super::{MiraValue, MiraValueKind, value::ValueTag};

/// One field produced by [`MiraRecord::iter`].
pub struct MiraRecordEntry<'a> {
    index: usize,
    key: &'a str,
    value: MiraManageable,
}

impl<'a> MiraRecordEntry<'a> {
    /// Create an entry for a record field at its iteration index.
    pub fn new(index: usize, key: &'a str, value: MiraManageable) -> Self {
        Self { index, key, value }
    }

    /// Return the field's iteration index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Return the field name.
    pub fn key(&self) -> &'a str {
        self.key
    }

    /// Consume the entry and return its value for insertion into a [`Runtime`].
    pub fn into_value(self) -> MiraManageable {
        self.value
    }
}

/// A sequential iterator over record fields.
pub type MiraRecordIter<'a> = Box<dyn ExactSizeIterator<Item = Result<MiraRecordEntry<'a>>> + 'a>;

impl MiraValue {
    /// Create a `MiraValue` representing a record.
    #[inline]
    pub const fn record<T: MiraRecord + ?Sized>(value: MiraHandle<T>) -> Self {
        Self::handle(ValueTag::Record, value.erase_record())
    }

    /// Check whether this value is a record.
    #[inline]
    pub const fn is_record(&self) -> bool {
        matches!(self.tag(), Some(ValueTag::Record))
    }

    /// Return the record handle, or `None` for another value type.
    #[inline]
    pub fn as_record(&self) -> Option<MiraHandle<dyn MiraRecord>> {
        match self.kind() {
            MiraValueKind::Record(value) => Some(value),
            _ => None,
        }
    }
}

/// A read-only MiraScript record view.
pub trait MiraRecord: Any {
    /// Return the number of fields.
    fn len(&self) -> usize;

    /// Find a field's iteration index.
    fn index_of(&self, key: &str) -> Option<usize>;

    /// Find an integer field key without allocating when possible.
    fn index_of_i(&self, key: u32) -> Option<usize> {
        self.index_of(&key.to_string())
    }

    /// Read a field key by iteration index.
    fn key(&self, index: usize) -> Result<&str>;

    /// Read a field value by iteration index.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;

    /// Return a more efficient sequential field iterator when indexed reads are unsuitable.
    ///
    /// The default `None` keeps full materialization on [`MiraRecord::key`] and
    /// [`MiraRecord::get`]. Implementations backed by containers without constant-time indexed
    /// access can return an iterator to avoid repeatedly traversing from the beginning.
    fn iter<'a>(
        &'a self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &'a Runtime,
    ) -> Option<MiraRecordIter<'a>> {
        let _ = (self_handle, runtime);
        None
    }

    /// Return whether the record is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[doc(hidden)]
    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraRecord>> {
        let _ = runtime;
        Ok(None)
    }
}

/// A fixed-shape record whose field names are known from its Rust type.
pub trait MiraShapedRecord: Any {
    /// Return the fixed number of fields.
    fn len() -> usize;

    /// Find a field's iteration index.
    fn index_of(key: &str) -> Option<usize>;

    /// Find an integer field key without allocating when possible.
    fn index_of_i(key: u32) -> Option<usize> {
        Self::index_of(&key.to_string())
    }

    /// Read a static field key by iteration index.
    fn key(index: usize) -> Result<&'static str>;

    /// Read a field value by iteration index.
    ///
    /// You should not call this method directly; use [`MiraRecord::get`] instead.
    #[doc(hidden)]
    fn get_shaped(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;
}

impl<T: MiraShapedRecord> MiraRecord for T {
    fn len(&self) -> usize {
        T::len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        T::index_of(key)
    }

    fn index_of_i(&self, key: u32) -> Option<usize> {
        T::index_of_i(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        T::key(index)
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        T::get_shaped(self, self_handle, runtime, index)
    }
}

#[cfg(test)]
fn test_record<T: MiraRecord + Into<MiraManageable>>(record: T, expected_json: &str) {
    let mut runtime = Runtime::new();

    runtime.insert_global("r_json", expected_json).unwrap();

    let len: usize = runtime
        .eval("from_json(r_json)::keys()::len()")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(record.len(), len);

    let empty = runtime
        .eval("from_json(r_json)::keys()::len() == 0")
        .unwrap()
        .as_boolean_unchecked();
    assert_eq!(record.is_empty(), empty);

    runtime.insert_global("record", record).unwrap();
    assert!(
        runtime
            .eval("record == from_json(r_json)")
            .unwrap()
            .as_boolean_unchecked()
    );
    assert!(
        runtime
            .eval("record::entries()::len() == from_json(r_json)::entries()::len()")
            .unwrap()
            .as_boolean_unchecked()
    );
    assert!(
        runtime
            .eval("record[`non exist key DO NOT INSERT THIS IN TESTING`]")
            .unwrap()
            .is_nil()
    )
}

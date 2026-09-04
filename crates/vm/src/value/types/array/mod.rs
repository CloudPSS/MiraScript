mod arr;
mod handle;
mod slice;

use std::any::Any;

use crate::{MiraArrayHandle, MiraHandle, MiraManageable, Result, Runtime};

use super::{MiraValue, MiraValueKind, value::ValueTag};

/// One element produced by [`MiraArray::iter`].
pub struct MiraArrayEntry {
    index: usize,
    value: MiraManageable,
}

impl MiraArrayEntry {
    /// Create an entry for an array element at its iteration index.
    pub fn new(index: usize, value: MiraManageable) -> Self {
        Self { index, value }
    }

    /// Return the element index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Consume the entry and return its value for insertion into a [`Runtime`].
    pub fn into_value(self) -> MiraManageable {
        self.value
    }
}

/// A sequential iterator over array elements.
pub type MiraArrayIter<'a> = Box<dyn ExactSizeIterator<Item = Result<MiraArrayEntry>> + 'a>;

impl MiraValue {
    /// Create a `MiraValue` representing an array value.
    #[inline]
    pub const fn array<T: MiraArray + ?Sized>(value: MiraHandle<T>) -> Self {
        Self::handle(ValueTag::Array, value.erase_array())
    }

    /// Check whether this value is an array value.
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self.tag(), Some(ValueTag::Array))
    }

    /// Return the array handle, or `None` for another value type.
    #[inline]
    pub fn as_array(&self) -> Option<MiraArrayHandle> {
        match self.kind() {
            MiraValueKind::Array(value) => Some(value),
            _ => None,
        }
    }
}

/// A read-only MiraScript array view.
pub trait MiraArray: Any {
    /// Return the number of elements.
    fn len(&self) -> usize;

    /// Read one element by index.
    fn get(
        &self,
        self_handle: MiraArrayHandle,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;

    /// Return a more efficient sequential iterator when indexed reads are unsuitable.
    ///
    /// The default `None` keeps full materialization on [`MiraArray::get`]. Implementations
    /// backed by containers without constant-time indexed access can return an iterator to
    /// avoid repeatedly traversing from the beginning.
    fn iter<'a>(
        &'a self,
        self_handle: MiraArrayHandle,
        runtime: &'a Runtime,
    ) -> Option<MiraArrayIter<'a>> {
        let _ = (self_handle, runtime);
        None
    }

    /// Return whether the array is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[doc(hidden)]
    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraArray>> {
        let _ = runtime;
        Ok(None)
    }
}

/// A fixed-shape array whose length is known from its Rust type.
pub trait MiraShapedArray: Any {
    /// Return the fixed number of elements.
    fn len() -> usize;

    /// Read one element by index.
    ///
    /// You should not call this method directly; use [`MiraArray::get`] instead.
    #[doc(hidden)]
    fn get_shaped(
        &self,
        self_handle: MiraArrayHandle,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;
}

impl<T: MiraShapedArray> MiraArray for T {
    fn len(&self) -> usize {
        T::len()
    }

    fn get(
        &self,
        self_handle: MiraArrayHandle,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        T::get_shaped(self, self_handle, runtime, index)
    }
}

#[cfg(test)]
fn test_array<T: MiraArray + Into<MiraManageable>>(array: T, expected_json: &str) {
    let mut runtime = Runtime::new();

    runtime.insert_global("a_json", expected_json).unwrap();

    let len: usize = runtime
        .eval("from_json(a_json)::len()")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(array.len(), len);

    let empty = runtime
        .eval("from_json(a_json)::len() == 0")
        .unwrap()
        .as_boolean_unchecked();
    assert_eq!(array.is_empty(), empty);

    runtime.insert_global("array", array).unwrap();
    assert!(
        runtime
            .eval("array == from_json(a_json)")
            .unwrap()
            .as_boolean_unchecked()
    );
}

mod arr;
mod boxed_slice;
mod vec;

use std::any::Any;

use crate::{MiraHandle, MiraManageable, Result, Runtime};

use super::{MiraValue, MiraValueKind, value::ValueTag};

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
    pub fn as_array(&self) -> Option<MiraHandle<dyn MiraArray>> {
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
        self_handle: MiraHandle<dyn MiraArray>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;

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
pub trait MiraShapedArray: Any + 'static {
    /// Return the fixed number of elements.
    fn len() -> usize;

    /// Read one element by index.
    ///
    /// You should not call this method directly; use [`MiraArray::get`] instead.
    #[doc(hidden)]
    fn get_shaped(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
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
        self_handle: MiraHandle<dyn MiraArray>,
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
        .as_boolean()
        .unwrap();
    assert_eq!(array.is_empty(), empty);

    runtime.insert_global("array", array).unwrap();
    assert!(
        runtime
            .eval("array == from_json(a_json)")
            .unwrap()
            .as_boolean()
            .unwrap()
    );
}

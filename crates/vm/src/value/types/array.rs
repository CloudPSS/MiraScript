use std::any::Any;

use crate::{
    MiraError, Result, Runtime, RuntimeErrorKind,
    value::{MiraHandle, MiraManageable},
};

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
    fn get(
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
        T::get(self, self_handle, runtime, index)
    }
}

impl<T: Clone + Into<MiraManageable> + 'static> MiraArray for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.as_slice()
            .get(index)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: Clone + Into<MiraManageable> + 'static> From<Vec<T>> for MiraManageable {
    fn from(value: Vec<T>) -> Self {
        Self::from_array(value)
    }
}

impl<T: Clone + Into<MiraManageable> + 'static, const N: usize> MiraArray for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.as_slice()
            .get(index)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: Clone + Into<MiraManageable> + 'static, const N: usize> From<[T; N]> for MiraManageable {
    fn from(value: [T; N]) -> Self {
        Self::from_array(value)
    }
}

impl<T: Clone + Into<MiraManageable> + 'static> MiraArray for Box<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.as_ref()
            .get(index)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: Clone + Into<MiraManageable> + 'static> From<Box<[T]>> for MiraManageable {
    fn from(value: Box<[T]>) -> Self {
        Self::from_array(value)
    }
}

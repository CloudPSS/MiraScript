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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_missing_index(error: &MiraError) {
        assert!(matches!(
            error,
            MiraError::Runtime {
                kind: RuntimeErrorKind::MissingIndexOrField,
                ..
            }
        ));
    }

    fn assert_value(value: MiraManageable, expected: MiraValue) {
        assert!(matches!(value, MiraManageable::Value(value) if value == expected));
    }

    #[test]
    fn owned_array_shapes_expose_values_and_bounds() {
        let mut runtime = Runtime::new();

        let vector = runtime.insert_array(vec![1_i32, 2]).unwrap();
        let value = MiraValue::array(vector);
        assert!(value.is_array());
        assert_eq!(value.as_array(), Some(vector.erase_array()));
        assert!(MiraValue::nil().as_array().is_none());
        let array = runtime.get_array(vector).unwrap();
        assert_eq!(array.len(), 2);
        assert!(!array.is_empty());
        assert!(MiraArray::resolve(array, &runtime).unwrap().is_none());
        assert_value(
            array.get(vector.erase_array(), &runtime, 1).unwrap(),
            2.into(),
        );
        assert_missing_index(
            array
                .get(vector.erase_array(), &runtime, usize::MAX)
                .err()
                .unwrap()
                .as_ref(),
        );

        let fixed = runtime.insert_array([3_i32, 4]).unwrap();
        let array = runtime.get_array(fixed).unwrap();
        assert_eq!(array.len(), 2);
        assert_value(
            array.get(fixed.erase_array(), &runtime, 0).unwrap(),
            3.into(),
        );
        assert_missing_index(
            array
                .get(fixed.erase_array(), &runtime, 2)
                .err()
                .unwrap()
                .as_ref(),
        );

        let boxed = runtime
            .insert_array(Vec::from([5_i32, 6]).into_boxed_slice())
            .unwrap();
        let array = runtime.get_array(boxed).unwrap();
        assert_eq!(array.len(), 2);
        assert_value(
            array.get(boxed.erase_array(), &runtime, 1).unwrap(),
            6.into(),
        );
        assert_missing_index(
            array
                .get(boxed.erase_array(), &runtime, 2)
                .err()
                .unwrap()
                .as_ref(),
        );

        let empty = runtime.insert_array(Vec::<i32>::new()).unwrap();
        assert!(runtime.get_array(empty).unwrap().is_empty());
    }
}

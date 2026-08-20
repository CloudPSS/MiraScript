use crate::{
    MiraError, Result,
    interpreter::Runtime,
    value::arena::{MiraHandle, MiraManageable},
};

/// A MiraScript array.
pub trait MiraArray: std::any::Any + 'static {
    /// Return the number of elements in the array.
    fn len(&self) -> usize;

    /// Read an element by index, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable>;

    /// Return whether this array currently contains no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
/// A shaped array is an array that has a fixed set of elements, where the types of the elements are known at compile time. Shaped arrays can be used to represent tuples or arrays with a known schema.
pub trait MiraShapedArray: std::any::Any + 'static {
    /// Return the number of elements in the array.
    fn len() -> usize;

    /// Read an element by index, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable>;
}

impl<T: MiraShapedArray> MiraArray for T {
    fn len(&self) -> usize {
        <T as MiraShapedArray>::len()
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        <T as MiraShapedArray>::get(self, self_handle, runtime, index)
    }
}

impl<T: Into<MiraManageable> + Clone + 'static> MiraArray for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        let val = self
            .as_slice()
            .get(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.clone().into())
    }
}

impl<T: Into<MiraManageable> + Clone + 'static, const N: usize> MiraArray for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        if index >= N {
            return Err(Box::new(MiraError::MissingIndexOrField));
        }
        let val = &self[index];
        Ok(val.clone().into())
    }
}

impl<T: Into<MiraManageable> + Clone + 'static> MiraArray for Box<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        let val = self
            .as_ref()
            .get(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.clone().into())
    }
}

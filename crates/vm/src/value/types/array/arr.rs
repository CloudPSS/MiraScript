use crate::{
    __private::{MiraField, MiraFieldGetter, shaped_array_from_array, shaped_array_from_record},
    MiraError, MiraHandle, MiraManageable, MiraRecord, Result, Runtime, RuntimeErrorKind,
};

use super::{MiraArray, MiraShapedArray};

impl<T: MiraField, const N: usize> MiraShapedArray for [T; N] {
    fn len() -> usize {
        N
    }

    fn get_shaped(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        let self_handle = unsafe { self_handle.upcast::<Self>() };
        self.as_slice()
            .get(index)
            .map(|v| v.from_array(self_handle, index, |s, index| &s[index]))
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: MiraField, const N: usize> MiraField for [T; N] {
    fn from_record<P: MiraRecord>(
        &self,
        parent: MiraHandle<P>,
        index: usize,
        getter: MiraFieldGetter<P, Self>,
    ) -> MiraManageable {
        shaped_array_from_record(parent, index, getter)
    }

    fn from_array<P: MiraArray>(
        &self,
        parent: MiraHandle<P>,
        index: usize,
        getter: MiraFieldGetter<P, Self>,
    ) -> MiraManageable {
        shaped_array_from_array(parent, index, getter)
    }
}

impl<T: MiraField, const N: usize> From<[T; N]> for MiraManageable {
    fn from(value: [T; N]) -> Self {
        Self::from_array(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::types::array::test_array;

    #[test]
    fn array_array() {
        let arr = [[1, 2], [3, 4]];
        test_array(arr, r#"[[1, 2], [3, 4]]"#);
    }
}

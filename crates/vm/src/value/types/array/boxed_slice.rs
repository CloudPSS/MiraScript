use crate::{
    __private::{MiraField, MiraFieldGetter, array_from_array, array_from_record},
    MiraError, MiraHandle, MiraManageable, MiraRecord, Result, Runtime, RuntimeErrorKind,
};

use super::MiraArray;

impl<T: MiraField> MiraArray for Box<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        let self_handle = unsafe { self_handle.upcast::<Self>() };
        self.as_ref()
            .get(index)
            .map(|v| v.from_array(self_handle, index, |s, index| &s[index]))
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: MiraField> MiraField for Box<[T]> {
    fn from_record<P: MiraRecord>(
        &self,
        parent: MiraHandle<P>,
        index: usize,
        getter: MiraFieldGetter<P, Self>,
    ) -> MiraManageable {
        array_from_record(parent, index, self, getter)
    }

    fn from_array<P: MiraArray>(
        &self,
        parent: MiraHandle<P>,
        index: usize,
        getter: MiraFieldGetter<P, Self>,
    ) -> MiraManageable {
        array_from_array(parent, index, self, getter)
    }
}

impl<T: MiraField> From<Box<[T]>> for MiraManageable {
    fn from(value: Box<[T]>) -> Self {
        Self::from_array(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::types::array::test_array;

    #[test]
    fn boxed_slice_array() {
        let arr: Box<[Box<[_]>]> = Box::new([Box::new([1, 2]), Box::new([3, 4, 5])]);
        test_array(arr, r#"[[1, 2], [3, 4, 5]]"#);
    }
    #[test]

    fn in_boxed_slice_array() {
        let arr: Box<[Box<[_]>]> = Box::new([Box::new(["x"]), Box::new(["y", "z"])]);
        test_array(arr, r#"[["x"], ["y", "z"]]"#);
    }
}

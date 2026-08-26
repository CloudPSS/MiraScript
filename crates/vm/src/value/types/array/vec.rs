use crate::{
    __private::{MiraField, MiraFieldGetter, array_from_array, array_from_record},
    MiraError, MiraHandle, MiraManageable, MiraRecord, Result, Runtime, RuntimeErrorKind,
};

use super::MiraArray;

impl<T: MiraField> MiraArray for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn get(
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

impl<T: MiraField> MiraField for Vec<T> {
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

impl<T: MiraField> From<Vec<T>> for MiraManageable {
    fn from(value: Vec<T>) -> Self {
        Self::from_array(value)
    }
}

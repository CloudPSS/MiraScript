use crate::{MiraArray, MiraHandle, MiraManageable, MiraRecord, MiraShapedArray, Result, Runtime};

use super::MiraFieldGetter;
struct ArrayFromRecord<P: MiraRecord + 'static, T> {
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
}

impl<P: MiraRecord, T: MiraShapedArray> MiraArray for ArrayFromRecord<P, T> {
    fn len(&self) -> usize {
        T::len()
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        T::get(
            (self.getter)(runtime.get_record(self.parent)?, self.index),
            self_handle,
            runtime,
            index,
        )
    }

    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraArray>> {
        Ok(Some((self.getter)(
            runtime.get_record(self.parent)?,
            self.index,
        )))
    }
}

struct ArrayFromArray<P: MiraArray + 'static, T> {
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
}

impl<P: MiraArray, T: MiraShapedArray> MiraArray for ArrayFromArray<P, T> {
    fn len(&self) -> usize {
        T::len()
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraArray>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        T::get(
            (self.getter)(runtime.get_array(self.parent)?, self.index),
            self_handle,
            runtime,
            index,
        )
    }

    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraArray>> {
        Ok(Some((self.getter)(
            runtime.get_array(self.parent)?,
            self.index,
        )))
    }
}

/// Build a live array field projection from a record parent.
#[doc(hidden)]
pub fn shaped_array_from_record<P: MiraRecord, T: MiraShapedArray>(
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
) -> MiraManageable {
    MiraManageable::from_array(ArrayFromRecord {
        parent,
        index,
        getter,
    })
}

/// Build a live array field projection from an array parent.
#[doc(hidden)]
pub fn shaped_array_from_array<P: MiraArray, T: MiraShapedArray>(
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
) -> MiraManageable {
    MiraManageable::from_array(ArrayFromArray {
        parent,
        index,
        getter,
    })
}

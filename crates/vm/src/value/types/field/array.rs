use super::*;

struct ArrayFromRecord<P: MiraRecord, T> {
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
    len: usize,
}

impl<P: MiraRecord, T: MiraArray> MiraArray for ArrayFromRecord<P, T> {
    fn len(&self) -> usize {
        self.len
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

struct ArrayFromArray<P: MiraArray, T> {
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
    len: usize,
}

impl<P: MiraArray, T: MiraArray> MiraArray for ArrayFromArray<P, T> {
    fn len(&self) -> usize {
        self.len
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
pub fn array_from_record<P: MiraRecord, T: MiraArray>(
    parent: MiraHandle<P>,
    index: usize,
    value: &T,
    getter: MiraFieldGetter<P, T>,
) -> MiraManageable {
    MiraManageable::from_array(ArrayFromRecord {
        parent,
        index,
        getter,
        len: value.len(),
    })
}

/// Build a live array field projection from an array parent.
#[doc(hidden)]
pub fn array_from_array<P: MiraArray, T: MiraArray>(
    parent: MiraHandle<P>,
    index: usize,
    value: &T,
    getter: MiraFieldGetter<P, T>,
) -> MiraManageable {
    MiraManageable::from_array(ArrayFromArray {
        parent,
        index,
        getter,
        len: value.len(),
    })
}

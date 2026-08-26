use crate::{MiraArray, MiraHandle, MiraManageable, MiraRecord, MiraShapedRecord, Result, Runtime};

use super::MiraFieldGetter;

struct RecordFromRecord<P: MiraRecord + 'static, T> {
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
}

impl<P: MiraRecord, T: MiraShapedRecord> MiraRecord for RecordFromRecord<P, T> {
    fn len(&self) -> usize {
        T::len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        T::index_of(key)
    }

    fn index_of_i(&self, key: u32) -> Option<usize> {
        T::index_of_i(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        T::key(index)
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
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

    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraRecord>> {
        Ok(Some((self.getter)(
            runtime.get_record(self.parent)?,
            self.index,
        )))
    }
}

struct RecordFromArray<P: MiraArray + 'static, T> {
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
}

impl<P: MiraArray, T: MiraShapedRecord> MiraRecord for RecordFromArray<P, T> {
    fn len(&self) -> usize {
        T::len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        T::index_of(key)
    }

    fn index_of_i(&self, key: u32) -> Option<usize> {
        T::index_of_i(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        T::key(index)
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
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

    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraRecord>> {
        Ok(Some((self.getter)(
            runtime.get_array(self.parent)?,
            self.index,
        )))
    }
}

/// Build a live shaped-record field projection from a record parent.
#[doc(hidden)]
pub fn shaped_record_from_record<P: MiraRecord, T: MiraShapedRecord>(
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
) -> MiraManageable {
    MiraManageable::from_record(RecordFromRecord {
        parent,
        index,
        getter,
    })
}

/// Build a live shaped-record field projection from an array parent.
#[doc(hidden)]
pub fn shaped_record_from_array<P: MiraArray, T: MiraShapedRecord>(
    parent: MiraHandle<P>,
    index: usize,
    getter: MiraFieldGetter<P, T>,
) -> MiraManageable {
    MiraManageable::from_record(RecordFromArray {
        parent,
        index,
        getter,
    })
}

use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

use indexmap::IndexMap;

use crate::{
    Result, Runtime,
    value::{MiraHandle, MiraManageable},
};

use super::{MiraArray, MiraRecord, MiraShapedArray, MiraShapedRecord};

/// Hidden routing contract used by the derive macros for projected fields.
///
/// Implementations decide whether a field is copied inline or exposed as a
/// live array/record projection backed by its parent's typed handle.
#[doc(hidden)]
pub trait MiraField: Sized + 'static {
    /// Project a field from a record parent.
    fn from_record<P: MiraRecord>(
        &self,
        parent: MiraHandle<P>,
        getter: fn(&P) -> &Self,
    ) -> MiraManageable;

    /// Project a field from an array parent.
    fn from_array<P: MiraArray>(
        &self,
        parent: MiraHandle<P>,
        getter: fn(&P) -> &Self,
    ) -> MiraManageable;
}

macro_rules! impl_copy_field {
    ($($type:ty),* $(,)?) => {$ (
        impl MiraField for $type {
            fn from_record<P: MiraRecord>(
                &self,
                _parent: MiraHandle<P>,
                _getter: fn(&P) -> &Self,
            ) -> MiraManageable {
                (*self).into()
            }

            fn from_array<P: MiraArray>(
                &self,
                _parent: MiraHandle<P>,
                _getter: fn(&P) -> &Self,
            ) -> MiraManageable {
                (*self).into()
            }
        }
    )*};
}

impl_copy_field!(
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    crate::MiraValue,
);

impl MiraField for String {
    fn from_record<P: MiraRecord>(
        &self,
        _parent: MiraHandle<P>,
        _getter: fn(&P) -> &Self,
    ) -> MiraManageable {
        self.clone().into()
    }

    fn from_array<P: MiraArray>(
        &self,
        _parent: MiraHandle<P>,
        _getter: fn(&P) -> &Self,
    ) -> MiraManageable {
        self.clone().into()
    }
}

struct RecordFromRecord<P: 'static, T> {
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
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
            (self.getter)(runtime.get_record(self.parent)?),
            self_handle,
            runtime,
            index,
        )
    }

    fn target_any<'a>(&'a self, runtime: &'a Runtime) -> Result<&'a dyn Any> {
        Ok((self.getter)(runtime.get_record(self.parent)?))
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct RecordFromArray<P: 'static, T> {
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
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
            (self.getter)(runtime.get_array(self.parent)?),
            self_handle,
            runtime,
            index,
        )
    }

    fn target_any<'a>(&'a self, runtime: &'a Runtime) -> Result<&'a dyn Any> {
        Ok((self.getter)(runtime.get_array(self.parent)?))
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct ArrayFromRecord<P: 'static, T> {
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
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
            (self.getter)(runtime.get_record(self.parent)?),
            self_handle,
            runtime,
            index,
        )
    }

    fn target_any<'a>(&'a self, runtime: &'a Runtime) -> Result<&'a dyn Any> {
        Ok((self.getter)(runtime.get_record(self.parent)?))
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct ArrayFromArray<P: 'static, T> {
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
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
            (self.getter)(runtime.get_array(self.parent)?),
            self_handle,
            runtime,
            index,
        )
    }

    fn target_any<'a>(&'a self, runtime: &'a Runtime) -> Result<&'a dyn Any> {
        Ok((self.getter)(runtime.get_array(self.parent)?))
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Build a live shaped-record field projection from a record parent.
#[doc(hidden)]
pub fn shaped_record_from_record<P: MiraRecord, T: MiraShapedRecord>(
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
) -> MiraManageable {
    MiraManageable::from_record(RecordFromRecord { parent, getter })
}

/// Build a live shaped-record field projection from an array parent.
#[doc(hidden)]
pub fn shaped_record_from_array<P: MiraArray, T: MiraShapedRecord>(
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
) -> MiraManageable {
    MiraManageable::from_record(RecordFromArray { parent, getter })
}

/// Build a live array field projection from a record parent.
#[doc(hidden)]
pub fn shaped_array_from_record<P: MiraRecord, T: MiraShapedArray>(
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
) -> MiraManageable {
    MiraManageable::from_array(ArrayFromRecord { parent, getter })
}

/// Build a live array field projection from an array parent.
#[doc(hidden)]
pub fn shaped_array_from_array<P: MiraArray, T: MiraShapedArray>(
    parent: MiraHandle<P>,
    getter: fn(&P) -> &T,
) -> MiraManageable {
    MiraManageable::from_array(ArrayFromArray { parent, getter })
}

macro_rules! impl_array_field {
    ($type:ty) => {
        impl<T> MiraField for $type
        where
            $type: MiraArray + Clone,
            T: Clone + Into<MiraManageable> + 'static,
        {
            fn from_record<P: MiraRecord>(
                &self,
                _parent: MiraHandle<P>,
                _getter: fn(&P) -> &Self,
            ) -> MiraManageable {
                MiraManageable::from_array(self.clone())
            }

            fn from_array<P: MiraArray>(
                &self,
                _parent: MiraHandle<P>,
                _getter: fn(&P) -> &Self,
            ) -> MiraManageable {
                MiraManageable::from_array(self.clone())
            }
        }
    };
}

impl_array_field!(Vec<T>);
impl_array_field!(Box<[T]>);

impl<T, const N: usize> MiraField for [T; N]
where
    T: Clone + Into<MiraManageable> + 'static,
{
    fn from_record<P: MiraRecord>(
        &self,
        _parent: MiraHandle<P>,
        _getter: fn(&P) -> &Self,
    ) -> MiraManageable {
        MiraManageable::from_array(self.clone())
    }

    fn from_array<P: MiraArray>(
        &self,
        _parent: MiraHandle<P>,
        _getter: fn(&P) -> &Self,
    ) -> MiraManageable {
        MiraManageable::from_array(self.clone())
    }
}

macro_rules! impl_record_field {
    ($type:ty) => {
        impl<T> MiraField for $type
        where
            $type: MiraRecord + Clone,
            T: Clone + Into<MiraManageable> + 'static,
        {
            fn from_record<P: MiraRecord>(
                &self,
                _parent: MiraHandle<P>,
                _getter: fn(&P) -> &Self,
            ) -> MiraManageable {
                MiraManageable::from_record(self.clone())
            }

            fn from_array<P: MiraArray>(
                &self,
                _parent: MiraHandle<P>,
                _getter: fn(&P) -> &Self,
            ) -> MiraManageable {
                MiraManageable::from_record(self.clone())
            }
        }
    };
}

impl_record_field!(IndexMap<String, T>);
impl_record_field!(BTreeMap<String, T>);

impl<T, S> MiraField for HashMap<String, T, S>
where
    T: Clone + Into<MiraManageable> + 'static,
    S: BuildHasher + Clone + 'static,
{
    fn from_record<P: MiraRecord>(
        &self,
        _parent: MiraHandle<P>,
        _getter: fn(&P) -> &Self,
    ) -> MiraManageable {
        MiraManageable::from_record(self.clone())
    }

    fn from_array<P: MiraArray>(
        &self,
        _parent: MiraHandle<P>,
        _getter: fn(&P) -> &Self,
    ) -> MiraManageable {
        MiraManageable::from_record(self.clone())
    }
}

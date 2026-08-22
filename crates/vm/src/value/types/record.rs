use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

use indexmap::IndexMap;

use crate::{
    MiraError, Result, Runtime, RuntimeErrorKind,
    value::{MiraHandle, MiraManageable},
};

use super::{MiraValue, MiraValueKind};

impl MiraValue {
    /// Create a `MiraValue` representing a record.
    #[inline]
    pub fn record<T: MiraRecord + ?Sized>(value: MiraHandle<T>) -> Self {
        Self::from_record_handle(value.erase_record())
    }

    /// Check whether this value is a record.
    #[inline]
    pub fn is_record(&self) -> bool {
        matches!(self.kind(), MiraValueKind::Record(_))
    }

    /// Return the record handle, or `None` for another value type.
    #[inline]
    pub fn as_record(&self) -> Option<MiraHandle<dyn MiraRecord>> {
        match self.kind() {
            MiraValueKind::Record(value) => Some(value),
            _ => None,
        }
    }
}

/// A read-only MiraScript record view.
pub trait MiraRecord: Any {
    /// Return the number of fields.
    fn len(&self) -> usize;

    /// Find a field's iteration index.
    fn index_of(&self, key: &str) -> Option<usize>;

    /// Find an integer field key without allocating when possible.
    fn index_of_i(&self, key: u32) -> Option<usize> {
        self.index_of(&key.to_string())
    }

    /// Read a field key by iteration index.
    fn key(&self, index: usize) -> Result<&str>;

    /// Read a field value by iteration index.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;

    /// Return whether the record is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[doc(hidden)]
    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraRecord>> {
        let _ = runtime;
        Ok(None)
    }
}

/// A fixed-shape record whose field names are known from its Rust type.
pub trait MiraShapedRecord: Any + 'static {
    /// Return the fixed number of fields.
    fn len() -> usize;

    /// Find a field's iteration index.
    fn index_of(key: &str) -> Option<usize>;

    /// Find an integer field key without allocating when possible.
    fn index_of_i(key: u32) -> Option<usize> {
        Self::index_of(&key.to_string())
    }

    /// Read a static field key by iteration index.
    fn key(index: usize) -> Result<&'static str>;

    /// Read a field value by iteration index.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;
}

impl<T: MiraShapedRecord> MiraRecord for T {
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
        T::get(self, self_handle, runtime, index)
    }
}

macro_rules! impl_map_record {
    ($type:ty) => {
        impl<T: Clone + Into<MiraManageable> + 'static> MiraRecord for $type {
            fn len(&self) -> usize {
                <$type>::len(self)
            }

            fn index_of(&self, key: &str) -> Option<usize> {
                self.keys().position(|candidate| candidate == key)
            }

            fn key(&self, index: usize) -> Result<&str> {
                self.keys()
                    .nth(index)
                    .map(String::as_str)
                    .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
            }

            fn get(
                &self,
                _self_handle: MiraHandle<dyn MiraRecord>,
                _runtime: &Runtime,
                index: usize,
            ) -> Result<MiraManageable> {
                self.values()
                    .nth(index)
                    .cloned()
                    .map(Into::into)
                    .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
            }
        }

        impl<T: Clone + Into<MiraManageable> + 'static> From<$type> for MiraManageable {
            fn from(value: $type) -> Self {
                Self::from_record(value)
            }
        }
    };
}

impl<T: Clone + Into<MiraManageable> + 'static> MiraRecord for IndexMap<String, T> {
    fn len(&self) -> usize {
        IndexMap::len(self)
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.get_index_of(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        self.get_index(index)
            .map(|(key, _)| key.as_str())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.get_index(index)
            .map(|(_, value)| value.clone().into())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: Clone + Into<MiraManageable> + 'static> From<IndexMap<String, T>> for MiraManageable {
    fn from(value: IndexMap<String, T>) -> Self {
        Self::from_record(value)
    }
}

impl_map_record!(BTreeMap<String, T>);

impl<T, S> MiraRecord for HashMap<String, T, S>
where
    T: Clone + Into<MiraManageable> + 'static,
    S: BuildHasher + 'static,
{
    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.keys().position(|candidate| candidate == key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        self.keys()
            .nth(index)
            .map(String::as_str)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.values()
            .nth(index)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl MiraShapedRecord for () {
    fn len() -> usize {
        0
    }

    fn index_of(_key: &str) -> Option<usize> {
        None
    }

    fn key(_index: usize) -> Result<&'static str> {
        Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        _index: usize,
    ) -> Result<MiraManageable> {
        Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl From<()> for MiraManageable {
    fn from(value: ()) -> Self {
        Self::from_record(value)
    }
}

macro_rules! impl_tuple_record {
    ($len:expr; $(($T:ident, $index:tt)),+ $(,)?) => {
        impl<$($T),+> MiraShapedRecord for ($($T,)+)
        where
            $($T: Clone + Into<MiraManageable> + 'static,)+
        {
            fn len() -> usize { $len }

            fn index_of(key: &str) -> Option<usize> {
                key.parse::<usize>().ok().filter(|index| *index < $len)
            }

            fn index_of_i(key: u32) -> Option<usize> {
                let key = key as usize;
                (key < $len).then_some(key)
            }

            fn key(index: usize) -> Result<&'static str> {
                match index {
                    $($index => Ok(stringify!($index)),)+
                    _ => Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField)),
                }
            }

            fn get(
                &self,
                _self_handle: MiraHandle<dyn MiraRecord>,
                _runtime: &Runtime,
                index: usize,
            ) -> Result<MiraManageable> {
                match index {
                    $($index => Ok(self.$index.clone().into()),)+
                    _ => Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField)),
                }
            }
        }

        impl<$($T),+> From<($($T,)+)> for MiraManageable
        where
            $($T: Clone + Into<MiraManageable> + 'static,)+
        {
            fn from(value: ($($T,)+)) -> Self {
                Self::from_record(value)
            }
        }
    };
}

impl_tuple_record!(1; (T0, 0));
impl_tuple_record!(2; (T0, 0), (T1, 1));
impl_tuple_record!(3; (T0, 0), (T1, 1), (T2, 2));
impl_tuple_record!(4; (T0, 0), (T1, 1), (T2, 2), (T3, 3));
impl_tuple_record!(5; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4));
impl_tuple_record!(6; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5));
impl_tuple_record!(7; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6));
impl_tuple_record!(8; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7));

use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

use super::MiraValue;
use crate::{
    MiraError, Result,
    interpreter::Runtime,
    value::arena::{MiraHandle, MiraManageable},
};
use indexmap::IndexMap;

/// A record is a collection of key-value pairs, where keys are strings and values are MiraScript constants. Records can be used to represent objects, structs, or any other data structure that has named fields.
pub trait MiraRecord: std::any::Any + 'static {
    /// Return the number of fields in the record.
    fn len(&self) -> usize;

    /// Get index of a field by key, in iteration order, Returns [`None`] if the field does not exist.
    fn index_of(&self, key: &str) -> Option<usize>;

    /// Get index of a field by integer key, in iteration order, Returns [`None`] if the field does not exist.
    fn index_of_i(&self, key: u32) -> Option<usize> {
        let key = key.to_string();
        self.index_of(&key)
    }

    /// Read a key by index, in iteration order, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn key(&self, index: usize) -> Result<&str>;

    /// Read a field by index, in iteration order, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable>;

    /// Return whether this record currently contains no fields.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A shaped record is a record that has a fixed set of fields, where the keys and types of the fields are known at compile time. Shaped records can be used to represent structs or objects with a known schema.
pub trait MiraShapedRecord: std::any::Any + 'static {
    /// Return the number of fields in the record.
    fn len() -> usize;

    /// Get index of a field by key, in iteration order, Returns [`None`] if the field does not exist.
    fn index_of(key: &str) -> Option<usize>;

    /// Get index of a field by integer key, in iteration order, Returns [`None`] if the field does not exist.
    fn index_of_i(key: u32) -> Option<usize> {
        let key = key.to_string();
        <Self as MiraShapedRecord>::index_of(&key)
    }

    /// Read a key by index, in iteration order, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn key(index: usize) -> Result<&'static str>;

    /// Read a field by index, in iteration order, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable>;
}

impl<T: MiraShapedRecord> MiraRecord for T {
    fn len(&self) -> usize {
        <T as MiraShapedRecord>::len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        <T as MiraShapedRecord>::index_of(key)
    }

    fn index_of_i(&self, key: u32) -> Option<usize> {
        <T as MiraShapedRecord>::index_of_i(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        <T as MiraShapedRecord>::key(index)
    }

    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraRecord>,
        runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        <T as MiraShapedRecord>::get(self, self_handle, runtime, index)
    }
}

impl<T: Into<MiraValue> + Clone + 'static> MiraRecord for IndexMap<String, T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.get_index_of(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        let val = self
            .get_index(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.0.as_str())
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        let val = self
            .get_index(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.1.clone().into().into())
    }
}

impl<T: Into<MiraValue> + Clone + 'static, S: BuildHasher + 'static> MiraRecord
    for HashMap<String, T, S>
{
    fn len(&self) -> usize {
        self.len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.iter().position(|(k, _)| k == key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        let val = self
            .iter()
            .nth(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.0.as_str())
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        let val = self
            .iter()
            .nth(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.1.clone().into().into())
    }
}

impl<T: Into<MiraValue> + Clone + 'static> MiraRecord for BTreeMap<String, T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.iter().position(|(k, _)| k == key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        let val = self
            .iter()
            .nth(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.0.as_str())
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime<'_>,
        index: usize,
    ) -> Result<MiraManageable> {
        let val = self
            .iter()
            .nth(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.1.clone().into().into())
    }
}

impl MiraRecord for () {
    fn len(&self) -> usize {
        0
    }

    fn index_of(&self, _key: &str) -> Option<usize> {
        None
    }

    fn key(&self, _index: usize) -> Result<&str> {
        Err(Box::new(MiraError::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime<'_>,
        _index: usize,
    ) -> Result<MiraManageable> {
        Err(Box::new(MiraError::MissingIndexOrField))
    }

    fn is_empty(&self) -> bool {
        true
    }
}

macro_rules! impl_mira_record_for_tuple {
    ($len:expr; $(($T:ident, $idx:tt)),+ $(,)?) => {
        impl<$($T),*> MiraRecord for ($($T,)*)
        where
            $(
                $T: Into<MiraManageable> + Clone + 'static,
            )*
        {
            fn len(&self) -> usize {
                $len
            }

            fn index_of(&self, key: &str) -> Option<usize> {
                if self.is_empty() {
                    return None;
                }
                match key.parse::<usize>() {
                    Ok(idx) if idx < $len => Some(idx),
                    _ => None,
                }
            }

            fn index_of_i(&self, key: u32) -> Option<usize> {
                let idx = key as usize;
                if idx < $len {
                    Some(idx)
                } else {
                    None
                }
            }

            fn key(&self, index: usize) -> Result<&'static str> {
                match index {
                    $(
                        $idx => Ok(stringify!($idx)),
                    )*
                    _ => Err(Box::new(MiraError::MissingIndexOrField)),
                }
            }

            fn get(
                &self,
                _self_handle: MiraHandle<dyn MiraRecord>,
                _runtime: &Runtime<'_>,
                index: usize
            ) -> Result<MiraManageable> {
                match index {
                    $(
                        $idx => Ok(self.$idx.clone().into()),
                    )*
                    _ => Err(Box::new(MiraError::MissingIndexOrField)),
                }
            }

            fn is_empty(&self) -> bool {
                false
            }
        }
    };
}

impl_mira_record_for_tuple!(1; (T0, 0));
impl_mira_record_for_tuple!(2; (T0, 0), (T1, 1));
impl_mira_record_for_tuple!(3; (T0, 0), (T1, 1), (T2, 2));
impl_mira_record_for_tuple!(4; (T0, 0), (T1, 1), (T2, 2), (T3, 3));
impl_mira_record_for_tuple!(5; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4));
impl_mira_record_for_tuple!(6; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5));
impl_mira_record_for_tuple!(7; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6));
impl_mira_record_for_tuple!(8; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7));
impl_mira_record_for_tuple!(9; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8));
impl_mira_record_for_tuple!(10; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9));
impl_mira_record_for_tuple!(11; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9), (T10, 10));
impl_mira_record_for_tuple!(12; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9), (T10, 10), (T11, 11));
impl_mira_record_for_tuple!(13; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9), (T10, 10), (T11, 11), (T12, 12));
impl_mira_record_for_tuple!(14; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9), (T10, 10), (T11, 11), (T12, 12), (T13, 13));
impl_mira_record_for_tuple!(15; (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9), (T10, 10), (T11, 11), (T12, 12), (T13, 13), (T14, 14));

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_tuple_record() {
        fn check_record<R: MiraRecord>(record: &R) {
            assert_eq!(record.len(), 3);

            assert_eq!(record.index_of("0"), Some(0));
            assert_eq!(record.index_of("1"), Some(1));
            assert_eq!(record.index_of("2"), Some(2));
            assert_eq!(record.index_of("3"), None);

            assert_eq!(record.index_of_i(0), Some(0));
            assert_eq!(record.index_of_i(1), Some(1));
            assert_eq!(record.index_of_i(2), Some(2));
            assert_eq!(record.index_of_i(3), None);

            assert_eq!(record.key(0).unwrap(), "0");
            assert_eq!(record.key(1).unwrap(), "1");
            assert_eq!(record.key(2).unwrap(), "2");
            assert!(record.key(3).is_err());

            let runtime: &mut Runtime = &mut unsafe { Runtime::unused() };
            let handle: MiraHandle<dyn MiraRecord> = MiraHandle::empty();
            let v0 = record.get(handle, runtime, 0).unwrap();
            let v0 = runtime.insert(v0);
            let v1 = record.get(handle, runtime, 1).unwrap();
            let v1 = runtime.insert(v1);
            let v2 = record.get(handle, runtime, 2).unwrap();
            let v2 = runtime.insert(v2);
            assert_eq!(v0.as_number().unwrap(), 1f64);
            assert!(v1.as_boolean().unwrap());
            assert_eq!(v2.as_string(runtime).unwrap(), "Hello");
            assert!(record.get(handle, runtime, 3).is_err());
        }

        let record = (1, true, &"Hello");
        check_record(&record);
    }
}

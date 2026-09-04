use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

use crate::{
    __private::MiraField, MiraError, MiraHandle, MiraManageable, Result, Runtime, RuntimeErrorKind,
};

use super::{MiraRecord, MiraRecordEntry, MiraRecordIter};

macro_rules! impl_map (
    ($m:ident, $ty:ty, $dt:ty $(, $($generics:tt)*)? ) => {
        mod $m {
            use super::*;

            impl<T: MiraField $(, $($generics)*)?> MiraRecord for $ty {
                fn len(&self) -> usize {
                    <$ty>::len(self)
                }

                fn index_of(&self, key: &str) -> Option<usize> {
                    <$ty>::keys(self).position(|candidate| candidate == key)
                }

                fn key(&self, index: usize) -> Result<&str> {
                    <$ty>::keys(self)
                        .nth(index)
                        .map(String::as_str)
                        .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
                }

                fn get(
                    &self,
                    self_handle: MiraHandle<dyn MiraRecord>,
                    _runtime: &Runtime,
                    index: usize,
                ) -> Result<MiraManageable> {
                    let self_handle = unsafe { self_handle.upcast::<Self>() };
                    <$ty>::values(self)
                        .nth(index)
                        .map(|v| {
                            v.from_record(self_handle, index, |s, index| {
                                <$ty>::values(s)
                                    .nth(index)
                                    .expect(concat!(stringify!($ty), " changed unexpectedly"))
                            })
                        })
                        .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
                }

                fn iter<'a>(
                    &'a self,
                    self_handle: MiraHandle<dyn MiraRecord>,
                    _runtime: &'a Runtime,
                ) -> Option<MiraRecordIter<'a>> {
                    let self_handle = unsafe { self_handle.upcast::<Self>() };
                    Some(Box::new(<$ty>::iter(self).enumerate().map(
                        move |(index, (key, value))| {
                            Ok(MiraRecordEntry::new(
                                index,
                                key,
                                value.from_record(self_handle, index, |s, index| {
                                    <$ty>::values(s).nth(index).expect(concat!(
                                        stringify!($ty),
                                        " changed unexpectedly"
                                    ))
                                }),
                            ))
                        },
                    )))
                }
            }

            impl<T: MiraField $(, $($generics)*)?> From<$ty> for MiraManageable {
                fn from(value: $ty) -> Self {
                    Self::from_record(value)
                }
            }

            impl_map!(@tests $dt, $dt);
        }
    };

    (@tests $rdt:ty, $dt:ty) => {
        #[cfg(test)]
        mod tests {
            use crate::value::types::record::test_record;
            use super::*;

            type Rec<T> = $dt;

            fn new<T, F>(actions: F) -> Rec<T>
            where
                F: FnOnce(&mut $rdt),
            {
                let mut map = <$rdt>::new();
                actions(&mut map);
                map.into()
            }

            #[test]
            fn empty_record() {
                let map: Rec<i32> = new(|_| {});
                test_record(map, "{}");
            }

            #[test]
            fn one_element_record() {
                let map: Rec<i32> = new(|map| {
                    map.insert("0".to_string(), 1);
                });
                test_record(map, r#"{"0": 1}"#);
            }

            #[test]
            fn two_element_record() {
                let map: Rec<[u64; 3]> = new(|map| {
                    map.insert("0".to_string(), [1, 2, 3]);
                    map.insert("1".to_string(), [4, 5, 6]);
                });
                test_record(map, r#"{"0": [1, 2, 3], "1": [4, 5, 6]}"#);
            }
        }
    }
);

impl_map!(b_tree_map, BTreeMap<String, T>, BTreeMap<String, T>);
impl_map!(hash_map, HashMap<String, T, S>, HashMap<String, T>, S: BuildHasher + 'static);

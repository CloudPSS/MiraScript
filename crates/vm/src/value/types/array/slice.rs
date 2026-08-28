use std::{boxed::Box, rc::Rc, vec::Vec};

use crate::{
    __private::{MiraField, MiraFieldGetter, array_from_array, array_from_record},
    MiraError, MiraHandle, MiraManageable, MiraRecord, Result, Runtime, RuntimeErrorKind,
};

use super::MiraArray;

macro_rules! impl_slice (
    ($m:ident, $ty:ty, $as_slice:ident) => {
        mod $m {
            use super::*;

            impl<T: MiraField> MiraArray for $ty {
                fn len(&self) -> usize {
                    self.$as_slice().len()
                }

                fn get(
                    &self,
                    self_handle: MiraHandle<dyn MiraArray>,
                    _runtime: &Runtime,
                    index: usize,
                ) -> Result<MiraManageable> {
                    let self_handle = unsafe { self_handle.upcast::<Self>() };
                    self.$as_slice()
                        .get(index)
                        .map(|v| v.from_array(self_handle, index, |s, index| &s.$as_slice()[index]))
                        .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
                }
            }

            impl<T: MiraField> MiraField for $ty {
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

            impl<T: MiraField> From<$ty> for MiraManageable {
                fn from(value: $ty) -> Self {
                    Self::from_array(value)
                }
            }

            #[cfg(test)]
            mod tests {
                use crate::value::types::array::test_array;
                use super::*;

                type Arr<T> = $ty;

                #[test]
                fn int_array() {
                    let arr: Arr<Arr<i32>> = [[1, 2].into(), [3, 4, 5].into()].into();
                    test_array(arr, r#"[[1, 2], [3, 4, 5]]"#);
                }
                #[test]

                fn str_array() {
                    let arr: Arr<Arr<&str>> = [["x"].into(), ["y", "z"].into()].into();
                    test_array(arr, r#"[["x"], ["y", "z"]]"#);
                }
            }
        }
    };
);

impl_slice!(boxed_slice, Box<[T]>, as_ref);
impl_slice!(rc_slice, Rc<[T]>, as_ref);
impl_slice!(vec, Vec<T>, as_slice);

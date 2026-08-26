use crate::{
    __private::MiraField, MiraError, MiraHandle, MiraManageable, MiraShapedRecord, Result, Runtime,
    RuntimeErrorKind,
};

use super::MiraRecord;

impl MiraShapedRecord for () {
    fn len() -> usize {
        0
    }

    fn index_of(_key: &str) -> Option<usize> {
        None
    }

    fn index_of_i(_key: u32) -> Option<usize> {
        None
    }

    fn key(_index: usize) -> Result<&'static str> {
        Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get_shaped(
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
            $($T: MiraField,)+
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

            fn get_shaped(
                &self,
                self_handle: MiraHandle<dyn MiraRecord>,
                _runtime: &Runtime,
                index: usize,
            ) -> Result<MiraManageable> {
                let self_handle = unsafe { self_handle.upcast::<Self>() };
                match index {
                    $($index => Ok(<$T as MiraField>::from_record(&self.$index, self_handle, $index, |s, _| &s.$index)),)+
                    _ => Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField)),
                }
            }
        }

        impl<$($T),+> From<($($T,)+)> for MiraManageable
        where
            $($T: MiraField,)+
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

#[cfg(test)]
mod tests {
    use crate::{MiraValue, Runtime, value::types::record::test_record};

    #[test]
    fn empty_tuple_record() {
        test_record((), "{}");

        let mut runtime = Runtime::new();
        runtime.insert_global("tuple", ()).unwrap();
        assert!(runtime.eval("tuple.0").unwrap().is_nil());
    }
    #[test]
    fn one_element_tuple_record() {
        test_record((1,), r#"{"0": 1}"#);

        let mut runtime = Runtime::new();
        runtime.insert_global("tuple", (1,)).unwrap();
        assert_eq!(runtime.eval("tuple.0").unwrap().as_number().unwrap(), 1.0);
        assert!(runtime.eval("tuple.1").unwrap().is_nil());
    }
    #[test]
    fn two_element_tuple_record() {
        test_record((1, [1, 2, 3]), r#"{"0": 1, "1": [1, 2, 3]}"#);
    }
    #[test]
    fn three_element_tuple_record() {
        test_record((1, &"x", false), r#"{"0": 1, "1": "x", "2": false}"#);
    }
    #[test]
    fn four_element_tuple_record() {
        test_record(
            (1, &"x", false, MiraValue::nil()),
            r#"{"0": 1, "1": "x", "2": false, "3": null}"#,
        );
    }
}

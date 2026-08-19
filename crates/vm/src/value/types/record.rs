use super::MiraValue;
use crate::{MiraError, Result, interpreter::Runtime};
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
    fn get(&self, runtime: &Runtime<'_>, index: usize) -> Result<MiraValue>;

    /// Return whether this record currently contains no fields.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl MiraRecord for IndexMap<String, MiraValue> {
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

    fn get(&self, _runtime: &Runtime<'_>, index: usize) -> Result<MiraValue> {
        let val = self
            .get_index(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.1.clone())
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

    fn get(&self, _runtime: &Runtime<'_>, _index: usize) -> Result<MiraValue> {
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
                $T: Into<MiraValue> + Clone + 'static,
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

            fn get(&self, _runtime: &Runtime<'_>, index: usize) -> Result<MiraValue> {
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

            let runtime: &Runtime = &unsafe { Runtime::unused() };
            assert_eq!(record.get(runtime, 0).unwrap().as_number().unwrap(), 1f64);
            assert!(record.get(runtime, 1).unwrap().as_boolean().unwrap());
            assert_eq!(
                record.get(runtime, 2).unwrap().as_string(runtime).unwrap(),
                "Hello"
            );
            assert!(record.get(runtime, 3).is_err());
        }

        let record = (1, true, &"Hello");
        check_record(&record);
    }
}

use std::collections::BTreeMap;

use crate::{
    __private::MiraField, MiraError, MiraHandle, MiraManageable, Result, Runtime, RuntimeErrorKind,
};

use super::MiraRecord;

impl<T: MiraField> MiraRecord for BTreeMap<String, T> {
    fn len(&self) -> usize {
        BTreeMap::len(self)
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
        self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        let self_handle = unsafe { self_handle.upcast::<Self>() };
        self.values()
            .nth(index)
            .map(|v| {
                v.from_record(self_handle, index, |s, index| {
                    s.values()
                        .nth(index)
                        .expect("BTreeMap changed unexpectedly")
                })
            })
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: MiraField> From<BTreeMap<String, T>> for MiraManageable {
    fn from(value: BTreeMap<String, T>) -> Self {
        Self::from_record(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::types::record::test_record;

    use super::*;

    #[test]
    fn empty_b_tree_map_record() {
        test_record(BTreeMap::<String, u32>::new(), "{}");
    }

    #[test]
    fn one_element_b_tree_map_record() {
        let mut map = BTreeMap::new();
        map.insert("0".to_string(), 1);
        test_record(map, r#"{"0": 1}"#);
    }

    #[test]
    fn two_element_b_tree_map_record() {
        let mut map = BTreeMap::new();
        map.insert("0".to_string(), [1, 2, 3]);
        map.insert("1".to_string(), [4, 5, 6]);
        test_record(map, r#"{"0": [1, 2, 3], "1": [4, 5, 6]}"#);
    }
}

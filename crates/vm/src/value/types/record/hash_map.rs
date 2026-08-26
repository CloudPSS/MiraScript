use std::{collections::HashMap, hash::BuildHasher};

use crate::{
    __private::MiraField, MiraError, MiraHandle, MiraManageable, Result, Runtime, RuntimeErrorKind,
};

use super::MiraRecord;

impl<T: MiraField, S: BuildHasher + 'static> MiraRecord for HashMap<String, T, S> {
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
        self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        let self_handle = unsafe { self_handle.upcast::<Self>() };
        self.values()
            .nth(index)
            .map(|v| {
                v.from_record(self_handle, index, |s, index| {
                    s.values().nth(index).expect("HashMap changed unexpectedly")
                })
            })
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: MiraField, S: BuildHasher + 'static> From<HashMap<String, T, S>> for MiraManageable {
    fn from(value: HashMap<String, T, S>) -> Self {
        Self::from_record(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::types::record::test_record;

    use super::*;

    #[test]
    fn empty_hash_map_record() {
        test_record(HashMap::<String, u32>::new(), "{}");
    }

    #[test]
    fn one_element_hash_map_record() {
        let mut map = HashMap::new();
        map.insert("0".to_string(), 12);
        test_record(map, r#"{"0": 12}"#);
    }

    #[test]
    fn two_element_hash_map_record() {
        let mut map = HashMap::new();
        map.insert("0".to_string(), ["x", "y"]);
        map.insert("1".to_string(), ["a", "b"]);
        test_record(map, r#"{"0": ["x", "y"], "1": ["a", "b"]}"#);
    }
}

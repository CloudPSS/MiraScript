use indexmap::IndexMap;

use crate::{
    __private::MiraField, MiraError, MiraHandle, MiraManageable, Result, Runtime, RuntimeErrorKind,
};

use super::MiraRecord;

impl<T: MiraField> MiraRecord for IndexMap<String, T> {
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
        self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        let self_handle = unsafe { self_handle.upcast::<Self>() };
        self.get_index(index)
            .map(|(_, v)| {
                v.from_record(self_handle, index, |s, index| {
                    s.get_index(index)
                        .map(|(_, v)| v)
                        .expect("IndexMap changed unexpectedly")
                })
            })
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: MiraField> From<IndexMap<String, T>> for MiraManageable {
    fn from(value: IndexMap<String, T>) -> Self {
        Self::from_record(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::types::record::test_record;

    use super::*;

    #[test]
    fn empty_index_map_record() {
        test_record(IndexMap::<String, u32>::new(), "{}");
    }

    #[test]
    fn one_element_index_map_record() {
        let mut map = IndexMap::new();
        map.insert("0".to_string(), "a");
        test_record(map, r#"{"0": "a"}"#);
    }

    #[test]
    fn two_element_index_map_record() {
        let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
        map.insert("0".to_string(), vec!["x".to_string(), "y".to_string()]);
        map.insert(
            "1".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        test_record(map, r#"{"0": ["x", "y"], "1": ["a", "b", "c"]}"#);
    }

    #[test]
    fn three_element_index_map_record() {
        let mut map: IndexMap<String, Box<[Vec<i32>]>> = IndexMap::new();
        map.insert("0".to_string(), [[1, 2, 3].into(), [4, 5, 6].into()].into());
        map.insert("1".to_string(), [[7, 8].into(), [9].into()].into());
        test_record(map, r#"{"0": [[1, 2, 3], [4, 5, 6]], "1": [[7, 8], [9]]}"#);
    }
}

use indexmap::IndexMap;

use crate::{
    MiraError, Result, Runtime, RuntimeErrorKind,
    value::{MiraHandle, MiraManageable},
};

use super::MiraRecord;

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
        map.insert("0".to_string(), 1);
        test_record(map, r#"{"0": 1}"#);
    }
}

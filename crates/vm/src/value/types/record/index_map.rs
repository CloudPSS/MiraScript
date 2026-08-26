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
}

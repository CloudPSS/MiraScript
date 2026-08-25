use std::{collections::HashMap, hash::BuildHasher};

use crate::{
    MiraError, Result, Runtime, RuntimeErrorKind,
    value::{MiraHandle, MiraManageable},
};

use super::MiraRecord;

impl<T, S> MiraRecord for HashMap<String, T, S>
where
    T: Clone + Into<MiraManageable> + 'static,
    S: BuildHasher + 'static,
{
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
        _self_handle: MiraHandle<dyn MiraRecord>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.values()
            .nth(index)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

impl<T: Clone + Into<MiraManageable> + 'static, S: BuildHasher + 'static>
    From<HashMap<String, T, S>> for MiraManageable
{
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
        map.insert("0".to_string(), 1);
        test_record(map, r#"{"0": 1}"#);
    }
}

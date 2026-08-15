use indexmap::IndexMap;

use crate::{MiraAny, MiraNativeFn};

/// Globals visible to a MiraScript execution.
#[derive(Clone)]
pub struct MiraContext {
    values: IndexMap<String, MiraAny>,
}

impl MiraContext {
    pub fn new() -> Self {
        let mut context = Self {
            values: IndexMap::new(),
        };
        crate::standard_library::install(&mut context);
        context
    }

    pub fn empty() -> Self {
        Self {
            values: IndexMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: impl Into<MiraAny>,
    ) -> Option<MiraAny> {
        self.values.insert(name.into(), value.into())
    }

    pub fn insert_fn(&mut self, name: impl Into<String>, function: impl Into<MiraNativeFn>) {
        self.insert(
            name,
            MiraAny::Function(crate::MiraFunction::Native(function.into())),
        );
    }

    pub fn get(&self, name: &str) -> Option<MiraAny> {
        self.get_ref(name).cloned()
    }

    pub(crate) fn get_ref(&self, name: &str) -> Option<&MiraAny> {
        self.values.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

impl Default for MiraContext {
    fn default() -> Self {
        Self::new()
    }
}

use indexmap::IndexMap;

use crate::{MiraAny, MiraNativeFn};

/// Globals visible to a MiraScript execution.
#[derive(Clone)]
pub struct MiraContext {
    values: IndexMap<String, MiraAny>,
}

impl MiraContext {
    /// Create a context populated with the MiraScript standard library.
    pub fn new() -> Self {
        let mut context = Self {
            values: IndexMap::new(),
        };
        crate::standard_library::install(&mut context);
        context
    }

    /// Create a context without standard-library globals.
    pub fn empty() -> Self {
        Self {
            values: IndexMap::new(),
        }
    }

    /// Insert or replace a global value, returning the previous value.
    ///
    /// # Examples
    ///
    /// ```
    /// use mira_vm::{MiraAny, MiraContext};
    ///
    /// let mut context = MiraContext::empty();
    /// assert_eq!(context.insert("answer", 42), None);
    /// assert_eq!(context.get("answer"), Some(MiraAny::Number(42.0)));
    /// ```
    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: impl Into<MiraAny>,
    ) -> Option<MiraAny> {
        self.values.insert(name.into(), value.into())
    }

    /// Insert or replace a named native function.
    pub fn insert_fn(&mut self, name: impl Into<String>, function: impl Into<MiraNativeFn>) {
        self.insert(
            name,
            MiraAny::Function(crate::MiraFunction::Native(function.into())),
        );
    }

    /// Clone a global value by name.
    pub fn get(&self, name: &str) -> Option<MiraAny> {
        self.get_ref(name).cloned()
    }

    pub(crate) fn get_ref(&self, name: &str) -> Option<&MiraAny> {
        self.values.get(name)
    }

    /// Return whether a global name is defined.
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Iterate over global names in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

impl Default for MiraContext {
    fn default() -> Self {
        Self::new()
    }
}

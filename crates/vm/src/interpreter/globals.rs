use std::{cell::LazyCell, collections::HashMap};

use indexmap::IndexMap;

use crate::{MiraManageable, MiraNativeFn};

use super::*;

const GLOBALS_VEC_THRESHOLD: usize = 16;

enum GlobalData {
    Vec(Vec<(String, MiraAny)>),
    Map(HashMap<String, MiraAny>),
}

fn read_global(value: Option<MiraAny>) -> Option<MiraValue> {
    match value {
        Some(value) if !value.is_uninitialized() => Some(value.unwrap()),
        _ => None,
    }
}

impl GlobalData {
    fn get(&self, name: &str) -> Option<MiraValue> {
        let value = match self {
            GlobalData::Vec(vec) => vec.iter().find_map(|(existing_name, existing_value)| {
                if existing_name == name {
                    Some(existing_value)
                } else {
                    None
                }
            }),
            GlobalData::Map(map) => map.get(name),
        };
        read_global(value.cloned())
    }

    fn insert(&mut self, name: String, value: MiraValue) -> Option<MiraValue> {
        match self {
            GlobalData::Vec(vec) => {
                if let Some((_, existing_value)) = vec
                    .iter_mut()
                    .find(|(existing_name, _)| existing_name == &name)
                {
                    return read_global(Some(std::mem::replace(existing_value, value.into())));
                }
                vec.push((name, value.into()));
                if vec.len() > GLOBALS_VEC_THRESHOLD {
                    let map = vec.drain(..).collect::<HashMap<_, _>>();
                    *self = GlobalData::Map(map);
                }
                None
            }
            GlobalData::Map(map) => read_global(map.insert(name, value.into())),
        }
    }

    fn contains_key(&self, name: &str) -> bool {
        match self {
            GlobalData::Vec(vec) => vec.iter().any(|(existing_name, _)| existing_name == name),
            GlobalData::Map(map) => map.contains_key(name),
        }
    }

    fn len(&self) -> usize {
        match self {
            GlobalData::Vec(vec) => vec.len(),
            GlobalData::Map(map) => map.len(),
        }
    }
}

pub(crate) struct Globals {
    std: IndexMap<String, MiraAny>,
    context: GlobalData,
}

impl Globals {
    pub(super) fn new() -> Self {
        Self {
            std: IndexMap::new(),
            context: GlobalData::Vec(Vec::new()),
        }
    }

    fn insert_std(&mut self, name: &'static str, value: MiraValue) -> usize {
        debug_assert!(
            !self.std.contains_key(name),
            "standard-library global name collision: {name}"
        );
        debug_assert_eq!(
            self.context.len(),
            0,
            "standard-library globals must be inserted before any context globals"
        );
        debug_assert!(
            !value.is_uninitialized(),
            "standard-library globals must be initialized"
        );

        self.std.insert(name.to_string(), value.into());
        self.std.len() - 1
    }

    fn insert(&mut self, name: String, value: MiraValue) -> Option<MiraValue> {
        if let Some(std_slot) = self.std.get_mut(&name) {
            return read_global(Some(std::mem::replace(std_slot, value.into())));
        }
        self.context.insert(name, value)
    }

    pub fn get(&self, name: &str) -> Option<MiraValue> {
        self.context
            .get(name)
            .or_else(|| read_global(self.std.get(name).cloned()))
    }

    pub fn get_hint(&self, name: &str, index: Option<usize>) -> Option<MiraValue> {
        if let Some(index) = index
            && let Some((std_name, value)) = self.std.get_index(index)
        {
            debug_assert_eq!(
                std_name, name,
                "standard-library global name collision: expected {std_name}, got {name}"
            );
            read_global(Some(*value))
        } else {
            self.get(name)
        }
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.std.contains_key(name) || self.context.contains_key(name)
    }

    fn keys(&self) -> impl Iterator<Item = &str> {
        let mut keys = self.std.keys().map(String::as_str).collect::<Vec<_>>();
        match &self.context {
            GlobalData::Vec(vec) => keys.extend(vec.iter().map(|(k, _)| k.as_str())),
            GlobalData::Map(map) => keys.extend(map.keys().map(String::as_str)),
        }
        keys.into_iter()
    }
}

impl Runtime {
    /// Insert or replace a global after converting it into a Runtime value.
    pub fn insert_global(
        &mut self,
        name: impl Into<String>,
        value: impl Into<MiraManageable>,
    ) -> Result<Option<MiraValue>> {
        let value = self.insert(value)?;
        Ok(self.globals.insert(name.into(), value))
    }

    /// Insert a named native function into the global namespace.
    pub fn insert_fn(
        &mut self,
        name: impl Into<String>,
        function: impl Into<MiraNativeFn>,
    ) -> Result<Option<MiraValue>> {
        let name = name.into();
        let function = function.into().with_name(name.clone());
        let handle = self.insert_function(function)?;
        Ok(self.globals.insert(name, MiraValue::function(handle)))
    }

    /// Clone a global value by name.
    pub fn get_global(&self, name: &str) -> Option<MiraValue> {
        self.globals.get(name)
    }

    /// Return whether a global name is defined.
    pub fn contains_global(&self, name: &str) -> bool {
        self.globals.contains_key(name)
    }

    /// Remove a global value by name, returning the previous value if it existed.
    /// You shall call `take_*` to remove the value from the Runtime's arena if you want to reclaim its memory.
    pub fn remove_global(&mut self, name: &str) -> Option<MiraValue> {
        if !self.globals.contains_key(name) {
            None
        } else {
            self.globals
                .insert(name.to_string(), MiraValue::UNINITIALIZED)
        }
    }

    /// Iterate over global names in insertion order.
    pub fn global_names(&self) -> impl Iterator<Item = &str> {
        self.globals.keys()
    }

    pub(crate) fn insert_std(
        &mut self,
        name: &'static str,
        value: impl Into<MiraManageable>,
    ) -> usize {
        let value = self
            .insert(value)
            .expect("standard-library function allocation must fit in a fresh Runtime arena");
        self.globals.insert_std(name, value)
    }
}

thread_local! {
    static RUNTIME: LazyCell<IndexMap<String, ()>> = LazyCell::new(||{
        let runtime = Runtime::new();
        let std = runtime.globals.std;
        std.into_keys().map(|name| (name, ())).collect()
    });
}

pub(crate) fn std_slot(name: &str) -> Option<usize> {
    RUNTIME.with(|l| l.get_index_of(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_slot_returns_none_for_nonexistent_name() {
        let index = std_slot("nonexistent");
        assert!(index.is_none());
    }

    #[test]
    fn add_global() {
        let mut runtime = Runtime::new();

        assert!(!runtime.contains_global("foo"));

        // Insert a global and verify it can be retrieved.
        runtime.insert_global("foo", 42).unwrap();
        assert_eq!(
            runtime.get_global("foo").unwrap().as_number_unchecked(),
            42.0
        );

        // Remove the global and verify it is no longer present.
        let removed = runtime.remove_global("foo");
        assert_eq!(removed.unwrap().as_number_unchecked(), 42.0);
        assert!(runtime.get_global("foo").is_none());

        // Insert a new global with the same name and verify it can be retrieved.
        runtime.insert_global("foo", "Hello").unwrap();
        assert_eq!(
            runtime.get_global("foo").unwrap().as_str(&runtime).unwrap(),
            Some("Hello")
        );
    }

    #[test]
    fn add_std_global() {
        let mut runtime = Runtime::new();

        assert!(runtime.contains_global("sin"));

        // Insert a global and verify it can be retrieved.
        runtime.insert_global("sin", 42).unwrap();
        assert_eq!(
            runtime.get_global("sin").unwrap().as_number_unchecked(),
            42.0
        );

        // Remove the global and verify it is no longer present.
        let removed = runtime.remove_global("sin");
        assert_eq!(removed.unwrap().as_number_unchecked(), 42.0);
        assert!(runtime.get_global("sin").is_none());

        // Insert a new global with the same name and verify it can be retrieved.
        runtime.insert_global("sin", "Hello").unwrap();
        assert_eq!(
            runtime.get_global("sin").unwrap().as_str(&runtime).unwrap(),
            Some("Hello")
        );
    }

    #[test]
    fn remove_std() {
        let mut runtime = Runtime::new();

        assert!(runtime.contains_global("sin"));

        // Remove the global and verify it is no longer present.
        let removed = runtime.remove_global("sin");
        assert!(removed.unwrap().is_function());
        assert!(runtime.get_global("sin").is_none());

        // Re-add the global and verify it can be retrieved.
        runtime.insert_global("sin", removed.unwrap()).unwrap();
        assert!(runtime.get_global("sin").unwrap().is_function());

        assert_eq!(runtime.eval_unchecked("sin(0)").as_number_unchecked(), 0.0);
    }

    #[test]
    fn remove_nonexistent() {
        let mut runtime = Runtime::new();

        assert!(!runtime.contains_global("nonexistent"));

        // Remove the global and verify it is no longer present.
        let removed = runtime.remove_global("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn iter_global_names() {
        let mut runtime = Runtime::new();

        let names: Vec<_> = runtime.global_names().collect();
        assert!(names.contains(&"sin"));
        assert!(names.contains(&"cos"));

        runtime.insert_global("foo", 42).unwrap();
        runtime.insert_global("bar", "Hello").unwrap();

        let names: Vec<_> = runtime.global_names().collect();
        assert!(names.contains(&"sin"));
        assert!(names.contains(&"cos"));
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"bar"));
    }
}

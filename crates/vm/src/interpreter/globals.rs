use std::{cell::LazyCell, collections::HashMap};

use indexmap::IndexMap;

use crate::MiraManageable;

use super::*;

const GLOBALS_VEC_THRESHOLD: usize = 16;

enum GlobalData {
    Vec(Vec<(String, MiraValue)>),
    Map(HashMap<String, MiraValue>),
}

impl GlobalData {
    fn get(&self, name: &str) -> Option<&MiraValue> {
        match self {
            GlobalData::Vec(vec) => vec.iter().find_map(|(existing_name, existing_value)| {
                if existing_name == name {
                    Some(existing_value)
                } else {
                    None
                }
            }),
            GlobalData::Map(map) => map.get(name),
        }
    }

    fn insert(&mut self, name: String, value: MiraValue) -> Option<MiraValue> {
        match self {
            GlobalData::Vec(vec) => {
                if let Some((_, existing_value)) = vec
                    .iter_mut()
                    .find(|(existing_name, _)| existing_name == &name)
                {
                    return Some(std::mem::replace(existing_value, value));
                }
                vec.push((name, value));
                if vec.len() > GLOBALS_VEC_THRESHOLD {
                    let map = vec.drain(..).collect::<HashMap<_, _>>();
                    *self = GlobalData::Map(map);
                }
                None
            }
            GlobalData::Map(map) => map.insert(name, value),
        }
    }

    fn contains_key(&self, name: &str) -> bool {
        match self {
            GlobalData::Vec(vec) => vec.iter().any(|(existing_name, _)| existing_name == name),
            GlobalData::Map(map) => map.contains_key(name),
        }
    }
}

pub(crate) struct Globals {
    std: IndexMap<String, MiraValue>,
    context: GlobalData,
}

impl Globals {
    pub fn new() -> Self {
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
        self.std.insert(name.to_string(), value);
        self.std.len() - 1
    }

    pub fn insert(&mut self, name: String, value: MiraValue) -> Option<MiraValue> {
        if let Some(std_slot) = self.std.get_mut(&name) {
            return Some(std::mem::replace(std_slot, value));
        }
        self.context.insert(name, value)
    }

    pub fn get(&self, name: &str) -> Option<MiraValue> {
        self.context
            .get(name)
            .or_else(|| self.std.get(name))
            .copied()
    }

    pub fn get_hint(&self, name: &str, index: Option<usize>) -> Option<MiraValue> {
        if let Some(index) = index
            && let Some((std_name, value)) = self.std.get_index(index)
            && std_name == name
        {
            Some(*value)
        } else {
            self.get(name)
        }
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.std.contains_key(name) || self.context.contains_key(name)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        let mut keys = self.std.keys().map(String::as_str).collect::<Vec<_>>();
        match &self.context {
            GlobalData::Vec(vec) => keys.extend(vec.iter().map(|(k, _)| k.as_str())),
            GlobalData::Map(map) => keys.extend(map.keys().map(String::as_str)),
        }
        keys.into_iter()
    }
}

impl Runtime {
    pub(crate) fn insert_std(
        &mut self,
        name: &'static str,
        value: impl Into<MiraManageable>,
    ) -> Result<usize> {
        let value = self.insert(value)?;
        Ok(self.globals.insert_std(name, value))
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

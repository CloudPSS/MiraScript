use std::fmt;
use std::rc::Rc;

use indexmap::IndexMap;

use super::MiraAny;

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct ScriptModule {
    pub execution: u64,
    pub frame: usize,
    pub exports: IndexMap<String, usize>,
    pub name: Rc<str>,
}

#[derive(Clone)]
pub enum MiraModule {
    Native {
        name: Rc<str>,
        values: Rc<IndexMap<String, MiraAny>>,
    },
    #[doc(hidden)]
    Script(Rc<ScriptModule>),
}

impl MiraModule {
    pub fn new(name: impl Into<String>, values: IndexMap<String, MiraAny>) -> Self {
        Self::Native {
            name: Rc::from(name.into()),
            values: Rc::new(values),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Native { name, .. } => name,
            Self::Script(module) => &module.name,
        }
    }

    pub fn keys(&self) -> Vec<String> {
        match self {
            Self::Native { values, .. } => values.keys().cloned().collect(),
            Self::Script(module) => module.exports.keys().cloned().collect(),
        }
    }

    pub fn get_native(&self, key: &str) -> Option<MiraAny> {
        match self {
            Self::Native { values, .. } => values.get(key).cloned(),
            Self::Script(_) => None,
        }
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native { values: a, .. }, Self::Native { values: b, .. }) => Rc::ptr_eq(a, b),
            (Self::Script(a), Self::Script(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Debug for MiraModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiraModule")
            .field("name", &self.name())
            .field("keys", &self.keys())
            .finish()
    }
}

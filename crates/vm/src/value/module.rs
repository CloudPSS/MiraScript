use std::fmt;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::interpreter::{ExecutionId, FrameId};

use super::MiraAny;

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct ScriptModule {
    pub(crate) execution: ExecutionId,
    pub frame: FrameId,
    pub exports: IndexMap<String, usize>,
    pub name: Rc<str>,
}

#[derive(Clone)]
/// A named collection of MiraScript-visible values.
pub enum MiraModule {
    /// A host-created module that may safely outlive an execution.
    Native {
        /// Module name used in diagnostics and debug output.
        name: Rc<str>,
        /// Exported values keyed by field name.
        values: Rc<IndexMap<String, MiraAny>>,
    },
    #[doc(hidden)]
    Script(Rc<ScriptModule>),
}

impl MiraModule {
    /// Create a native module from an ordered map of exported values.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexmap::IndexMap;
    /// use mirascript_vm::{MiraAny, MiraModule};
    ///
    /// let module = MiraModule::new(
    ///     "constants",
    ///     IndexMap::from([("answer".into(), MiraAny::Number(42.0))]),
    /// );
    /// assert_eq!(module.name(), "constants");
    /// assert_eq!(module.get_native("answer"), Some(MiraAny::Number(42.0)));
    /// ```
    pub fn new(name: impl Into<String>, values: IndexMap<String, MiraAny>) -> Self {
        Self::Native {
            name: Rc::from(name.into()),
            values: Rc::new(values),
        }
    }

    /// Return the module name.
    pub fn name(&self) -> &str {
        match self {
            Self::Native { name, .. } => name,
            Self::Script(module) => &module.name,
        }
    }

    /// Return exported field names in insertion order.
    pub fn keys(&self) -> Vec<String> {
        match self {
            Self::Native { values, .. } => values.keys().cloned().collect(),
            Self::Script(module) => module.exports.keys().cloned().collect(),
        }
    }

    /// Clone an export from a native module.
    ///
    /// Script modules return `None`; they are resolved by the active runtime.
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

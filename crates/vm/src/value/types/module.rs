use std::any::Any;

use indexmap::IndexMap;

use crate::{
    MiraError, Result, Runtime, RuntimeErrorKind,
    value::{MiraHandle, MiraManageable},
};

use super::{MiraValue, MiraValueKind, value::ValueTag};

impl MiraValue {
    /// Create a `MiraValue` representing a module value.
    #[inline]
    pub const fn module<T: MiraModule + ?Sized>(value: MiraHandle<T>) -> Self {
        Self::handle(ValueTag::Module, value.erase_module())
    }

    /// Check whether this value is a module value.
    #[inline]
    pub const fn is_module(&self) -> bool {
        matches!(self.tag(), Some(ValueTag::Module))
    }

    /// Return the module handle, or `None` for another value type.
    #[inline]
    pub fn as_module(&self) -> Option<MiraHandle<dyn MiraModule>> {
        match self.kind() {
            MiraValueKind::Module(value) => Some(value),
            _ => None,
        }
    }
}

/// A named collection of MiraScript-visible values.
pub trait MiraModule: Any {
    /// Return the module name shown in diagnostics.
    fn name(&self) -> &str;

    /// Return the number of exports.
    fn len(&self) -> usize;

    /// Find an export's iteration index.
    fn index_of(&self, key: &str) -> Option<usize>;

    /// Read an export name by index.
    fn key(&self, index: usize) -> Result<&str>;

    /// Read an export value by index.
    fn get(
        &self,
        self_handle: MiraHandle<dyn MiraModule>,
        runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable>;

    /// Return whether this module has no exports.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[doc(hidden)]
    fn resolve<'a>(&self, runtime: &'a Runtime) -> Result<Option<&'a dyn MiraModule>> {
        let _ = runtime;
        Ok(None)
    }
}

pub(crate) struct MiraMapModule {
    name: String,
    values: IndexMap<String, MiraValue>,
}

impl MiraMapModule {
    pub(crate) fn new(name: impl Into<String>, values: IndexMap<String, MiraValue>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }
}

impl MiraModule for MiraMapModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.values.get_index_of(key)
    }

    fn key(&self, index: usize) -> Result<&str> {
        self.values
            .get_index(index)
            .map(|(key, _)| key.as_str())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: MiraHandle<dyn MiraModule>,
        _runtime: &Runtime,
        index: usize,
    ) -> Result<MiraManageable> {
        self.values
            .get_index(index)
            .map(|(_, value)| (*value).into())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

pub(crate) fn map_module(
    name: impl Into<String>,
    values: IndexMap<String, MiraValue>,
) -> MiraManageable {
    MiraManageable::from_module(MiraMapModule::new(name, values))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_missing_index(error: &MiraError) {
        assert!(matches!(
            error,
            MiraError::Runtime {
                kind: RuntimeErrorKind::MissingIndexOrField,
                ..
            }
        ));
    }

    fn assert_value(value: MiraManageable, expected: MiraValue) {
        assert!(matches!(value, MiraManageable::Value(value) if value == expected));
    }

    #[test]
    fn map_modules_expose_metadata_values_and_bounds() {
        let mut runtime = Runtime::new();
        let mut values = IndexMap::new();
        values.insert("answer".into(), MiraValue::from(42));
        let module = runtime
            .insert_module(MiraMapModule::new("test", values))
            .unwrap();
        let value = MiraValue::module(module);

        assert!(value.is_module());
        assert_eq!(value.as_module(), Some(module.erase_module()));
        assert!(MiraValue::NIL.as_module().is_none());

        let value = runtime.get_module(module).unwrap();
        assert_eq!(value.name(), "test");
        assert_eq!(value.len(), 1);
        assert!(!value.is_empty());
        assert_eq!(value.index_of("answer"), Some(0));
        assert_eq!(value.index_of("missing"), None);
        assert_eq!(value.key(0).unwrap(), "answer");
        assert_missing_index(value.key(1).unwrap_err().as_ref());
        assert_value(
            value.get(module.erase_module(), &runtime, 0).unwrap(),
            42.into(),
        );
        assert_missing_index(
            value
                .get(module.erase_module(), &runtime, 1)
                .err()
                .unwrap()
                .as_ref(),
        );
        assert!(MiraModule::resolve(value, &runtime).unwrap().is_none());

        assert!(MiraMapModule::new("empty", IndexMap::new()).is_empty());
    }
}

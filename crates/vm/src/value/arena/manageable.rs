use std::rc::Rc;

use super::{MiraArray, MiraFunction, MiraModule, MiraRecord, MiraValue};

/// An owned value waiting to be inserted into a Runtime arena.
pub enum MiraManageable {
    /// A value whose payload is already inline or already belongs to this Runtime.
    Value(MiraValue),
    /// An owned string.
    String(String),
    /// An owned array implementation.
    Array(Box<dyn MiraArray>),
    /// An owned record implementation.
    Record(Box<dyn MiraRecord>),
    /// An owned callable implementation.
    Function(Rc<dyn MiraFunction>),
    /// An owned module implementation.
    Module(Box<dyn MiraModule>),
}

impl From<MiraValue> for MiraManageable {
    fn from(value: MiraValue) -> Self {
        Self::Value(value)
    }
}

impl From<String> for MiraManageable {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MiraManageable {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl MiraManageable {
    /// Wrap an array implementation for insertion into a Runtime.
    pub fn from_array(value: impl MiraArray) -> Self {
        Self::Array(Box::new(value))
    }

    /// Wrap a record implementation for insertion into a Runtime.
    pub fn from_record(value: impl MiraRecord) -> Self {
        Self::Record(Box::new(value))
    }

    /// Wrap a function implementation for insertion into a Runtime.
    pub fn from_function(value: impl MiraFunction) -> Self {
        Self::Function(Rc::new(value))
    }

    /// Wrap a module implementation for insertion into a Runtime.
    pub fn from_module(value: impl MiraModule) -> Self {
        Self::Module(Box::new(value))
    }

    /// Build a simple named module from already materialized Runtime values.
    pub fn map_module(
        name: impl Into<String>,
        values: indexmap::IndexMap<String, MiraValue>,
    ) -> Self {
        crate::value::types::map_module(name, values)
    }
}

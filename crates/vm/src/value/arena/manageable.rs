use std::rc::Rc;

use crate::{MiraArray, MiraFunction, MiraModule, MiraRecord, MiraType, MiraValue};

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

impl MiraManageable {
    /// Return the MiraScript type of this value.
    pub fn value_type(&self) -> MiraType {
        match self {
            Self::Value(value) => value.value_type(),
            Self::String(_) => MiraType::String,
            Self::Array(_) => MiraType::Array,
            Self::Record(_) => MiraType::Record,
            Self::Function(_) => MiraType::Function,
            Self::Module(_) => MiraType::Module,
        }
    }
}

impl<T> From<T> for MiraManageable
where
    T: Into<MiraValue>,
{
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

impl TryFrom<MiraManageable> for MiraValue {
    type Error = ();

    fn try_from(value: MiraManageable) -> Result<Self, Self::Error> {
        match value {
            MiraManageable::Value(value) => Ok(value),
            _ => Err(()),
        }
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

#[cfg(test)]
mod tests {
    use crate::{Runtime, value::map_module};

    use super::*;

    #[test]
    fn test_value() {
        let value: MiraManageable = MiraValue::number(42.0).into();
        assert_eq!(value.value_type(), MiraType::Number);
        let mut runtime = Runtime::new();
        runtime.insert_global("test_value", value).unwrap();
        let result = runtime.eval_unchecked("test_value + 1");
        assert_eq!(result.as_number_unchecked(), 43.0);
    }

    #[test]
    fn test_string() {
        let value: MiraManageable = "Hello".into();
        assert_eq!(value.value_type(), MiraType::String);
        let mut runtime = Runtime::new();
        runtime.insert_global("test_string", value).unwrap();
        let result = runtime.eval_unchecked("'$test_string World!'");
        assert_eq!(result.as_str(&runtime).unwrap(), Some("Hello World!"));
    }

    #[test]
    fn test_array() {
        let value: MiraManageable =
            MiraManageable::from_array(vec![MiraValue::number(1.0), MiraValue::number(2.0)]);
        assert_eq!(value.value_type(), MiraType::Array);
        let mut runtime = Runtime::new();
        runtime.insert_global("test_array", value).unwrap();
        let result = runtime.eval_unchecked("test_array[0] + test_array[1]");
        assert_eq!(result.as_number_unchecked(), 3.0);
    }

    #[test]
    fn test_record() {
        let value: MiraManageable = MiraManageable::from_record(std::collections::HashMap::<
            String,
            MiraValue,
        >::from_iter([
            ("a".to_string(), MiraValue::number(1.0)),
            ("b".to_string(), MiraValue::number(2.0)),
        ]));
        assert_eq!(value.value_type(), MiraType::Record);
        let mut runtime = Runtime::new();
        runtime.insert_global("test_record", value).unwrap();
        let result = runtime.eval_unchecked("test_record.a + test_record.b");
        assert_eq!(result.as_number_unchecked(), 3.0);
    }

    #[test]
    fn test_function() {
        let value: MiraManageable = MiraManageable::from_function(
            |_: &mut Runtime, args: &[MiraValue]| -> crate::Result<MiraManageable> {
                Ok(MiraValue::number(args.len() as f64).into())
            },
        );
        assert_eq!(value.value_type(), MiraType::Function);
        let mut runtime = Runtime::new();
        runtime.insert_global("test_function", value).unwrap();
        let result = runtime.eval_unchecked("test_function(1, 2, 3)");
        assert_eq!(result.as_number_unchecked(), 3.0);
    }

    #[test]
    fn test_module() {
        let value: MiraManageable = map_module("test", indexmap::IndexMap::new());
        assert_eq!(value.value_type(), MiraType::Module);
        let mut runtime = Runtime::new();
        runtime.insert_global("test_module", value).unwrap();
        let result = runtime.eval_unchecked("test_module");
        assert!(result.is_module());
    }
}

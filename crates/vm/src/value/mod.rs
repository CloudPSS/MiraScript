mod arena;
mod bridge;
mod convert;
mod function;
mod indirect;
mod module;
mod shared;
mod types;

use std::fmt;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::{MiraError, Result, interpreter::ExecutionId};

use bridge::{ArrayObject, RecordObject};
pub use bridge::{MiraArray, MiraBridge, MiraRecord};
pub(crate) use function::NativeRuntime;
pub use function::{MiraCallContext, MiraFunction, MiraNativeFn};
pub use indirect::MiraIndirect;
pub use module::MiraModule;
pub(crate) use module::ScriptModule;
pub use shared::MiraShared;
pub use types::*;

/// A value understood by the Rust VM.
///
/// Scalar payloads are stored inline. All other payloads use shared
/// copy-on-write indirection so that every value occupies exactly 16 bytes.
#[derive(Clone, Default)]
pub enum MiraAny {
    #[doc(hidden)]
    Uninitialized,
    /// The MiraScript `nil` value.
    #[default]
    Nil,
    /// A boolean value.
    Boolean(bool),
    /// A double-precision numeric value.
    Number(f64),
    /// An owned UTF-8 string.
    String(MiraIndirect<String>),
    /// An owned MiraScript array.
    Array(MiraIndirect<Vec<MiraAny>>),
    /// An owned MiraScript record with insertion-ordered keys.
    Record(MiraIndirect<IndexMap<String, MiraAny>>),
    /// A callable native or script function.
    Function(MiraIndirect<MiraFunction>),
    /// A native or execution-scoped module.
    Module(MiraIndirect<MiraModule>),
    #[doc(hidden)]
    RustRecord(MiraIndirect<Rc<dyn RecordObject>>),
    #[doc(hidden)]
    RustArray(MiraIndirect<Rc<dyn ArrayObject>>),
}

const _: () = assert!(std::mem::size_of::<MiraAny>() == 16);

impl MiraAny {
    /// Return the MiraScript type name for this value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Uninitialized | Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) | Self::RustArray(_) => "array",
            Self::Record(_) | Self::RustRecord(_) => "record",
            Self::Function(_) => "function",
            Self::Module(_) => "module",
        }
    }

    /// Wrap a Rust record in a new [`MiraShared`] allocation.
    pub fn from_record<T: MiraRecord>(value: T) -> Self {
        Self::from_record_shared(MiraShared::new(value))
    }

    /// Wrap an existing shared Rust record as a live read-only view.
    pub fn from_record_shared<T: MiraRecord>(value: MiraShared<T>) -> Self {
        let value: Rc<dyn RecordObject> = Rc::new(value);
        Self::RustRecord(value.into())
    }

    /// Wrap a Rust array in a new [`MiraShared`] allocation.
    pub fn from_array<T: MiraArray>(value: T) -> Self {
        Self::from_array_shared(MiraShared::new(value))
    }

    /// Wrap an existing shared Rust array as a live read-only view.
    pub fn from_array_shared<T: MiraArray>(value: MiraShared<T>) -> Self {
        let value: Rc<dyn ArrayObject> = Rc::new(value);
        Self::RustArray(value.into())
    }

    /// Return whether this value is initialized and safe to expose to host code.
    pub fn is_initialized(&self) -> bool {
        !matches!(self, Self::Uninitialized)
    }

    pub(crate) fn into_element(self) -> Result<Self> {
        match self {
            Self::Uninitialized => Err(MiraError::runtime("Uninitialized value")),
            Self::Function(_) | Self::Module(_) => Ok(Self::Nil),
            value => Ok(value),
        }
    }

    pub(crate) fn contains_script_reference(&self, execution: ExecutionId) -> bool {
        match self {
            Self::Function(function) => matches!(
                function.as_ref(),
                MiraFunction::Script {
                execution: owner, ..
                } if *owner == execution
            ),
            Self::Module(module) => matches!(
                module.as_ref(),
                MiraModule::Script(module) if module.execution == execution
            ),
            Self::Array(values) => values
                .iter()
                .any(|value| value.contains_script_reference(execution)),
            Self::Record(values) => values
                .values()
                .any(|value| value.contains_script_reference(execution)),
            _ => false,
        }
    }

    pub(crate) fn record_keys(&self) -> Result<Option<Vec<String>>> {
        match self {
            Self::Record(record) => Ok(Some(record.keys().cloned().collect())),
            Self::RustRecord(record) => record.keys().map(Some),
            _ => Ok(None),
        }
    }

    pub(crate) fn record_get(&self, key: &str) -> Result<Option<MiraAny>> {
        match self {
            Self::Record(record) => Ok(record.get(key).cloned()),
            Self::RustRecord(record) => record.get(key),
            _ => Ok(None),
        }
    }

    pub(crate) fn array_len(&self) -> Result<Option<usize>> {
        match self {
            Self::Array(array) => Ok(Some(array.len())),
            Self::RustArray(array) => array.len().map(Some),
            _ => Ok(None),
        }
    }

    pub(crate) fn array_get(&self, index: usize) -> Result<Option<MiraAny>> {
        match self {
            Self::Array(array) => Ok(array.get(index).cloned()),
            Self::RustArray(array) => array.get(index),
            _ => Ok(None),
        }
    }
}

fn same_record(a: &MiraAny, b: &MiraAny) -> bool {
    let Ok(Some(a_keys)) = a.record_keys() else {
        return false;
    };
    let Ok(Some(b_keys)) = b.record_keys() else {
        return false;
    };
    if a_keys.len() != b_keys.len() {
        return false;
    }
    a_keys.into_iter().all(|key| {
        if !b_keys.contains(&key) {
            return false;
        }
        match (
            crate::operations::get(a, &key),
            crate::operations::get(b, &key),
        ) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    })
}

fn same_array(a: &MiraAny, b: &MiraAny) -> bool {
    let (Ok(Some(a_len)), Ok(Some(b_len))) = (a.array_len(), b.array_len()) else {
        return false;
    };
    if a_len != b_len {
        return false;
    }
    (0..a_len).all(|index| match (a.array_get(index), b.array_get(index)) {
        (Ok(a), Ok(b)) => a.unwrap_or(MiraAny::Nil) == b.unwrap_or(MiraAny::Nil),
        _ => false,
    })
}

impl PartialEq for MiraAny {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Uninitialized, Self::Uninitialized) | (Self::Nil, Self::Nil) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Array(_), Self::Array(_))
            | (Self::Array(_), Self::RustArray(_))
            | (Self::RustArray(_), Self::Array(_))
            | (Self::RustArray(_), Self::RustArray(_)) => same_array(self, other),
            (Self::Record(_), Self::Record(_))
            | (Self::Record(_), Self::RustRecord(_))
            | (Self::RustRecord(_), Self::Record(_))
            | (Self::RustRecord(_), Self::RustRecord(_)) => same_record(self, other),
            (Self::Function(a), Self::Function(b)) => a.same(b),
            (Self::Module(a), Self::Module(b)) => a.same(b),
            _ => false,
        }
    }
}

impl fmt::Debug for MiraAny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uninitialized => f.write_str("Uninitialized"),
            Self::Nil => f.write_str("Nil"),
            Self::Boolean(value) => f.debug_tuple("Boolean").field(value).finish(),
            Self::Number(value) => f.debug_tuple("Number").field(value).finish(),
            Self::String(value) => f.debug_tuple("String").field(value).finish(),
            Self::Array(value) => f.debug_tuple("Array").field(value).finish(),
            Self::Record(value) => f.debug_tuple("Record").field(value).finish(),
            Self::Function(value) => f.debug_tuple("Function").field(value).finish(),
            Self::Module(value) => f.debug_tuple("Module").field(value).finish(),
            Self::RustRecord(value) => f
                .debug_struct("RustRecord")
                .field("tag", &value.tag())
                .field("identity", &value.identity())
                .finish(),
            Self::RustArray(value) => f
                .debug_struct("RustArray")
                .field("tag", &value.tag())
                .field("identity", &value.identity())
                .finish(),
        }
    }
}

use crate::{MiraType, value::arena::MiraHandle};

use super::{MiraArray, MiraExtern, MiraFunction, MiraModule, MiraRecord};

/// A compact value understood by the Rust VM.
///
/// Scalar payloads are stored inline. Runtime-owned payloads are represented by
/// checked handles into a [`crate::Runtime`] arena.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MiraValue {
    /// The MiraScript `nil` value.
    #[default]
    Nil,
    /// A boolean value.
    Boolean(bool),
    /// A double-precision numeric value.
    Number(f64),
    /// A compile-time static UTF-8 string stored as a thin pointer.
    StaticStr(&'static &'static str),
    /// A runtime-owned UTF-8 string.
    String(MiraHandle<String>),
    /// A MiraScript array.
    Array(MiraHandle<dyn MiraArray>),
    /// A MiraScript record.
    Record(MiraHandle<dyn MiraRecord>),
    /// A MiraScript function.
    Function(MiraHandle<dyn MiraFunction>),
    /// A MiraScript module.
    Module(MiraHandle<dyn MiraModule>),
    /// Reserved external value placeholder.
    #[doc(hidden)]
    Extern(MiraHandle<dyn MiraExtern>),
}

const _: () = assert!(std::mem::size_of::<MiraValue>() == 16);
const _: () = assert!(std::mem::size_of::<Option<MiraValue>>() == 16);

impl MiraValue {
    /// Return this value's MiraScript category.
    #[inline]
    pub const fn value_type(self) -> MiraType {
        match self {
            Self::Nil => MiraType::Nil,
            Self::Boolean(_) => MiraType::Boolean,
            Self::Number(_) => MiraType::Number,
            Self::StaticStr(_) | Self::String(_) => MiraType::String,
            Self::Array(_) => MiraType::Array,
            Self::Record(_) => MiraType::Record,
            Self::Function(_) => MiraType::Function,
            Self::Module(_) => MiraType::Module,
            Self::Extern(_) => MiraType::Extern,
        }
    }

    /// Return the MiraScript type name for this value.
    #[inline]
    pub const fn type_name(self) -> &'static str {
        self.value_type().name()
    }
}

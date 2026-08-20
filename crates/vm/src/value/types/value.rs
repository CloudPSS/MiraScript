use crate::value::arena::MiraHandle;

use super::{MiraArray, MiraExtern, MiraFunction, MiraModule, MiraRecord};

/// A value understood by the Rust VM.
///
/// Scalar payloads are stored inline. All other payloads
/// are stored in a shared arena.
#[derive(Clone, Debug, Default, PartialEq, strum::EnumIs)]
pub enum MiraValue {
    /// The MiraScript `nil` value.
    #[default]
    Nil,
    /// A boolean value.
    Boolean(bool),
    /// A double-precision numeric value.
    Number(f64),
    /// A compile-time static UTF-8 string.
    #[strum(disabled)]
    Str(&'static &'static str),
    /// A UTF-8 string.
    #[strum(disabled)]
    String(MiraHandle<String>),
    /// A MiraScript array.
    Array(MiraHandle<dyn MiraArray>),
    /// A MiraScript record with insertion-ordered keys.
    Record(MiraHandle<dyn MiraRecord>),
    /// A MiraScript function.
    Function(MiraHandle<dyn MiraFunction>),
    /// A MiraScript module.
    Module(MiraHandle<dyn MiraModule>),
    /// A MiraScript external value.
    Extern(MiraHandle<dyn MiraExtern>),
}
const _: () = assert!(std::mem::size_of::<MiraValue>() == 16);

impl MiraValue {
    #[inline]
    /// Return the MiraScript type name for this value.
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) | Self::Str(_) => "string",
            Self::Array(_) => "array",
            Self::Record(_) => "record",
            Self::Function(_) => "function",
            Self::Module(_) => "module",
            Self::Extern(_) => "extern",
        }
    }
}

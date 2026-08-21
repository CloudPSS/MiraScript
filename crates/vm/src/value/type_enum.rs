use std::fmt;

/// A MiraScript runtime value category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiraType {
    /// The `nil` value.
    Nil,
    /// A boolean value.
    Boolean,
    /// A numeric value.
    Number,
    /// A string value.
    String,
    /// An array value.
    Array,
    /// A record value.
    Record,
    /// A callable function.
    Function,
    /// A module value.
    Module,
    /// A reserved external value.
    Extern,
}

impl MiraType {
    /// Return the MiraScript type name for this category.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
            Self::Array => "array",
            Self::Record => "record",
            Self::Function => "function",
            Self::Module => "module",
            Self::Extern => "extern",
        }
    }
}

impl fmt::Display for MiraType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

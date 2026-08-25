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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_display() {
        assert_eq!(MiraType::Nil.to_string(), "nil");
        assert_eq!(MiraType::Boolean.to_string(), "boolean");
        assert_eq!(MiraType::Number.to_string(), "number");
        assert_eq!(MiraType::String.to_string(), "string");
        assert_eq!(MiraType::Array.to_string(), "array");
        assert_eq!(MiraType::Record.to_string(), "record");
        assert_eq!(MiraType::Function.to_string(), "function");
        assert_eq!(MiraType::Module.to_string(), "module");
        assert_eq!(MiraType::Extern.to_string(), "extern");
    }
}

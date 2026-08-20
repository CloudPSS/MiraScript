use super::MiraValue;
use crate::{Result, Runtime, value::arena::MiraManageable};

impl MiraValue {
    #[inline]
    /// Return whether this value is a static or arena-managed string.
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_) | Self::StaticString(_))
    }

    #[inline]
    /// Borrow this value's string payload from its owning Runtime.
    pub fn as_string<'s>(&'s self, runtime: &'s Runtime) -> Result<Option<&'s str>> {
        match self {
            Self::String(handle) => runtime.get_string(*handle).map(Some),
            Self::StaticString(value) => Ok(Some(value)),
            _ => Ok(None),
        }
    }

    #[inline]
    /// Allocate an owned string in a Runtime and return its value handle.
    pub fn new_string(value: impl Into<String>, runtime: &mut Runtime) -> Result<Self> {
        runtime.insert_string(value).map(Self::String)
    }
}

impl From<&'static &'static str> for MiraValue {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        Self::StaticString(value)
    }
}

impl From<&'static &'static str> for MiraManageable {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        std::convert::Into::<MiraValue>::into(value).into()
    }
}

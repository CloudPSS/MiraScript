use crate::{MiraHandle, MiraType, Result, Runtime};

use super::{
    MiraValue, MiraValueKind, Payload,
    value::{RawValue, ValueTag},
};

impl MiraValue {
    /// Create a [`MiraValue`] representing a static string.
    #[inline]
    pub fn str(value: &'static &'static str) -> Self {
        Self::from_raw(RawValue::from_tagged(
            ValueTag::StaticStr,
            Payload::from_address(std::ptr::from_ref(value)).to_bytes(),
        ))
    }

    /// Create a [`MiraValue`] representing a string value.
    #[inline]
    pub const fn string(value: MiraHandle<String>) -> Self {
        Self::handle(ValueTag::String, value)
    }

    /// Return whether this value is a static or arena-managed string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self.value_type(), MiraType::String)
    }

    /// Borrow this value's string payload from its owning Runtime.
    #[inline]
    pub fn as_str<'s>(&self, runtime: &'s Runtime) -> Result<Option<&'s str>> {
        match self.kind() {
            MiraValueKind::String(handle) => runtime.get_string(handle).map(Some),
            MiraValueKind::StaticStr(value) => Ok(Some(value)),
            _ => Ok(None),
        }
    }

    /// Borrow this value's string payload from its owning Runtime.
    #[inline]
    pub fn as_str_unchecked<'s>(&self, runtime: &'s Runtime) -> &'s str {
        self.as_str(runtime)
            .expect("Runtime error")
            .expect("Value is not a string")
    }
}

impl From<&'static &'static str> for MiraValue {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        Self::str(value)
    }
}

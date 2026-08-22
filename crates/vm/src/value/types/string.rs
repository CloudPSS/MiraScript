use super::{MiraValue, MiraValueKind};
use crate::{Result, Runtime, value::arena::MiraManageable};

impl MiraValue {
    /// Create a `MiraValue` representing a static string.
    #[inline]
    pub fn str(value: &'static &'static str) -> Self {
        Self::boxed_static_str(value)
    }

    /// Allocate an owned string in a Runtime and return its value handle.
    #[inline]
    pub fn new_string(value: impl Into<String>, runtime: &mut Runtime) -> Result<Self> {
        runtime.insert_string(value).map(Self::from_string_handle)
    }

    /// Return whether this value is a static or arena-managed string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(
            self.kind(),
            MiraValueKind::String(_) | MiraValueKind::StaticStr(_)
        )
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
}

impl From<&'static &'static str> for MiraValue {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        Self::str(value)
    }
}

impl From<&'static &'static str> for MiraManageable {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        std::convert::Into::<MiraValue>::into(value).into()
    }
}

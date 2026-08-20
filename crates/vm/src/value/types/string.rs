use super::MiraValue;
use crate::{interpreter::Runtime, value::arena::MiraManageable};

impl MiraValue {
    #[inline]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_) | Self::Str(_))
    }

    #[inline]
    pub fn as_string<'s>(&'s self, runtime: &'s Runtime) -> Option<&str> {
        match self {
            Self::String(handle) => runtime.get_string(*handle),
            Self::Str(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn new_string(value: impl Into<String>, runtime: &mut Runtime) -> Self {
        let handle = runtime.insert_string(value);
        Self::String(handle)
    }
}

impl From<&'static &'static str> for MiraValue {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        Self::Str(value)
    }
}

impl From<&'static &'static str> for MiraManageable {
    #[inline]
    fn from(value: &'static &'static str) -> Self {
        std::convert::Into::<MiraValue>::into(value).into()
    }
}

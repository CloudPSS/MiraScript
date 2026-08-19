use crate::{MiraError, Result};

use super::MiraValue;

impl MiraValue {
    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }
}

impl From<bool> for MiraValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl TryFrom<MiraValue> for bool {
    type Error = Box<MiraError>;

    #[inline]
    fn try_from(value: MiraValue) -> Result<Self> {
        value
            .as_boolean()
            .ok_or_else(|| MiraError::conversion("bool", &value))
    }
}

use crate::value::arena::MiraManageable;

use super::MiraValue;
pub use MiraValue::Nil;

impl<T: Into<MiraValue>> From<Option<T>> for MiraValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Nil,
        }
    }
}

impl<T: Into<MiraManageable>> From<Option<T>> for MiraManageable {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Nil.into(),
        }
    }
}

use crate::MiraType;

use super::{MiraValue, value::ValueTag};

impl MiraValue {
    /// A [`MiraValue`] representing the MiraScript `nil` value.
    pub const NIL: MiraValue = Self::empty(ValueTag::Nil);

    /// Check whether this value is the MiraScript `nil` value.
    #[inline]
    pub const fn is_nil(&self) -> bool {
        matches!(self.value_type(), MiraType::Nil)
    }
}

impl<T: Into<MiraValue>> From<Option<T>> for MiraValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => MiraValue::NIL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nil() {
        assert_eq!(MiraValue::NIL, MiraValue::NIL);
        assert!(MiraValue::NIL.is_nil());
        assert!(!MiraValue::number(42.0).is_nil());
    }

    #[test]
    fn test_option_into_value() {
        let value: MiraValue = Some(42).into();
        assert_eq!(value, MiraValue::number(42.0));

        let value: MiraValue = None::<i32>.into();
        assert_eq!(value, MiraValue::NIL);
    }
}

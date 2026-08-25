use super::{MiraValue, value::RawValue};
use crate::value::arena::MiraManageable;
use crate::{MiraError, Result};

impl MiraValue {
    /// Create a `MiraValue` representing a numeric value.
    #[inline]
    pub const fn number(value: f64) -> Self {
        Self::from_raw(RawValue::number(value))
    }

    /// Check whether this value is a numeric value.
    #[inline]
    pub const fn is_number(&self) -> bool {
        self.tag().is_none()
    }

    /// Return the inline numeric payload, or `None` for another value type.
    #[inline]
    pub const fn as_number(&self) -> Option<f64> {
        if self.is_number() {
            Some(f64::from_bits(self.raw().0))
        } else {
            None
        }
    }
}

macro_rules! number_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl From<$ty> for MiraValue {
            fn from(value: $ty) -> Self {
                Self::number(value as f64)
            }
        }
        impl From<$ty> for MiraManageable {
            fn from(value: $ty) -> Self {
                std::convert::Into::<MiraValue>::into(value as f64).into()
            }
        }

        impl TryFrom<MiraManageable> for $ty {
            type Error = Box<MiraError>;

            fn try_from(value: MiraManageable) -> Result<Self> {
                let MiraManageable::Value(value) = value else {
                    return Err(MiraError::conversion_type(stringify!($ty), value.value_type()));
                };
                Self::try_from(value)
            }
        }
    )* };
}

number_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

macro_rules! integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraValue> for $ty {
            type Error = Box<MiraError>;

            fn try_from(value: MiraValue) -> Result<Self> {
                let Some(number) = value.as_number() else {
                    return Err(MiraError::conversion_type(stringify!($ty), value.value_type()));
                };
                if !number.is_finite()
                    || number.trunc() != number
                    || number < (<$ty>::MIN as f64)
                    || number > (<$ty>::MAX as f64)
                {
                    return Err(MiraError::conversion_number(stringify!($ty), number));
                }
                Ok(number as $ty)
            }
        }
    )* };
}
integer_try_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl TryFrom<MiraValue> for f64 {
    type Error = Box<MiraError>;

    fn try_from(value: MiraValue) -> Result<Self> {
        value
            .as_number()
            .ok_or_else(|| MiraError::conversion_type("f64", value.value_type()))
    }
}

impl TryFrom<MiraValue> for f32 {
    type Error = Box<MiraError>;

    fn try_from(value: MiraValue) -> Result<Self> {
        let value = f64::try_from(value)?;
        if value.is_finite() && (value < f32::MIN as f64 || value > f32::MAX as f64) {
            return Err(MiraError::conversion_number("f32", value));
        }
        Ok(value as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(MiraValue::number(42.0), MiraValue::number(42.0));
    }

    #[test]
    fn test_number_is_number() {
        assert!(MiraValue::number(42.0).is_number());
        assert!(!MiraValue::nil().is_number());
    }

    #[test]
    fn test_number_as_number() {
        assert_eq!(MiraValue::number(42.0).as_number(), Some(42.0));
        assert_eq!(MiraValue::nil().as_number(), None);
    }

    #[test]
    fn test_number_try_from() {
        let value = MiraValue::number(42.0);
        assert_eq!(u8::try_from(value).unwrap(), 42u8);
        assert_eq!(i32::try_from(value).unwrap(), 42i32);
        assert_eq!(u128::try_from(value).unwrap(), 42u128);
        assert_eq!(f32::try_from(value).unwrap(), 42f32);
        assert_eq!(f64::try_from(value).unwrap(), 42f64);
    }
    #[test]
    fn test_number_try_from_underflow() {
        let value = MiraValue::number(-1.0);
        assert!(u8::try_from(value).is_err());
        assert!(u32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_overflow() {
        let value = MiraValue::number(256.0);

        assert!(dbg!(u8::try_from(value)).is_err());
        let value = MiraValue::number(u32::MAX as f64 * 1.1);
        assert!(dbg!(u32::try_from(value)).is_err());
        let value = MiraValue::number(u128::MAX as f64 * 1.1);
        assert!(dbg!(u128::try_from(value)).is_err());
        let value = MiraValue::number(f32::MAX as f64 * 1.1);
        assert!(f32::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_integer() {
        let value = MiraValue::number(42.5);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_manageable() {
        let value: fn() -> MiraManageable = || MiraValue::number(42.0).into();
        assert_eq!(u8::try_from(value()).unwrap(), 42u8);
        assert_eq!(i32::try_from(value()).unwrap(), 42i32);
        assert_eq!(u128::try_from(value()).unwrap(), 42u128);
        assert_eq!(f32::try_from(value()).unwrap(), 42f32);
        assert_eq!(f64::try_from(value()).unwrap(), 42f64);

        let value: fn() -> MiraManageable = || "hello".into();
        assert!(u8::try_from(value()).is_err());
        assert!(i32::try_from(value()).is_err());
        assert!(u128::try_from(value()).is_err());
        assert!(f32::try_from(value()).is_err());
        assert!(f64::try_from(value()).is_err());
    }

    #[test]
    fn test_number_try_from_non_finite() {
        let value = MiraValue::number(f64::INFINITY);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
        let value = MiraValue::number(f64::NAN);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_number() {
        assert!(f64::try_from(MiraValue::nil()).is_err());

        let value = MiraValue::boolean(true);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());

        let value = MiraValue::str(&"42");
        assert!(u8::try_from(value).is_err());
        assert!(f64::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_negative_zero() {
        let value = MiraValue::number(-0.0);
        assert_eq!(u8::try_from(value).unwrap(), 0u8);
        assert_eq!(i32::try_from(value).unwrap(), 0i32);
        assert_eq!(u128::try_from(value).unwrap(), 0u128);
        assert_eq!(f32::try_from(value).unwrap(), -0.0f32);
        assert_eq!(f64::try_from(value).unwrap(), -0.0f64);
    }

    #[test]
    fn test_number_into() {
        assert_eq!(MiraValue::from(42u8), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42i32), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42u128), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42f32), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42f64), MiraValue::number(42.0));
    }
}

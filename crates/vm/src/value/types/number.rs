use super::MiraValue;
use crate::value::arena::MiraManageable;
use crate::{MiraError, Result};

impl MiraValue {
    /// Create a `MiraValue` representing a numeric value.
    #[inline]
    pub const fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Check whether this value is a numeric value.
    #[inline]
    pub const fn is_number(self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Return the inline numeric payload, or `None` for another value type.
    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
}

macro_rules! number_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl From<$ty> for MiraValue {
            fn from(value: $ty) -> Self {
                Self::Number(value as f64)
            }
        }
        impl From<$ty> for MiraManageable {
            fn from(value: $ty) -> Self {
                std::convert::Into::<MiraValue>::into(value as f64).into()
            }
        }
    )* };
}

number_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

macro_rules! unsigned_integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraValue> for $ty {
            type Error = Box<MiraError>;

            fn try_from(value: MiraValue) -> Result<Self> {
                let number = value.as_number().ok_or_else(|| MiraError::conversion_type(stringify!($ty), value.value_type()))?;
                if !number.is_finite()
                    || number.trunc() != number
                    || number < 0.0
                    || number >= 2_f64.powi(<$ty>::BITS as i32)
                {
                    return Err(MiraError::conversion_number(stringify!($ty), number));
                }
                Ok(number as $ty)
            }
        }
    )* };
}

macro_rules! signed_integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraValue> for $ty {
            type Error = Box<MiraError>;

            fn try_from(value: MiraValue) -> Result<Self> {
                let MiraValue::Number(number) = value else {
                    return Err(MiraError::conversion_type(stringify!($ty), value.value_type()));
                };
                let limit = 2_f64.powi(<$ty>::BITS as i32 - 1);
                if !number.is_finite()
                    || number.trunc() != number
                    || number < -limit
                    || number >= limit
                {
                    return Err(MiraError::conversion_number(stringify!($ty), number));
                }
                Ok(number as $ty)
            }
        }
    )* };
}

unsigned_integer_try_from!(u8, u16, u32, u64, u128, usize);
signed_integer_try_from!(i8, i16, i32, i64, i128, isize);

impl TryFrom<MiraValue> for f64 {
    type Error = Box<MiraError>;

    fn try_from(value: MiraValue) -> Result<Self> {
        match value {
            MiraValue::Number(value) => Ok(value),
            value => Err(MiraError::conversion_type("f64", value.value_type())),
        }
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
        assert_eq!(MiraValue::number(42.0), MiraValue::Number(42.0));
    }

    #[test]
    fn test_number_is_number() {
        assert!(MiraValue::Number(42.0).is_number());
        assert!(!MiraValue::Nil.is_number());
    }

    #[test]
    fn test_number_as_number() {
        assert_eq!(MiraValue::Number(42.0).as_number(), Some(42.0));
        assert_eq!(MiraValue::Nil.as_number(), None);
    }

    #[test]
    fn test_number_try_from() {
        let value = MiraValue::Number(42.0);
        assert_eq!(u8::try_from(value).unwrap(), 42u8);
        assert_eq!(i32::try_from(value).unwrap(), 42i32);
        assert_eq!(u128::try_from(value).unwrap(), 42u128);
        assert_eq!(f32::try_from(value).unwrap(), 42f32);
        assert_eq!(f64::try_from(value).unwrap(), 42f64);
    }
    #[test]
    fn test_number_try_from_underflow() {
        let value = MiraValue::Number(-1.0);
        assert!(u8::try_from(value).is_err());
        assert!(u32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_overflow() {
        let value = MiraValue::Number(256.0);
        assert!(u8::try_from(value).is_err());
        let value = MiraValue::Number(2_f64.powi(32));
        assert!(u32::try_from(value).is_err());
        let value = MiraValue::Number(2_f64.powi(128));
        assert!(u128::try_from(value).is_err());
        let value = MiraValue::Number(f64::MAX);
        assert!(f32::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_integer() {
        let value = MiraValue::Number(42.5);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_finite() {
        let value = MiraValue::Number(f64::INFINITY);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
        let value = MiraValue::Number(f64::NAN);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_number() {
        assert!(f64::try_from(MiraValue::Nil).is_err());

        let value = MiraValue::Boolean(true);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());

        let value = MiraValue::StaticStr(&"42");
        assert!(u8::try_from(value).is_err());
        assert!(f64::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_negative_zero() {
        let value = MiraValue::Number(-0.0);
        assert_eq!(u8::try_from(value).unwrap(), 0u8);
        assert_eq!(i32::try_from(value).unwrap(), 0i32);
        assert_eq!(u128::try_from(value).unwrap(), 0u128);
        assert_eq!(f32::try_from(value).unwrap(), -0.0f32);
        assert_eq!(f64::try_from(value).unwrap(), -0.0f64);
    }

    #[test]
    fn test_number_into() {
        assert_eq!(MiraValue::from(42u8), MiraValue::Number(42.0));
        assert_eq!(MiraValue::from(42i32), MiraValue::Number(42.0));
        assert_eq!(MiraValue::from(42u128), MiraValue::Number(42.0));
        assert_eq!(MiraValue::from(42f32), MiraValue::Number(42.0));
        assert_eq!(MiraValue::from(42f64), MiraValue::Number(42.0));
    }
}

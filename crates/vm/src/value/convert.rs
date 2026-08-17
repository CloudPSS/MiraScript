use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};

use indexmap::IndexMap;

use crate::{MiraError, Result};

use super::{MiraAny, MiraBridge, MiraFunction, MiraModule, MiraNativeFn, MiraShared};

impl From<()> for MiraAny {
    fn from(_: ()) -> Self {
        Self::Nil
    }
}

impl From<bool> for MiraAny {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for MiraAny {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&str> for MiraAny {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

macro_rules! number_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl From<$ty> for MiraAny {
            fn from(value: $ty) -> Self {
                Self::Number(value as f64)
            }
        }
    )* };
}

number_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

impl<T: Into<MiraAny>> From<Option<T>> for MiraAny {
    fn from(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or(Self::Nil)
    }
}

impl<T: Into<MiraAny>> From<Vec<T>> for MiraAny {
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect::<Vec<_>>().into())
    }
}

impl<T: Into<MiraAny>, const N: usize> From<[T; N]> for MiraAny {
    fn from(value: [T; N]) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect::<Vec<_>>().into())
    }
}

impl<T: Into<MiraAny>> From<IndexMap<String, T>> for MiraAny {
    fn from(value: IndexMap<String, T>) -> Self {
        Self::Record(
            value
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect::<IndexMap<_, _>>()
                .into(),
        )
    }
}

impl<K, T, S> From<HashMap<K, T, S>> for MiraAny
where
    K: Into<String> + Eq + Hash,
    T: Into<MiraAny>,
    S: BuildHasher,
{
    fn from(value: HashMap<K, T, S>) -> Self {
        let mut entries: Vec<_> = value
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        Self::Record(entries.into_iter().collect::<IndexMap<_, _>>().into())
    }
}

impl<K, T> From<BTreeMap<K, T>> for MiraAny
where
    K: Into<String> + Ord,
    T: Into<MiraAny>,
{
    fn from(value: BTreeMap<K, T>) -> Self {
        Self::Record(
            value
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect::<IndexMap<_, _>>()
                .into(),
        )
    }
}

impl From<MiraNativeFn> for MiraAny {
    fn from(value: MiraNativeFn) -> Self {
        Self::Function(MiraFunction::Native(value).into())
    }
}

impl From<MiraModule> for MiraAny {
    fn from(value: MiraModule) -> Self {
        Self::Module(value.into())
    }
}

impl<T: MiraBridge> From<MiraShared<T>> for MiraAny {
    fn from(value: MiraShared<T>) -> Self {
        T::into_mira_shared(value)
    }
}

impl TryFrom<MiraAny> for bool {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        match value {
            MiraAny::Boolean(value) => Ok(value),
            value => Err(MiraError::conversion("bool", &value)),
        }
    }
}

impl TryFrom<MiraAny> for String {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        match value {
            MiraAny::String(value) => Ok(value.into_inner()),
            value => Err(MiraError::conversion("String", &value)),
        }
    }
}

impl<T> TryFrom<MiraAny> for Option<T>
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        if value == MiraAny::Nil {
            Ok(None)
        } else {
            T::try_from(value).map(Some)
        }
    }
}

impl<T> TryFrom<MiraAny> for Vec<T>
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let MiraAny::Array(values) = value else {
            return Err(MiraError::conversion("Vec", &value));
        };
        values
            .into_inner()
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                T::try_from(value).map_err(|error| error.at_path(index.to_string()))
            })
            .collect()
    }
}

impl<T, const N: usize> TryFrom<MiraAny> for [T; N]
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let values = Vec::<T>::try_from(value)?;
        let actual = values.len();
        values.try_into().map_err(|_| MiraError::Conversion {
            expected: format!("array of length {N}"),
            actual: format!("array of length {actual}"),
            path: None,
        })
    }
}

impl<T> TryFrom<MiraAny> for IndexMap<String, T>
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let MiraAny::Record(values) = value else {
            return Err(MiraError::conversion("record", &value));
        };
        values
            .into_inner()
            .into_iter()
            .map(|(key, value)| {
                T::try_from(value)
                    .map(|value| (key.clone(), value))
                    .map_err(|error| error.at_path(key))
            })
            .collect()
    }
}

macro_rules! unsigned_integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraAny> for $ty {
            type Error = MiraError;

            fn try_from(value: MiraAny) -> Result<Self> {
                let MiraAny::Number(number) = value else {
                    return Err(MiraError::conversion(stringify!($ty), &value));
                };
                if !number.is_finite()
                    || number.trunc() != number
                    || number < 0.0
                    || number >= 2_f64.powi(<$ty>::BITS as i32)
                {
                    return Err(MiraError::Conversion {
                        expected: stringify!($ty).into(),
                        actual: format!("number {number}"),
                        path: None,
                    });
                }
                Ok(number as $ty)
            }
        }
    )* };
}

macro_rules! signed_integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraAny> for $ty {
            type Error = MiraError;

            fn try_from(value: MiraAny) -> Result<Self> {
                let MiraAny::Number(number) = value else {
                    return Err(MiraError::conversion(stringify!($ty), &value));
                };
                let limit = 2_f64.powi(<$ty>::BITS as i32 - 1);
                if !number.is_finite()
                    || number.trunc() != number
                    || number < -limit
                    || number >= limit
                {
                    return Err(MiraError::Conversion {
                        expected: stringify!($ty).into(),
                        actual: format!("number {number}"),
                        path: None,
                    });
                }
                Ok(number as $ty)
            }
        }
    )* };
}

unsigned_integer_try_from!(u8, u16, u32, u64, u128, usize);
signed_integer_try_from!(i8, i16, i32, i64, i128, isize);

impl TryFrom<MiraAny> for f64 {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        match value {
            MiraAny::Number(value) => Ok(value),
            value => Err(MiraError::conversion("f64", &value)),
        }
    }
}

impl TryFrom<MiraAny> for f32 {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let value = f64::try_from(value)?;
        if value.is_finite() && (value < f32::MIN as f64 || value > f32::MAX as f64) {
            return Err(MiraError::Conversion {
                expected: "f32".into(),
                actual: format!("number {value}"),
                path: None,
            });
        }
        Ok(value as f32)
    }
}

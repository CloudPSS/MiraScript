use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};

use indexmap::IndexMap;

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

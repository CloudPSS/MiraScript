mod all_any;
mod entries;
mod find;
mod flatten;
mod fold;
mod group;
mod len;
mod map_filter;
mod new;
mod repeat;
mod reverse;
mod sort;
mod unique;
mod with;
mod zip;

use indexmap::IndexMap;

use crate::standard_library::{callable, const_value, global_builtin, optional_callable, required};
use crate::{
    MiraError, MiraFunctionHandle, MiraValue, MiraValueKind, Result, Runtime, RuntimeErrorKind,
    operations,
};

use with::array_length;

pub(super) enum Data {
    Primitive(MiraValue),
    Array(MiraValue),
    Record(IndexMap<String, MiraValue>),
}

impl Data {
    pub(super) fn from_value(runtime: &mut Runtime, value: MiraValue) -> Result<Self> {
        match value.kind() {
            MiraValueKind::Nil
            | MiraValueKind::Boolean(_)
            | MiraValueKind::Number(_)
            | MiraValueKind::String(_)
            | MiraValueKind::StaticStr(_) => Ok(Self::Primitive(value)),
            MiraValueKind::Array(_) => Ok(Self::Array(value)),
            MiraValueKind::Record(_) => {
                let mut record = IndexMap::new();
                for key in operations::record_keys(runtime, value)?.unwrap_or_default() {
                    record.insert(
                        key.clone(),
                        operations::record_get(runtime, value, &key)?.unwrap_or(MiraValue::NIL),
                    );
                }
                Ok(Self::Record(record))
            }
            _ => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                expected: "nil, number, boolean, string, array, or record",
                actual: value.value_type(),
            })),
        }
    }

    pub(super) fn original(&self, runtime: &mut Runtime) -> Result<MiraValue> {
        match self {
            Self::Primitive(value) => Ok(*value),
            Self::Array(value) => Ok(*value),
            Self::Record(value) => runtime.insert(value.clone()),
        }
    }
}

pub(super) fn install(context: &mut Runtime) {
    len::install(context);
    entries::install(context);
    map_filter::install(context);
    fold::install(context);
    group::install(context);
    flatten::install(context);
    find::install(context);
    all_any::install(context);
    reverse::install(context);
    sort::install(context);
    unique::install(context);
    repeat::install(context);
    new::install(context);
    zip::install(context);
    with::install(context);
}

pub(super) fn data_items(
    runtime: &mut Runtime,
    data: &Data,
) -> Result<Vec<(MiraValue, MiraValue)>> {
    iterate_data(
        runtime,
        data,
        |_, length| Ok(Vec::with_capacity(length)),
        |_, key, value, items| {
            items.push((key, value));
            Ok(true)
        },
    )
}

pub(super) fn iterate_data<T>(
    runtime: &mut Runtime,
    data: &Data,
    init: impl FnOnce(&mut Runtime, usize) -> Result<T>,
    mut body: impl FnMut(&mut Runtime, MiraValue, MiraValue, &mut T) -> Result<bool>,
) -> Result<T> {
    match data {
        Data::Primitive(value) => {
            let mut acc = init(runtime, 1)?;
            body(runtime, MiraValue::NIL, *value, &mut acc)?;
            Ok(acc)
        }
        Data::Array(value) => {
            let iter = operations::iterate_array(runtime, *value)?;
            let mut acc = init(runtime, iter.len())?;
            for entry in iter {
                let index = entry.index();
                let value = entry.get(runtime)?;
                if !body(runtime, MiraValue::number(index as f64), value, &mut acc)? {
                    break;
                }
            }
            Ok(acc)
        }
        Data::Record(values) => {
            let mut acc = init(runtime, values.len())?;
            for (key, value) in values {
                let key = runtime.insert(key.clone())?;
                if !body(runtime, key, *value, &mut acc)? {
                    break;
                }
            }
            Ok(acc)
        }
    }
}

pub(super) fn array_value(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<MiraValue>> {
    operations::iterable_array(runtime, value)
}

pub(super) fn pair(
    runtime: &mut Runtime,
    first: MiraValue,
    second: MiraValue,
) -> Result<MiraValue> {
    runtime.insert(IndexMap::from([("0".into(), first), ("1".into(), second)]))
}

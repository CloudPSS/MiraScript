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

use crate::standard_library::{const_value, insert_native, is_callable, required};
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

use with::array_length;

pub(super) enum Data {
    Primitive(MiraValue),
    Array(Vec<MiraValue>),
    Record(IndexMap<String, MiraValue>),
}

impl Data {
    pub(super) fn from_value(runtime: &mut Runtime, value: MiraValue) -> Result<Self> {
        match value {
            MiraValue::Nil
            | MiraValue::Boolean(_)
            | MiraValue::Number(_)
            | MiraValue::String(_)
            | MiraValue::StaticString(_) => Ok(Self::Primitive(value)),
            MiraValue::Array(_) => Ok(Self::Array(operations::iterable_array(runtime, value)?)),
            MiraValue::Record(_) => {
                let mut record = IndexMap::new();
                for key in operations::record_keys(runtime, value)?.unwrap_or_default() {
                    record.insert(
                        key.clone(),
                        operations::record_get(runtime, value, &key)?.unwrap_or(MiraValue::Nil),
                    );
                }
                Ok(Self::Record(record))
            }
            value => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                expected: "nil, number, boolean, string, array, or record",
                actual: value.value_type(),
            })),
        }
    }

    pub(super) fn original(&self, runtime: &mut Runtime) -> Result<MiraValue> {
        match self {
            Self::Primitive(value) => Ok(*value),
            Self::Array(value) => runtime.insert(value.clone()),
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
    Ok(match data {
        Data::Primitive(value) => vec![(MiraValue::Nil, *value)],
        Data::Array(values) => values
            .iter()
            .copied()
            .enumerate()
            .map(|(index, value)| (MiraValue::Number(index as f64), value))
            .collect(),
        Data::Record(values) => values
            .iter()
            .map(|(key, value)| Ok((runtime.insert(key.clone())?, *value)))
            .collect::<Result<Vec<_>>>()?,
    })
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

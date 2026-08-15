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
use crate::{MiraAny, MiraCallContext, MiraContext, MiraError, Result, operations};

use with::array_length;

pub(super) enum Data {
    Primitive(MiraAny),
    Array(Vec<MiraAny>),
    Record(IndexMap<String, MiraAny>),
}

impl Data {
    pub(super) fn from_value(value: &MiraAny) -> Result<Self> {
        match value {
            MiraAny::Nil | MiraAny::Boolean(_) | MiraAny::Number(_) | MiraAny::String(_) => {
                Ok(Self::Primitive(value.clone()))
            }
            MiraAny::Array(_) | MiraAny::RustArray(_) => {
                Ok(Self::Array(operations::materialize_array(value)?))
            }
            MiraAny::Record(_) | MiraAny::RustRecord(_) => {
                let mut record = IndexMap::new();
                for key in value.record_keys()?.unwrap_or_default() {
                    record.insert(key.clone(), value.record_get(&key)?.unwrap_or(MiraAny::Nil));
                }
                Ok(Self::Record(record))
            }
            _ => Err(MiraError::runtime(format!(
                "Expected nil, number, boolean, string, array or record, got {}",
                operations::display(value)
            ))),
        }
    }

    pub(super) fn original(&self) -> MiraAny {
        match self {
            Self::Primitive(value) => value.clone(),
            Self::Array(value) => MiraAny::Array(value.clone()),
            Self::Record(value) => MiraAny::Record(value.clone()),
        }
    }
}

pub(super) fn install(context: &mut MiraContext) {
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

pub(super) fn data_items(data: &Data) -> Vec<(MiraAny, MiraAny)> {
    match data {
        Data::Primitive(value) => vec![(MiraAny::Nil, value.clone())],
        Data::Array(values) => values
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| (MiraAny::Number(index as f64), value))
            .collect(),
        Data::Record(values) => values
            .iter()
            .map(|(key, value)| (MiraAny::String(key.clone()), value.clone()))
            .collect(),
    }
}

pub(super) fn array_value(value: &MiraAny) -> Result<Vec<MiraAny>> {
    operations::materialize_array(value)
}

pub(super) fn pair(first: MiraAny, second: MiraAny) -> MiraAny {
    MiraAny::Record(IndexMap::from([("0".into(), first), ("1".into(), second)]))
}

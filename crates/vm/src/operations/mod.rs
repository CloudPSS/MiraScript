mod array_range;
mod common;
mod convert;
mod iterable;
mod operator;
mod record;
mod slice;
mod spread;
mod type_check;

use std::cmp::Ordering;

use indexmap::IndexMap;

use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind};

use common::javascript_exponent;
use convert::inner_to_string;

pub(crate) use array_range::*;
pub(crate) use common::*;
pub(crate) use convert::*;
pub(crate) use iterable::*;
pub(crate) use operator::*;
pub(crate) use record::*;
pub(crate) use slice::*;
pub(crate) use spread::*;
pub(crate) use type_check::*;

#[inline]
pub(crate) const fn into_element(value: MiraValue) -> MiraValue {
    match value {
        MiraValue::Function(_) | MiraValue::Module(_) | MiraValue::Extern(_) => MiraValue::Nil,
        value => value,
    }
}

pub(crate) fn array_len(runtime: &Runtime, value: MiraValue) -> Result<Option<usize>> {
    match value {
        MiraValue::Array(handle) => Ok(Some(runtime.get_array_dyn(handle)?.len())),
        _ => Ok(None),
    }
}

pub(crate) fn array_get(
    runtime: &mut Runtime,
    value: MiraValue,
    index: usize,
) -> Result<Option<MiraValue>> {
    let MiraValue::Array(handle) = value else {
        return Ok(None);
    };
    let manageable = {
        let array = runtime.get_array_dyn(handle)?;
        if index >= array.len() {
            return Ok(None);
        }
        array.get(handle, runtime, index)?
    };
    runtime.insert(manageable).map(Some)
}

pub(crate) fn record_keys(runtime: &Runtime, value: MiraValue) -> Result<Option<Vec<String>>> {
    let MiraValue::Record(handle) = value else {
        return Ok(None);
    };
    let record = runtime.get_record_dyn(handle)?;
    (0..record.len())
        .map(|index| record.key(index).map(str::to_owned))
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

pub(crate) fn record_get(
    runtime: &mut Runtime,
    value: MiraValue,
    key: &str,
) -> Result<Option<MiraValue>> {
    let MiraValue::Record(handle) = value else {
        return Ok(None);
    };
    let manageable = {
        let record = runtime.get_record_dyn(handle)?;
        let Some(index) = record.index_of(key) else {
            return Ok(None);
        };
        record.get(handle, runtime, index)?
    };
    runtime.insert(manageable).map(Some)
}

pub(crate) fn module_keys(runtime: &Runtime, value: MiraValue) -> Result<Option<Vec<String>>> {
    let MiraValue::Module(handle) = value else {
        return Ok(None);
    };
    let module = runtime.get_module_dyn(handle)?;
    (0..module.len())
        .map(|index| module.key(index).map(str::to_owned))
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

pub(crate) fn module_get(
    runtime: &mut Runtime,
    value: MiraValue,
    key: &str,
) -> Result<Option<MiraValue>> {
    let MiraValue::Module(handle) = value else {
        return Ok(None);
    };
    let manageable = {
        let module = runtime.get_module_dyn(handle)?;
        let Some(index) = module.index_of(key) else {
            return Ok(None);
        };
        module.get(handle, runtime, index)?
    };
    runtime.insert(manageable).map(Some)
}

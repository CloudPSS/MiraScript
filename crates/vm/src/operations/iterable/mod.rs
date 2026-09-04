mod array;
mod module;
mod record;

use super::*;

pub(crate) use array::*;
pub(crate) use module::*;
pub(crate) use record::*;

pub(crate) fn length(runtime: &Runtime, value: MiraValue) -> Result<usize> {
    match value.kind() {
        MiraValueKind::Record(h) => h.len(runtime),
        MiraValueKind::Module(h) => h.len(runtime),
        MiraValueKind::Array(h) => h.len(runtime),
        _ => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array, record, or module",
            actual: value.value_type(),
        })),
    }
}

pub(crate) fn iterable(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<MiraValue>> {
    match value.kind() {
        MiraValueKind::Array(_) => iterable_array(runtime, value),
        MiraValueKind::Record(_) => iterate_record(runtime, value)?
            .map(|entry| runtime.insert(entry.key(runtime)?.to_owned()))
            .collect(),
        MiraValueKind::Module(_) => iterate_module(runtime, value)?
            .map(|entry| runtime.insert(entry.key(runtime)?.to_owned()))
            .collect(),
        _ => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "iterable value",
            actual: value.value_type(),
        })),
    }
}

pub(crate) fn in_value(runtime: &mut Runtime, needle: MiraValue, value: MiraValue) -> Result<bool> {
    match value.kind() {
        MiraValueKind::Array(_) => {
            for entry in iterate_array(runtime, value)? {
                let element = entry.get(runtime)?;
                if same_value(runtime, element, needle)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MiraValueKind::Record(_) | MiraValueKind::Module(_) => {
            let key = to_string(runtime, needle)?;
            has(runtime, value, MiraValue::NIL, Some(&key))
        }
        _ => Ok(false),
    }
}

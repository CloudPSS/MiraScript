use super::*;

pub(crate) fn length(runtime: &Runtime, value: MiraValue) -> Result<usize> {
    if let Some(length) = array_len(runtime, value)? {
        return Ok(length);
    }
    match value {
        MiraValue::Record(_) => Ok(record_keys(runtime, value)?.unwrap_or_default().len()),
        MiraValue::Module(_) => Ok(module_keys(runtime, value)?.unwrap_or_default().len()),
        value => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array, record, or module",
            actual: value.value_type(),
        })),
    }
}

pub(crate) fn iterable(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<MiraValue>> {
    match value {
        MiraValue::Array(_) => iterable_array(runtime, value),
        MiraValue::Record(_) => record_keys(runtime, value)?
            .unwrap_or_default()
            .into_iter()
            .map(|key| runtime.insert(key))
            .collect(),
        MiraValue::Module(_) => module_keys(runtime, value)?
            .unwrap_or_default()
            .into_iter()
            .map(|key| runtime.insert(key))
            .collect(),
        value => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "iterable value",
            actual: value.value_type(),
        })),
    }
}

pub(crate) fn iterable_array(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<MiraValue>> {
    let Some(length) = array_len(runtime, value)? else {
        return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array",
            actual: value.value_type(),
        }));
    };
    (0..length)
        .map(|index| Ok(array_get(runtime, value, index)?.unwrap_or(MiraValue::Nil)))
        .collect()
}

use super::*;

pub(crate) fn array_spread(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<MiraValue>> {
    match value.kind() {
        MiraValueKind::Nil => Ok(Vec::new()),
        MiraValueKind::Array(_) => iterable_array(runtime, value),
        _ => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array or nil",
            actual: value.value_type(),
        })),
    }
}

pub(crate) fn record_spread(
    runtime: &mut Runtime,
    value: MiraValue,
) -> Result<IndexMap<String, MiraValue>> {
    let mut result = IndexMap::new();
    match value.kind() {
        MiraValueKind::Nil => {}
        MiraValueKind::Record(_) => {
            for key in record_keys(runtime, value)?.unwrap_or_default() {
                result.insert(
                    key.clone(),
                    into_element(record_get(runtime, value, &key)?.unwrap_or_else(MiraValue::nil)),
                );
            }
        }
        MiraValueKind::Array(_) => {
            for (index, item) in iterable_array(runtime, value)?.into_iter().enumerate() {
                result.insert(index.to_string(), into_element(item));
            }
        }
        _ => {
            return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                expected: "record, array, or nil",
                actual: value.value_type(),
            }));
        }
    }
    Ok(result)
}

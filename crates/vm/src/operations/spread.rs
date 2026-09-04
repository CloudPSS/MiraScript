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
            let entries = iterable_record(runtime, value)?;
            result = IndexMap::with_capacity(entries.len());
            for (key, item) in entries {
                result.insert(key, into_element(item));
            }
        }
        MiraValueKind::Array(_) => {
            let iter = iterate_array(runtime, value)?;
            result = IndexMap::with_capacity(iter.len());
            for entry in iter {
                let index = entry.index();
                let item = entry.get(runtime)?;
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

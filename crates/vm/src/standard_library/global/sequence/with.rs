use super::*;

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(runtime, fn with(call, args) {
        let max = call.options().max_array_len;
        update_with(call, *required(args, 0, "data")?, &args[1..], max)
    });
}

fn update_with(
    runtime: &mut Runtime,
    data: MiraValue,
    entries: &[MiraValue],
    max_len: usize,
) -> Result<MiraValue> {
    if !entries.len().is_multiple_of(2) {
        return Err(MiraError::runtime(
            RuntimeErrorKind::InvalidUpdateEntryCount {
                actual: entries.len(),
            },
        ));
    }
    if !data.is_array() && !data.is_record() {
        return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array or record",
            actual: data.value_type(),
        }));
    }
    let mut result = data;
    for pair in entries.as_chunks::<2>().0 {
        let path = if operations::array_len(runtime, pair[0])?.is_some() {
            operations::iterable_array(runtime, pair[0])?
        } else if pair[0] == MiraValue::NIL {
            continue;
        } else {
            vec![pair[0]]
        };
        if path.is_empty() || path.contains(&MiraValue::NIL) {
            continue;
        }
        result = set_path(runtime, result, &path, const_value(pair[1])?, max_len)?;
    }
    Ok(result)
}

fn set_path(
    runtime: &mut Runtime,
    data: MiraValue,
    path: &[MiraValue],
    value: MiraValue,
    max_len: usize,
) -> Result<MiraValue> {
    if path.is_empty() {
        return Ok(value);
    }
    match data.kind() {
        MiraValueKind::Array(_) => {
            let mut values = operations::iterable_array(runtime, data)?;
            let index = array_index(runtime, path[0], max_len)?;
            while values.len() <= index {
                values.push(MiraValue::NIL);
            }
            let current = values[index];
            let current = container_for(runtime, current, path.get(1).cloned())?;
            values[index] = set_path(runtime, current, &path[1..], value, max_len)?;
            runtime.insert(values)
        }
        MiraValueKind::Record(_) => {
            let mut values = operations::iterable_record(runtime, data)?;
            let key = operations::to_string(runtime, path[0])?;
            let current = values.get(&key).cloned().unwrap_or(MiraValue::NIL);
            let current = container_for(runtime, current, path.get(1).cloned())?;
            values.insert(key, set_path(runtime, current, &path[1..], value, max_len)?);
            runtime.insert(values)
        }
        _ => {
            let container = container_for(runtime, data, path.first().cloned())?;
            set_path(runtime, container, path, value, max_len)
        }
    }
}

fn container_for(
    runtime: &mut Runtime,
    current: MiraValue,
    next: Option<MiraValue>,
) -> Result<MiraValue> {
    if current.is_array() || current.is_record() {
        return Ok(current);
    }
    if next.is_some_and(|value| {
        value
            .as_number()
            .is_some_and(|number| number.fract() == 0.0 && number >= 0.0)
    }) {
        runtime.insert(Vec::<MiraValue>::new())
    } else {
        runtime.insert(IndexMap::<String, MiraValue>::new())
    }
}

fn array_index(runtime: &Runtime, value: MiraValue, max_len: usize) -> Result<usize> {
    let index = operations::to_number(runtime, value)?;
    if !index.is_finite() || index < 0.0 {
        return Err(MiraError::runtime(
            RuntimeErrorKind::InvalidIntegerArgument {
                name: "index",
                constraint: "a non-negative integer",
            },
        ));
    }
    let index = index.trunc() as usize;
    if index >= max_len {
        return Err(MiraError::runtime(RuntimeErrorKind::ArrayLimit {
            requested: index.saturating_add(1),
            max: max_len,
        }));
    }
    Ok(index)
}

pub(super) fn array_length(runtime: &Runtime, value: MiraValue, max_len: usize) -> Result<usize> {
    let length = operations::to_number(runtime, value)?;
    if !length.is_finite() || length <= -1.0 {
        return Err(MiraError::runtime(
            RuntimeErrorKind::InvalidIntegerArgument {
                name: "length",
                constraint: "a non-negative integer",
            },
        ));
    }
    let length = length.trunc() as usize;
    if length > max_len {
        return Err(MiraError::runtime(RuntimeErrorKind::ArrayLimit {
            requested: length,
            max: max_len,
        }));
    }
    Ok(length)
}

use super::*;

pub(crate) fn has(
    runtime: &mut Runtime,
    value: MiraValue,
    key: MiraValue,
    known_key: Option<&str>,
) -> Result<bool> {
    let key = match known_key {
        Some(key) => key.to_owned(),
        None => to_string(runtime, key)?,
    };
    match value.kind() {
        MiraValueKind::Array(_) => {
            let Ok(index) = key.parse::<usize>() else {
                return Ok(false);
            };
            Ok(index < array_len(runtime, value)?.unwrap_or(0))
        }
        MiraValueKind::Record(_) => Ok(record_keys(runtime, value)?
            .is_some_and(|keys| keys.iter().any(|candidate| candidate == &key))),
        MiraValueKind::Module(_) => Ok(module_keys(runtime, value)?
            .is_some_and(|keys| keys.iter().any(|candidate| candidate == &key))),
        _ => Ok(false),
    }
}

pub(crate) fn get(runtime: &mut Runtime, value: MiraValue, key: &str) -> Result<MiraValue> {
    get_value(runtime, value, MiraValue::nil(), Some(key))
}

pub(crate) fn get_value(
    runtime: &mut Runtime,
    value: MiraValue,
    key: MiraValue,
    known_key: Option<&str>,
) -> Result<MiraValue> {
    if value.is_array() {
        let index = match to_number(runtime, key) {
            Ok(index) if index.is_finite() => index.trunc() as isize,
            _ => return Ok(MiraValue::nil()),
        };
        let length = array_len(runtime, value)?.unwrap_or(0);
        let index = if index < 0 {
            length.checked_add_signed(index)
        } else {
            Some(index as usize)
        };
        return match index {
            Some(index) if index < length => Ok(into_element(
                array_get(runtime, value, index)?.unwrap_or_else(MiraValue::nil),
            )),
            _ => Ok(MiraValue::nil()),
        };
    }

    let key = match known_key {
        Some(key) => key.to_owned(),
        None => to_string(runtime, key)?,
    };
    match value.kind() {
        MiraValueKind::Record(_) => Ok(into_element(
            record_get(runtime, value, &key)?.unwrap_or_else(MiraValue::nil),
        )),
        MiraValueKind::Module(_) => {
            Ok(module_get(runtime, value, &key)?.unwrap_or_else(MiraValue::nil))
        }
        _ => Ok(MiraValue::nil()),
    }
}

pub(crate) fn set(
    _runtime: &mut Runtime,
    obj: MiraValue,
    _key: MiraValue,
    _value: MiraValue,
) -> Result<()> {
    Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
        expected: "mutable extern",
        actual: obj.value_type(),
    }))
}

pub(crate) fn pick(runtime: &mut Runtime, value: MiraValue, keys: &[String]) -> Result<MiraValue> {
    let mut result = IndexMap::new();
    if value.is_record() {
        for key in keys {
            if has(runtime, value, MiraValue::nil(), Some(key))? {
                result.insert(key.clone(), get(runtime, value, key)?);
            }
        }
    }
    runtime.insert(result)
}

pub(crate) fn omit(runtime: &mut Runtime, value: MiraValue, keys: &[String]) -> Result<MiraValue> {
    let mut result = IndexMap::new();
    if let Some(existing) = record_keys(runtime, value)? {
        for key in existing {
            if !keys.contains(&key) {
                result.insert(key.clone(), get(runtime, value, &key)?);
            }
        }
    }
    runtime.insert(result)
}

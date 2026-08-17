use super::*;

pub(crate) fn has(value: &MiraAny, key: &MiraAny) -> Result<bool> {
    assert_initialized(value)?;
    let key = to_string(key)?;
    match value {
        MiraAny::Nil
        | MiraAny::Boolean(_)
        | MiraAny::Number(_)
        | MiraAny::String(_)
        | MiraAny::Function(_)
        | MiraAny::Uninitialized => Ok(false),
        MiraAny::Array(_) | MiraAny::RustArray(_) => {
            let Ok(index) = key.parse::<usize>() else {
                return Ok(false);
            };
            Ok(index < value.array_len()?.unwrap_or(0))
        }
        MiraAny::Record(_) | MiraAny::RustRecord(_) => Ok(value
            .record_keys()?
            .is_some_and(|keys| keys.iter().any(|candidate| candidate == &key))),
        MiraAny::Extern(value) => value.has(&key),
        MiraAny::Module(module) => Ok(module.keys().iter().any(|candidate| candidate == &key)),
    }
}

pub(crate) fn get(value: &MiraAny, key: &str) -> Result<MiraAny> {
    get_value(value, &MiraAny::String(key.into()))
}

pub(crate) fn get_value(value: &MiraAny, key: &MiraAny) -> Result<MiraAny> {
    assert_initialized(value)?;
    if matches!(value, MiraAny::Array(_) | MiraAny::RustArray(_)) {
        let index = match to_number(key) {
            Ok(index) if index.is_finite() => index.trunc() as isize,
            _ => return Ok(MiraAny::Nil),
        };
        let length = value.array_len()?.unwrap_or(0);
        let index = if index < 0 {
            length.checked_add_signed(index)
        } else {
            Some(index as usize)
        };
        return match index {
            Some(index) if index < length => value
                .array_get(index)?
                .unwrap_or(MiraAny::Nil)
                .into_element(),
            _ => Ok(MiraAny::Nil),
        };
    }

    let key = to_string(key)?;
    match value {
        MiraAny::Record(_) | MiraAny::RustRecord(_) => value
            .record_get(&key)?
            .unwrap_or(MiraAny::Nil)
            .into_element(),
        MiraAny::Extern(value) => {
            let result = value.get(&key)?.unwrap_or(MiraAny::Nil);
            assert_initialized(&result)?;
            Ok(result)
        }
        MiraAny::Module(module) => {
            let result = module.get_native(&key).unwrap_or(MiraAny::Nil);
            assert_initialized(&result)?;
            Ok(result)
        }
        _ => Ok(MiraAny::Nil),
    }
}

pub(crate) fn set(value: &MiraAny, key: &MiraAny, new_value: MiraAny) -> Result<()> {
    assert_initialized(value)?;
    assert_initialized(&new_value)?;
    let MiraAny::Extern(value) = value else {
        return Err(MiraError::runtime(format!("Expected extern, got {}", display(value))).into());
    };
    let key = to_string(key)?;
    if value.set(&key, new_value)? {
        Ok(())
    } else {
        Err(MiraError::runtime(format!("Extern field `{key}` is missing or read-only")).into())
    }
}

pub(crate) fn pick(value: &MiraAny, keys: &[String]) -> Result<MiraAny> {
    assert_initialized(value)?;
    if !matches!(value, MiraAny::Record(_) | MiraAny::RustRecord(_)) {
        return Ok(MiraAny::Record(IndexMap::new().into()));
    }
    let mut result = IndexMap::new();
    for key in keys {
        if has(value, &MiraAny::String(key.clone().into()))? {
            result.insert(key.clone(), get(value, key)?);
        }
    }
    Ok(MiraAny::Record(result.into()))
}

pub(crate) fn omit(value: &MiraAny, keys: &[String]) -> Result<MiraAny> {
    assert_initialized(value)?;
    if !matches!(value, MiraAny::Record(_) | MiraAny::RustRecord(_)) {
        return Ok(MiraAny::Record(IndexMap::new().into()));
    }
    let mut result = IndexMap::new();
    for key in value.record_keys()?.unwrap_or_default() {
        if !keys.contains(&key) {
            result.insert(key.clone(), get(value, &key)?);
        }
    }
    Ok(MiraAny::Record(result.into()))
}

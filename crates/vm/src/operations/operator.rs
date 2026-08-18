use super::*;

pub(crate) fn overload_number(a: &MiraAny, b: &MiraAny) -> bool {
    if matches!(a, MiraAny::Number(_)) || matches!(b, MiraAny::Number(_)) {
        return true;
    }
    if matches!(a, MiraAny::String(_)) || matches!(b, MiraAny::String(_)) {
        return false;
    }
    true
}

pub(crate) fn compare(a: &MiraAny, b: &MiraAny) -> Result<Option<Ordering>> {
    if overload_number(a, b) {
        let a = to_number(a)?;
        let b = to_number(b)?;
        Ok(a.partial_cmp(&b))
    } else {
        Ok(Some(to_string(a)?.cmp(&to_string(b)?)))
    }
}

pub(crate) fn approximately_equal(a: &MiraAny, b: &MiraAny) -> Result<bool> {
    if overload_number(a, b) {
        let a = to_number(a)?;
        let b = to_number(b)?;
        if a.is_nan() || b.is_nan() {
            return Ok(false);
        }
        if a == b {
            return Ok(true);
        }
        let difference = (a - b).abs();
        Ok(difference < 1e-15 || difference < a.abs().min(b.abs()) * 1e-15)
    } else {
        let a = to_string(a)?;
        let b = to_string(b)?;
        if a == b {
            return Ok(true);
        }
        Ok(a.to_lowercase().nfc().eq(b.to_lowercase().nfc()))
    }
}

pub(crate) fn in_value(needle: &MiraAny, value: &MiraAny) -> Result<bool> {
    assert_initialized(needle)?;
    assert_initialized(value)?;
    match value {
        MiraAny::Array(_) | MiraAny::RustArray(_) => Ok(iterable_array(value)?
            .iter()
            .any(|candidate| candidate == needle)),
        MiraAny::Record(_) | MiraAny::RustRecord(_) | MiraAny::Extern(_) | MiraAny::Module(_) => {
            has(value, &MiraAny::String(to_string(needle)?.into()))
        }
        _ => Ok(false),
    }
}

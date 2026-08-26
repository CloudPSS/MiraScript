use super::*;

pub(crate) fn overload_number(a: &MiraValue, b: &MiraValue) -> bool {
    if a.is_number() || b.is_number() {
        return true;
    }
    if a.is_string() || b.is_string() {
        return false;
    }
    true
}

fn equal_with(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    num_equal: impl Fn(f64, f64) -> bool,
    inner_equal: impl Fn(&mut Runtime, MiraValue, MiraValue) -> Result<bool>,
) -> Result<bool> {
    Ok(match (left.kind(), right.kind()) {
        (MiraValueKind::Number(left), MiraValueKind::Number(right)) => num_equal(left, right),
        _ if left.is_string() && right.is_string() => {
            left.as_str(runtime)? == right.as_str(runtime)?
        }
        (MiraValueKind::Array(_), MiraValueKind::Array(_)) => {
            let left_len = array_len(runtime, left)?.unwrap_or_default();
            let right_len = array_len(runtime, right)?.unwrap_or_default();
            if left_len != right_len {
                return Ok(false);
            }
            for index in 0..left_len {
                let Some(left_value) = array_get(runtime, left, index)? else {
                    return Ok(false);
                };
                let Some(right_value) = array_get(runtime, right, index)? else {
                    return Ok(false);
                };
                if !inner_equal(runtime, left_value, right_value)? {
                    return Ok(false);
                }
            }
            true
        }
        (MiraValueKind::Record(_), MiraValueKind::Record(_)) => {
            let left_keys = record_keys(runtime, left)?.unwrap_or_default();
            let right_keys = record_keys(runtime, right)?.unwrap_or_default();
            if left_keys.len() != right_keys.len() {
                return Ok(false);
            }
            for key in left_keys {
                let Some(left_value) = record_get(runtime, left, &key)? else {
                    return Ok(false);
                };
                let Some(right_value) = record_get(runtime, right, &key)? else {
                    return Ok(false);
                };
                if !inner_equal(runtime, left_value, right_value)? {
                    return Ok(false);
                }
            }
            true
        }
        (left, right) => left == right,
    })
}

pub(crate) fn equal(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Result<bool> {
    equal_with(
        runtime,
        left,
        right,
        |left, right| left == right,
        same_value,
    )
}

pub(crate) fn same_value(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Result<bool> {
    equal_with(
        runtime,
        left,
        right,
        |left, right| left == right || (left.is_nan() && right.is_nan()),
        same_value,
    )
}

pub(crate) fn host_equal(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Result<bool> {
    equal_with(
        runtime,
        left,
        right,
        |left, right| left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan()),
        host_equal,
    )
}

pub(crate) fn compare(
    runtime: &mut Runtime,
    a: MiraValue,
    b: MiraValue,
) -> Result<Option<Ordering>> {
    if overload_number(&a, &b) {
        let a = to_number(runtime, a)?;
        let b = to_number(runtime, b)?;
        Ok(a.partial_cmp(&b))
    } else {
        Ok(Some(to_string(runtime, a)?.cmp(&to_string(runtime, b)?)))
    }
}

pub(crate) fn approximately_equal(
    runtime: &mut Runtime,
    a: MiraValue,
    b: MiraValue,
) -> Result<bool> {
    if overload_number(&a, &b) {
        let a = to_number(runtime, a)?;
        let b = to_number(runtime, b)?;
        if a.is_nan() || b.is_nan() {
            return Ok(false);
        }
        if a == b {
            return Ok(true);
        }
        let difference = (a - b).abs();
        Ok(difference < 1e-15 || difference < a.abs().min(b.abs()) * 1e-15)
    } else {
        let a = to_string(runtime, a)?;
        let b = to_string(runtime, b)?;
        if a == b {
            return Ok(true);
        }
        use unicode_normalization::UnicodeNormalization;
        Ok(a.to_lowercase().nfc().eq(b.to_lowercase().nfc()))
    }
}

pub(crate) fn in_value(runtime: &mut Runtime, needle: MiraValue, value: MiraValue) -> Result<bool> {
    match value.kind() {
        MiraValueKind::Array(_) => {
            for candidate in iterable_array(runtime, value)? {
                if same_value(runtime, candidate, needle)? {
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

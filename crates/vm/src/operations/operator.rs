use super::*;

pub(crate) fn overload_number(a: MiraValue, b: MiraValue) -> bool {
    if matches!(a, MiraValue::Number(_)) || matches!(b, MiraValue::Number(_)) {
        return true;
    }
    if a.is_string() || b.is_string() {
        return false;
    }
    true
}

pub(crate) fn equal(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Result<bool> {
    Ok(match (left, right) {
        (MiraValue::Nil, MiraValue::Nil) => true,
        (MiraValue::Boolean(left), MiraValue::Boolean(right)) => left == right,
        (MiraValue::Number(left), MiraValue::Number(right)) => left == right,
        (left, right) if left.is_string() && right.is_string() => {
            to_string(runtime, left)? == to_string(runtime, right)?
        }
        (MiraValue::Array(left), MiraValue::Array(right)) => {
            let left = MiraValue::Array(left);
            let right = MiraValue::Array(right);
            let left_values = iterable_array(runtime, left)?;
            let right_values = iterable_array(runtime, right)?;
            if left_values.len() != right_values.len() {
                false
            } else {
                let mut same = true;
                for (left, right) in left_values.into_iter().zip(right_values) {
                    if !same_value(runtime, left, right)? {
                        same = false;
                        break;
                    }
                }
                same
            }
        }
        (MiraValue::Record(left), MiraValue::Record(right)) => {
            let left = MiraValue::Record(left);
            let right = MiraValue::Record(right);
            let left_keys = record_keys(runtime, left)?.unwrap_or_default();
            let right_keys = record_keys(runtime, right)?.unwrap_or_default();
            if left_keys.len() != right_keys.len() {
                false
            } else {
                let mut same = true;
                for key in left_keys {
                    let Some(left) = record_get(runtime, left, &key)? else {
                        same = false;
                        break;
                    };
                    let Some(right) = record_get(runtime, right, &key)? else {
                        same = false;
                        break;
                    };
                    if !same_value(runtime, left, right)? {
                        same = false;
                        break;
                    }
                }
                same
            }
        }
        _ => left == right,
    })
}

pub(crate) fn same_value(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Result<bool> {
    Ok(match (left, right) {
        (MiraValue::Nil, MiraValue::Nil) => true,
        (MiraValue::Boolean(left), MiraValue::Boolean(right)) => left == right,
        (MiraValue::Number(left), MiraValue::Number(right)) => {
            left == right || (left.is_nan() && right.is_nan())
        }
        (left, right) if left.is_string() && right.is_string() => {
            to_string(runtime, left)? == to_string(runtime, right)?
        }
        (MiraValue::Array(left), MiraValue::Array(right)) => {
            let left_values = iterable_array(runtime, MiraValue::Array(left))?;
            let right_values = iterable_array(runtime, MiraValue::Array(right))?;
            if left_values.len() != right_values.len() {
                false
            } else {
                let mut same = true;
                for (left, right) in left_values.into_iter().zip(right_values) {
                    if !same_value(runtime, left, right)? {
                        same = false;
                        break;
                    }
                }
                same
            }
        }
        (MiraValue::Record(left), MiraValue::Record(right)) => {
            let left = MiraValue::Record(left);
            let right = MiraValue::Record(right);
            let left_keys = record_keys(runtime, left)?.unwrap_or_default();
            let right_keys = record_keys(runtime, right)?.unwrap_or_default();
            if left_keys.len() != right_keys.len() {
                false
            } else {
                let mut same = true;
                for key in left_keys {
                    let Some(left) = record_get(runtime, left, &key)? else {
                        same = false;
                        break;
                    };
                    let Some(right) = record_get(runtime, right, &key)? else {
                        same = false;
                        break;
                    };
                    if !same_value(runtime, left, right)? {
                        same = false;
                        break;
                    }
                }
                same
            }
        }
        _ => left == right,
    })
}

pub(crate) fn host_equal(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Result<bool> {
    Ok(match (left, right) {
        (MiraValue::Nil, MiraValue::Nil) => true,
        (MiraValue::Boolean(left), MiraValue::Boolean(right)) => left == right,
        (MiraValue::Number(left), MiraValue::Number(right)) => {
            left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
        }
        (left, right) if left.is_string() && right.is_string() => {
            to_string(runtime, left)? == to_string(runtime, right)?
        }
        (MiraValue::Array(left), MiraValue::Array(right)) => {
            let left_values = iterable_array(runtime, MiraValue::Array(left))?;
            let right_values = iterable_array(runtime, MiraValue::Array(right))?;
            if left_values.len() != right_values.len() {
                false
            } else {
                let mut same = true;
                for (left, right) in left_values.into_iter().zip(right_values) {
                    if !host_equal(runtime, left, right)? {
                        same = false;
                        break;
                    }
                }
                same
            }
        }
        (MiraValue::Record(left), MiraValue::Record(right)) => {
            let left = MiraValue::Record(left);
            let right = MiraValue::Record(right);
            let left_keys = record_keys(runtime, left)?.unwrap_or_default();
            let right_keys = record_keys(runtime, right)?.unwrap_or_default();
            if left_keys.len() != right_keys.len() {
                false
            } else {
                let mut same = true;
                for key in left_keys {
                    let Some(left) = record_get(runtime, left, &key)? else {
                        same = false;
                        break;
                    };
                    let Some(right) = record_get(runtime, right, &key)? else {
                        same = false;
                        break;
                    };
                    if !host_equal(runtime, left, right)? {
                        same = false;
                        break;
                    }
                }
                same
            }
        }
        _ => left == right,
    })
}

pub(crate) fn compare(
    runtime: &mut Runtime,
    a: MiraValue,
    b: MiraValue,
) -> Result<Option<Ordering>> {
    if overload_number(a, b) {
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
    if overload_number(a, b) {
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
    match value {
        MiraValue::Array(_) => {
            for candidate in iterable_array(runtime, value)? {
                if same_value(runtime, candidate, needle)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MiraValue::Record(_) | MiraValue::Module(_) => {
            let key = to_string(runtime, needle)?;
            has(runtime, value, MiraValue::Nil, Some(&key))
        }
        _ => Ok(false),
    }
}

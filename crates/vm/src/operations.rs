use std::cmp::Ordering;

use indexmap::IndexMap;
use unicode_normalization::UnicodeNormalization;

use crate::{MiraAny, MiraError, Result};

pub(crate) fn assert_initialized(value: &MiraAny) -> Result<()> {
    if matches!(value, MiraAny::Uninitialized) {
        Err(MiraError::runtime("Uninitialized value"))
    } else {
        Ok(())
    }
}

pub(crate) fn assert_non_nil(value: &MiraAny) -> Result<()> {
    assert_initialized(value)?;
    if matches!(value, MiraAny::Nil) {
        Err(MiraError::runtime("Expected non-nil value"))
    } else {
        Ok(())
    }
}

pub(crate) fn to_boolean(value: &MiraAny) -> Result<bool> {
    assert_initialized(value)?;
    match value {
        MiraAny::Boolean(value) => Ok(*value),
        _ => Err(MiraError::runtime(format!(
            "Failed to convert value to boolean: {}",
            display(value)
        ))),
    }
}

pub(crate) fn to_number(value: &MiraAny) -> Result<f64> {
    assert_initialized(value)?;
    match value {
        MiraAny::Number(value) => Ok(*value),
        MiraAny::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
        MiraAny::String(value) => parse_number(value).ok_or_else(|| {
            MiraError::runtime(format!(
                "Failed to convert value to number: {}",
                display(&MiraAny::String(value.clone()))
            ))
        }),
        _ => Err(MiraError::runtime(format!(
            "Failed to convert value to number: {}",
            display(value)
        ))),
    }
}

fn parse_number(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match value {
        "inf" | "+inf" | "Infinity" | "+Infinity" => return Some(f64::INFINITY),
        "-inf" | "-Infinity" => return Some(f64::NEG_INFINITY),
        "nan" | "NaN" => return Some(f64::NAN),
        _ => {}
    }
    let (sign, unsigned) = if let Some(value) = value.strip_prefix('-') {
        (-1.0, value)
    } else if let Some(value) = value.strip_prefix('+') {
        (1.0, value)
    } else {
        (1.0, value)
    };
    if unsigned.is_empty() || !unsigned.as_bytes()[0].is_ascii_digit() {
        return None;
    }
    let parsed = if let Some(value) = unsigned
        .strip_prefix("0b")
        .or_else(|| unsigned.strip_prefix("0B"))
    {
        if value.is_empty() || !value.bytes().all(|byte| matches!(byte, b'0' | b'1')) {
            return None;
        }
        u64::from_str_radix(value, 2).ok().map(|value| value as f64)
    } else if let Some(value) = unsigned
        .strip_prefix("0o")
        .or_else(|| unsigned.strip_prefix("0O"))
    {
        if value.is_empty() || !value.bytes().all(|byte| matches!(byte, b'0'..=b'7')) {
            return None;
        }
        u64::from_str_radix(value, 8).ok().map(|value| value as f64)
    } else if let Some(value) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        u64::from_str_radix(value, 16)
            .ok()
            .map(|value| value as f64)
    } else if valid_decimal(unsigned) {
        unsigned.parse::<f64>().ok()
    } else {
        None
    }?;
    Some(sign * parsed)
}

fn valid_decimal(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index < bytes.len() && bytes[index] == b'.' {
        index += 1;
        let fraction = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == fraction {
            return false;
        }
    }
    if index < bytes.len() && matches!(bytes[index], b'e' | b'E') {
        index += 1;
        if index < bytes.len() && matches!(bytes[index], b'+' | b'-') {
            index += 1;
        }
        let exponent = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == exponent {
            return false;
        }
    }
    index == bytes.len()
}

pub(crate) fn number_to_string(value: f64, minus_zero: bool) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value == f64::INFINITY {
        return "inf".into();
    }
    if value == f64::NEG_INFINITY {
        return "-inf".into();
    }
    if value == 0.0 {
        if minus_zero && value.is_sign_negative() {
            return "-0".into();
        }
        return "0".into();
    }
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        return javascript_exponent(format!("{value:e}"));
    }
    value.to_string()
}

pub(crate) fn to_string(value: &MiraAny) -> Result<String> {
    assert_initialized(value)?;
    match value {
        MiraAny::String(value) => Ok(value.clone()),
        MiraAny::Nil => Ok(String::new()),
        value => inner_to_string(value, false),
    }
}

fn inner_to_string(value: &MiraAny, braces: bool) -> Result<String> {
    match value {
        MiraAny::Uninitialized | MiraAny::Nil => Ok("nil".into()),
        MiraAny::Boolean(value) => Ok(value.to_string()),
        MiraAny::Number(value) => Ok(number_to_string(*value, false)),
        MiraAny::String(value) => Ok(value.clone()),
        MiraAny::Function(function) => Ok(match function.name() {
            Some(name) => format!("<function {name}>"),
            None => "<function>".into(),
        }),
        MiraAny::Module(module) => Ok(format!("<module {}>", module.name())),
        MiraAny::Extern(value) => Ok(format!("<extern {}>", value.tag()?)),
        MiraAny::Array(_) | MiraAny::RustArray(_) => {
            let length = value.array_len()?.unwrap_or(0);
            let mut parts = Vec::with_capacity(length);
            for index in 0..length {
                parts.push(inner_to_string(
                    &value.array_get(index)?.unwrap_or(MiraAny::Nil),
                    true,
                )?);
            }
            let body = parts.join(", ");
            Ok(if braces { format!("[{body}]") } else { body })
        }
        MiraAny::Record(_) | MiraAny::RustRecord(_) => {
            let mut parts = Vec::new();
            for key in value.record_keys()?.unwrap_or_default() {
                let item = value.record_get(&key)?.unwrap_or(MiraAny::Nil);
                parts.push(format!("{key}: {}", inner_to_string(&item, true)?));
            }
            let body = parts.join(", ");
            Ok(if braces { format!("({body})") } else { body })
        }
    }
}

pub(crate) fn display(value: &MiraAny) -> String {
    inner_to_string(value, true).unwrap_or_else(|_| format!("<{}>", value.type_name()))
}

pub(crate) fn format_value(value: &MiraAny, format: Option<&str>) -> Result<String> {
    let format = format.unwrap_or_default().trim();
    if let MiraAny::Number(value) = value {
        if !value.is_finite() {
            return Ok(number_to_string(*value, false));
        }
        if let Some(digits) = format.strip_prefix('.').filter(|digits| {
            !digits.is_empty() && digits.bytes().all(|digit| digit.is_ascii_digit())
        }) {
            let digits = digits.parse::<usize>().unwrap_or(100).min(100);
            return Ok(format!("{:.*}", digits, value));
        }
        if *value == 0.0 {
            return Ok("0".into());
        }
        let plain = number_to_string(*value, false);
        if *value != 0.0 && (value.abs() >= 1000.0 || value.abs() < 0.001) {
            let shortest = javascript_exponent(format!("{value:e}"));
            let precision = javascript_exponent(format!("{value:.5e}"));
            return Ok(if shortest.len() < precision.len() {
                shortest
            } else {
                precision
            });
        }
        let exponent = value.abs().log10().floor() as i32;
        let compact = format!("{:.*}", (5 - exponent).max(0) as usize, value);
        return Ok(if compact.len() < plain.len() {
            compact
        } else {
            plain
        });
    }
    to_string(value)
}

fn javascript_exponent(value: String) -> String {
    let Some((mantissa, exponent)) = value.split_once('e') else {
        return value;
    };
    if exponent.starts_with('-') || exponent.starts_with('+') {
        value
    } else {
        format!("{mantissa}e+{exponent}")
    }
}

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
    get_value(value, &MiraAny::String(key.to_owned()))
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
        return Err(MiraError::runtime(format!(
            "Expected extern, got {}",
            display(value)
        )));
    };
    let key = to_string(key)?;
    if value.set(&key, new_value)? {
        Ok(())
    } else {
        Err(MiraError::runtime(format!(
            "Extern field `{key}` is missing or read-only"
        )))
    }
}

pub(crate) fn length(value: &MiraAny) -> Result<usize> {
    assert_initialized(value)?;
    if let Some(length) = value.array_len()? {
        return Ok(length);
    }
    match value {
        MiraAny::Record(_) | MiraAny::RustRecord(_) => {
            Ok(value.record_keys()?.unwrap_or_default().len())
        }
        MiraAny::Extern(value) => Ok(value.keys()?.len()),
        MiraAny::Module(module) => Ok(module.keys().len()),
        _ => Err(MiraError::runtime(format!(
            "Value has no length: {}",
            display(value)
        ))),
    }
}

pub(crate) fn iterable(value: &MiraAny) -> Result<Vec<MiraAny>> {
    assert_initialized(value)?;
    match value {
        MiraAny::Array(_) | MiraAny::RustArray(_) => materialize_array(value),
        MiraAny::Record(_) | MiraAny::RustRecord(_) => Ok(value
            .record_keys()?
            .unwrap_or_default()
            .into_iter()
            .map(MiraAny::String)
            .collect()),
        MiraAny::Extern(value) => Ok(value.keys()?.into_iter().map(MiraAny::String).collect()),
        MiraAny::Module(module) => Ok(module.keys().into_iter().map(MiraAny::String).collect()),
        _ => Err(MiraError::runtime(format!(
            "Value is not iterable: {}",
            display(value)
        ))),
    }
}

pub(crate) fn materialize_array(value: &MiraAny) -> Result<Vec<MiraAny>> {
    let Some(length) = value.array_len()? else {
        return Err(MiraError::runtime(format!(
            "Expected array, got {}",
            display(value)
        )));
    };
    (0..length)
        .map(|index| Ok(value.array_get(index)?.unwrap_or(MiraAny::Nil)))
        .collect()
}

pub(crate) fn array_spread(value: &MiraAny) -> Result<Vec<MiraAny>> {
    assert_initialized(value)?;
    match value {
        MiraAny::Nil => Ok(Vec::new()),
        MiraAny::Array(_) | MiraAny::RustArray(_) => materialize_array(value),
        MiraAny::Extern(value) => {
            if let Some(iterable) = value.iterate()? {
                Ok(iterable)
            } else if let Some(length) = value.array_len()? {
                (0..length)
                    .map(|index| Ok(value.get_index(index)?.unwrap_or(MiraAny::Nil)))
                    .collect()
            } else {
                Err(MiraError::runtime(format!(
                    "Expected array, iterable extern or nil, got {}",
                    display(&MiraAny::Extern(value.clone()))
                )))
            }
        }
        _ => Err(MiraError::runtime(format!(
            "Expected array, iterable extern or nil, got {}",
            display(value)
        ))),
    }
}

pub(crate) fn record_spread(value: &MiraAny) -> Result<IndexMap<String, MiraAny>> {
    assert_initialized(value)?;
    let mut result = IndexMap::new();
    match value {
        MiraAny::Nil => {}
        MiraAny::Record(_) | MiraAny::RustRecord(_) => {
            for key in value.record_keys()?.unwrap_or_default() {
                let item = value
                    .record_get(&key)?
                    .unwrap_or(MiraAny::Nil)
                    .into_element()?;
                result.insert(key, item);
            }
        }
        MiraAny::Array(_) | MiraAny::RustArray(_) => {
            for (index, item) in materialize_array(value)?.into_iter().enumerate() {
                result.insert(index.to_string(), item.into_element()?);
            }
        }
        MiraAny::Extern(value) => {
            for key in value.keys()? {
                if let Some(item) = value.get(&key)? {
                    result.insert(key, item.into_element()?);
                }
            }
        }
        _ => {
            return Err(MiraError::runtime(format!(
                "Expected record, array, extern or nil, got {}",
                display(value)
            )));
        }
    }
    Ok(result)
}

pub(crate) fn pick(value: &MiraAny, keys: &[String]) -> Result<MiraAny> {
    assert_initialized(value)?;
    if !matches!(value, MiraAny::Record(_) | MiraAny::RustRecord(_)) {
        return Ok(MiraAny::Record(IndexMap::new()));
    }
    let mut result = IndexMap::new();
    for key in keys {
        if has(value, &MiraAny::String(key.clone()))? {
            result.insert(key.clone(), get(value, key)?);
        }
    }
    Ok(MiraAny::Record(result))
}

pub(crate) fn omit(value: &MiraAny, keys: &[String]) -> Result<MiraAny> {
    assert_initialized(value)?;
    if !matches!(value, MiraAny::Record(_) | MiraAny::RustRecord(_)) {
        return Ok(MiraAny::Record(IndexMap::new()));
    }
    let mut result = IndexMap::new();
    for key in value.record_keys()?.unwrap_or_default() {
        if !keys.contains(&key) {
            result.insert(key.clone(), get(value, &key)?);
        }
    }
    Ok(MiraAny::Record(result))
}

pub(crate) fn slice(
    value: &MiraAny,
    start: Option<&MiraAny>,
    end: Option<&MiraAny>,
    exclusive: bool,
) -> Result<MiraAny> {
    assert_initialized(value)?;
    let array = materialize_array(value)?;
    let length = array.len() as i64;
    let mut start = match start {
        Some(value) => to_number(value).unwrap_or(f64::NAN),
        None => 0.0,
    };
    let mut end = match end {
        Some(value) => to_number(value).unwrap_or(f64::NAN),
        None => (length - if exclusive { 0 } else { 1 }) as f64,
    };
    if start.is_nan() {
        start = 0.0;
    } else if start < 0.0 {
        start += length as f64;
    }
    if end.is_nan() {
        end = (length - if exclusive { 0 } else { 1 }) as f64;
    } else if end < 0.0 {
        end += length as f64;
    }
    let start = (start.ceil() as i64).clamp(0, length) as usize;
    let end = if exclusive || end.fract() != 0.0 || !end.is_finite() {
        end.ceil() as i64
    } else {
        end as i64 + 1
    }
    .clamp(0, length) as usize;
    Ok(MiraAny::Array(if start >= end {
        Vec::new()
    } else {
        array[start..end].to_vec()
    }))
}

pub(crate) fn array_range(
    start: &MiraAny,
    end: &MiraAny,
    exclusive: bool,
    max_len: usize,
) -> Result<Vec<MiraAny>> {
    let start = to_number(start)?;
    let end = to_number(end)?;
    if !start.is_finite() || !end.is_finite() || start > end {
        return Ok(Vec::new());
    }
    let length = if exclusive {
        (end - start).ceil()
    } else {
        (end - start + 1.0).floor()
    };
    if length > max_len as f64 {
        return Err(MiraError::runtime(format!(
            "Array length exceeds maximum limit of {max_len}"
        )));
    }
    Ok((0..length.max(0.0) as usize)
        .map(|index| MiraAny::Number(start + index as f64))
        .collect())
}

pub(crate) fn in_value(needle: &MiraAny, value: &MiraAny) -> Result<bool> {
    assert_initialized(needle)?;
    assert_initialized(value)?;
    match value {
        MiraAny::Array(_) | MiraAny::RustArray(_) => Ok(materialize_array(value)?
            .iter()
            .any(|candidate| candidate == needle)),
        MiraAny::Record(_) | MiraAny::RustRecord(_) | MiraAny::Extern(_) | MiraAny::Module(_) => {
            has(value, &MiraAny::String(to_string(needle)?))
        }
        _ => Ok(false),
    }
}

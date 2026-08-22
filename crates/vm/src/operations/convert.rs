use crate::value::ANONYMOUS_FN_NAME;

use super::*;

pub(crate) fn to_boolean(value: MiraValue) -> Result<bool> {
    match value.kind() {
        MiraValueKind::Boolean(value) => Ok(value),
        _ => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "boolean",
            actual: value.value_type(),
        })),
    }
}

pub(crate) fn to_number(runtime: &Runtime, value: MiraValue) -> Result<f64> {
    match value.kind() {
        MiraValueKind::Number(value) => Ok(value),
        MiraValueKind::Boolean(value) => Ok(if value { 1.0 } else { 0.0 }),
        MiraValueKind::String(handle) => {
            parse_number(runtime.get_string(handle)?).ok_or_else(|| {
                MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                    expected: "number-convertible value",
                    actual: value.value_type(),
                })
            })
        }
        MiraValueKind::StaticStr(value) => parse_number(value).ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                expected: "number-convertible value",
                actual: crate::MiraType::String,
            })
        }),
        _ => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "number-convertible value",
            actual: value.value_type(),
        })),
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

pub(crate) fn to_string(runtime: &mut Runtime, value: MiraValue) -> Result<String> {
    match value.kind() {
        MiraValueKind::String(handle) => Ok(runtime.get_string(handle)?.to_owned()),
        MiraValueKind::StaticStr(value) => Ok(value.to_string()),
        MiraValueKind::Nil => Ok(String::new()),
        _ => inner_to_string(runtime, value, false),
    }
}

pub(super) fn inner_to_string(
    runtime: &mut Runtime,
    value: MiraValue,
    braces: bool,
) -> Result<String> {
    match value.kind() {
        MiraValueKind::Nil => Ok("nil".into()),
        MiraValueKind::Boolean(value) => Ok(value.to_string()),
        MiraValueKind::Number(value) => Ok(number_to_string(value, false)),
        MiraValueKind::String(handle) => Ok(runtime.get_string(handle)?.to_owned()),
        MiraValueKind::StaticStr(value) => Ok(value.to_string()),
        MiraValueKind::Function(handle) => {
            let function = runtime.get_function_dyn(handle)?;
            let name = function.name();
            Ok(if name == ANONYMOUS_FN_NAME {
                "<function>".into()
            } else {
                format!("<function {}>", name)
            })
        }
        MiraValueKind::Module(handle) => {
            let name = runtime.get_module_dyn(handle)?.name().to_owned();
            Ok(format!("<module {name}>"))
        }
        MiraValueKind::Array(_) => {
            let values = iterable_array(runtime, value)?;
            let mut parts = Vec::with_capacity(values.len());
            for item in values {
                parts.push(inner_to_string(runtime, item, true)?);
            }
            let body = parts.join(", ");
            Ok(if braces { format!("[{body}]") } else { body })
        }
        MiraValueKind::Record(_) => {
            let keys = record_keys(runtime, value)?.unwrap_or_default();
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                let item = record_get(runtime, value, &key)?.unwrap_or_else(MiraValue::nil);
                parts.push(format!("{key}: {}", inner_to_string(runtime, item, true)?));
            }
            let body = parts.join(", ");
            Ok(if braces { format!("({body})") } else { body })
        }
        MiraValueKind::Extern(_) => Ok("<extern>".into()),
    }
}

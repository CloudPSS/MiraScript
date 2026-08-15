use super::*;

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

pub(super) fn inner_to_string(value: &MiraAny, braces: bool) -> Result<String> {
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

use super::*;

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

pub(super) fn javascript_exponent(value: String) -> String {
    let Some((mantissa, exponent)) = value.split_once('e') else {
        return value;
    };
    if exponent.starts_with('-') || exponent.starts_with('+') {
        value
    } else {
        format!("{mantissa}e+{exponent}")
    }
}

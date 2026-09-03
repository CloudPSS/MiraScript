use super::*;

pub(crate) fn slice(
    runtime: &mut Runtime,
    value: MiraValue,
    start: Option<MiraValue>,
    end: Option<MiraValue>,
    exclusive: bool,
) -> Result<MiraValue> {
    let iter = iterate_array(runtime, value)?;
    let length = iter.len() as i64;
    let mut start = match start {
        Some(value) => to_number(runtime, value).unwrap_or(f64::NAN),
        None => 0.0,
    };
    let mut end = match end {
        Some(value) => to_number(runtime, value).unwrap_or(f64::NAN),
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
    let mut result = Vec::with_capacity(end.saturating_sub(start));
    for entry in iter {
        let index = entry.index();
        let item = entry.get(runtime)?;
        if index >= start && index < end {
            result.push(item);
        }
    }
    runtime.insert(result)
}

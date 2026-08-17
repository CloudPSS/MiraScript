use super::*;

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
    Ok(MiraAny::Array(
        (if start >= end {
            Vec::new()
        } else {
            array[start..end].to_vec()
        })
        .into(),
    ))
}

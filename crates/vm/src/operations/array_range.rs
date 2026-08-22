use super::*;

pub(crate) fn array_range(
    runtime: &Runtime,
    start: MiraValue,
    end: MiraValue,
    exclusive: bool,
    max_len: usize,
) -> Result<Vec<MiraValue>> {
    let start = to_number(runtime, start)?;
    let end = to_number(runtime, end)?;
    if !start.is_finite() || !end.is_finite() || start > end {
        return Ok(Vec::new());
    }
    let length = if exclusive {
        (end - start).ceil()
    } else {
        (end - start + 1.0).floor()
    };
    if length > max_len as f64 {
        return Err(MiraError::runtime(RuntimeErrorKind::ArrayLimit {
            requested: length as usize,
            max: max_len,
        }));
    }
    Ok((0..length.max(0.0) as usize)
        .map(|index| MiraValue::number(start + index as f64))
        .collect())
}

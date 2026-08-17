use super::*;

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
        return Err(
            MiraError::runtime(format!("Array length exceeds maximum limit of {max_len}")).into(),
        );
    }
    Ok((0..length.max(0.0) as usize)
        .map(|index| MiraAny::Number(start + index as f64))
        .collect())
}

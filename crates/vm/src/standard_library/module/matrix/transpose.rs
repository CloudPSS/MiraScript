use crate::standard_library::required;
use crate::{MiraValue, Result, Runtime};

use super::helpers::{as_matrix, from_matrix, shape};

pub(super) fn transpose(call: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
    let value = *required(args, 0, "matrix")?;
    let dimensions = shape(call, value)?;
    if dimensions.len() < 2 {
        return Ok(value);
    }
    let matrix = as_matrix(call, value)?;
    let rows = (0..dimensions[1])
        .map(|column| {
            (0..dimensions[0])
                .map(|row| {
                    matrix
                        .get(row)
                        .and_then(|row| row.get(column))
                        .cloned()
                        .unwrap_or(MiraValue::NIL)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    from_matrix(call, rows)
}

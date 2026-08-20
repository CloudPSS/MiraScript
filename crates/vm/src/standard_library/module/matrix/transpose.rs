use crate::standard_library::required;
use crate::{MiraAny, Result, Runtime};

use super::helpers::{as_matrix, shape};

pub(super) fn transpose(_call: &mut Runtime<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let value = required(args, 0, "matrix")?;
    let dimensions = shape(value)?;
    if dimensions.len() < 2 {
        return Ok(value.clone());
    }
    let matrix = as_matrix(value)?;
    Ok(MiraAny::Array(
        (0..dimensions[1])
            .map(|column| {
                MiraAny::Array(
                    (0..dimensions[0])
                        .map(|row| {
                            matrix
                                .get(row)
                                .and_then(|row| row.get(column))
                                .cloned()
                                .unwrap_or(MiraAny::Nil)
                        })
                        .collect(),
                )
            })
            .collect(),
    ))
}

use crate::standard_library::required;
use crate::{MiraAny, MiraError, Result, Runtime, operations};

use super::helpers::shape;

pub(super) fn filled(call: &mut Runtime<'_>, args: &[MiraAny], value: f64) -> Result<MiraAny> {
    let dimensions = dimensions(args, call.options().max_array_len)?;
    if dimensions.is_empty() {
        return Ok(MiraAny::Array(Vec::new().into()));
    }
    let mut result = MiraAny::Number(value);
    for length in dimensions.into_iter().rev() {
        call.checkpoint()?;
        result = MiraAny::Array(vec![result; length].into());
    }
    Ok(result)
}

pub(super) fn dimensions(args: &[MiraAny], max_len: usize) -> Result<Vec<usize>> {
    let values = if args.len() == 1 && args[0].array_len()?.is_some() {
        operations::iterable_array(&args[0])?
    } else {
        args.to_vec()
    };
    values
        .iter()
        .map(|value| {
            let value = operations::to_number(value)?;
            if !value.is_finite() || value <= -1.0 || value.trunc() as usize > max_len {
                return Err(MiraError::runtime("Invalid matrix size"));
            }
            Ok(value.trunc() as usize)
        })
        .collect()
}

pub(super) fn identity(call: &mut Runtime<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let dimensions = dimensions(args, call.options().max_array_len)?;
    if dimensions.is_empty() {
        return Ok(MiraAny::Array(Vec::new().into()));
    }
    if dimensions.len() > 2 {
        return Err(MiraError::runtime("Invalid matrix size"));
    }
    let rows = dimensions[0];
    let columns = *dimensions.get(1).unwrap_or(&rows);
    Ok(MiraAny::Array(
        (0..rows)
            .map(|row| {
                MiraAny::Array(
                    (0..columns)
                        .map(|column| MiraAny::Number(if row == column { 1.0 } else { 0.0 }))
                        .collect(),
                )
            })
            .collect(),
    ))
}

pub(super) fn diagonal(_call: &mut Runtime<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let value = required(args, 0, "x")?;
    let values = operations::iterable_array(value)?;
    let offset = match args.get(1) {
        None => 0,
        Some(value) => {
            let offset = operations::to_number(value)?;
            if !offset.is_finite() || offset.abs() > 9_007_199_254_740_991.0 {
                return Err(MiraError::runtime(
                    "Argument `offset` cannot be converted to integer",
                ));
            }
            offset.trunc() as isize
        }
    };
    if shape(value)?.len() == 2 {
        let mut result = Vec::new();
        for (row, values) in values.iter().enumerate() {
            let column = row as isize + offset;
            if column < 0 {
                continue;
            }
            let row = operations::iterable_array(values)?;
            if column as usize >= row.len() {
                break;
            }
            result.push(row[column as usize].clone());
        }
        return Ok(MiraAny::Array(result.into()));
    }
    let rows = values.len() + offset.min(0).unsigned_abs();
    let columns = values.len() + offset.max(0) as usize;
    Ok(MiraAny::Array(
        (0..rows)
            .map(|row| {
                MiraAny::Array(
                    (0..columns)
                        .map(|column| {
                            if row as isize + offset == column as isize {
                                values[if offset >= 0 { row } else { column }].clone()
                            } else {
                                MiraAny::Number(0.0)
                            }
                        })
                        .collect(),
                )
            })
            .collect(),
    ))
}

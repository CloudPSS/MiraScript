use crate::standard_library::required;
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

use super::helpers::{from_matrix, shape};

pub(in crate::standard_library::module) fn filled<const VALUE: i32>(
    call: &mut Runtime,
    args: &[MiraValue],
) -> Result<MiraValue> {
    let dimensions = dimensions(call, args, call.options().max_array_len)?;
    if dimensions.is_empty() {
        return call.insert(Vec::<MiraValue>::new());
    }
    let mut result = MiraValue::from(VALUE);
    for length in dimensions.into_iter().rev() {
        call.checkpoint()?;
        result = call.insert(vec![result; length])?;
    }
    Ok(result)
}

pub(super) fn dimensions(
    runtime: &mut Runtime,
    args: &[MiraValue],
    max_len: usize,
) -> Result<Vec<usize>> {
    if args.len() == 1 && operations::array_len(runtime, args[0])?.is_some() {
        let iter = operations::iterate_array(runtime, args[0])?;
        let mut dimensions = Vec::with_capacity(iter.len());
        for entry in iter {
            let value = entry.get(runtime)?;
            dimensions.push(dimension(runtime, value, max_len)?);
        }
        Ok(dimensions)
    } else {
        args.iter()
            .map(|value| dimension(runtime, *value, max_len))
            .collect()
    }
}

fn dimension(runtime: &Runtime, value: MiraValue, max_len: usize) -> Result<usize> {
    let value = operations::to_number(runtime, value)?;
    if !value.is_finite() || value <= -1.0 || value.trunc() as usize > max_len {
        return Err(MiraError::runtime(RuntimeErrorKind::InvalidMatrixSize));
    }
    Ok(value.trunc() as usize)
}

pub(in crate::standard_library::module) fn identity(
    call: &mut Runtime,
    args: &[MiraValue],
) -> Result<MiraValue> {
    let dimensions = dimensions(call, args, call.options().max_array_len)?;
    if dimensions.is_empty() {
        return call.insert(Vec::<MiraValue>::new());
    }
    if dimensions.len() > 2 {
        return Err(MiraError::runtime(RuntimeErrorKind::InvalidMatrixSize));
    }
    let rows = dimensions[0];
    let columns = *dimensions.get(1).unwrap_or(&rows);
    let values = (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| MiraValue::number(if row == column { 1.0 } else { 0.0 }))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    from_matrix(call, values)
}

pub(in crate::standard_library::module) fn diagonal(
    call: &mut Runtime,
    args: &[MiraValue],
) -> Result<MiraValue> {
    let value = *required(args, 0, "x")?;
    let offset = match args.get(1) {
        None => 0,
        Some(value) => {
            let offset = operations::to_number(call, *value)?;
            if !offset.is_finite() || offset.abs() > 9_007_199_254_740_991.0 {
                return Err(MiraError::runtime(
                    RuntimeErrorKind::InvalidIntegerArgument {
                        name: "offset",
                        constraint: "a finite representable integer",
                    },
                ));
            }
            offset.trunc() as isize
        }
    };
    let dimensions = shape(call, value)?;
    if dimensions.len() == 2 {
        let iter = operations::iterate_array(call, value)?;
        let mut result = Vec::with_capacity(iter.len());
        for entry in iter {
            let row = entry.index();
            let values = entry.get(call)?;
            let column = row as isize + offset;
            if column < 0 {
                continue;
            }
            let Some(row_length) = operations::array_len(call, values)? else {
                break;
            };
            if column as usize >= row_length {
                break;
            }
            result.push(
                operations::array_get(call, values, column as usize)?.unwrap_or(MiraValue::NIL),
            );
        }
        return call.insert(result);
    }
    let values = operations::iterable_array(call, value)?;
    let rows = values.len() + offset.min(0).unsigned_abs();
    let columns = values.len() + offset.max(0) as usize;
    let result = (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| {
                    if row as isize + offset == column as isize {
                        values[if offset >= 0 { row } else { column }]
                    } else {
                        MiraValue::number(0.0)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    from_matrix(call, result)
}

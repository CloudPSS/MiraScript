use crate::standard_library::required;
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

use super::helpers::{as_matrix, from_matrix, numeric, shape};

pub(in crate::standard_library::module) fn numeric_entrywise(
    runtime: &mut Runtime,
    args: &[MiraValue],
    operation: impl Fn(f64, f64) -> f64,
) -> Result<MiraValue> {
    let left = *required(args, 0, "a")?;
    let right = *required(args, 1, "b")?;
    entrywise(runtime, left, right, &mut |runtime, a, b| {
        Ok(MiraValue::number(operation(
            numeric(runtime, a)?,
            numeric(runtime, b)?,
        )))
    })
}

pub(in crate::standard_library::module) fn entrywise(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    operation: &mut impl FnMut(&mut Runtime, MiraValue, MiraValue) -> Result<MiraValue>,
) -> Result<MiraValue> {
    let left_shape = shape(runtime, left)?;
    let right_shape = shape(runtime, right)?;
    if left_shape.is_empty() && right_shape.is_empty() {
        return operation(runtime, left, right);
    }
    if left_shape.is_empty() {
        return broadcast_scalar(runtime, right, &right_shape, &mut |runtime, value| {
            operation(runtime, left, value)
        });
    }
    if right_shape.is_empty() {
        return broadcast_scalar(runtime, left, &left_shape, &mut |runtime, value| {
            operation(runtime, value, right)
        });
    }
    if left_shape.len() == 1 && right_shape.len() == 1 {
        let (iterated, other, iterated_is_left) = if left_shape[0] >= right_shape[0] {
            (left, right, true)
        } else {
            (right, left, false)
        };
        let other_handle = other.as_array().expect("one-dimensional value is an array");
        let other_length = if iterated_is_left {
            right_shape[0]
        } else {
            left_shape[0]
        };
        let iter = operations::iterate_array(runtime, iterated)?;
        let mut result = Vec::with_capacity(iter.len());
        for entry in iter {
            let index = entry.index();
            let value = entry.get(runtime)?;
            let other = if index < other_length {
                other_handle.get(runtime, index)?
            } else {
                MiraValue::NIL
            };
            let (left, right) = if iterated_is_left {
                (value, other)
            } else {
                (other, value)
            };
            result.push(operation(runtime, left, right)?);
        }
        return runtime.insert(result);
    }

    let left_matrix = as_matrix(runtime, left)?;
    let right_matrix = as_matrix(runtime, right)?;
    let rows = left_matrix.len().max(right_matrix.len());
    let left_columns = left_matrix.iter().map(Vec::len).max().unwrap_or(0);
    let right_columns = right_matrix.iter().map(Vec::len).max().unwrap_or(0);
    let columns = left_columns.max(right_columns);
    let mut result = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut output = Vec::with_capacity(columns);
        for column in 0..columns {
            let left_row = if left_matrix.len() == 1 { 0 } else { row };
            let right_row = if right_matrix.len() == 1 { 0 } else { row };
            let left_column = if left_columns == 1 { 0 } else { column };
            let right_column = if right_columns == 1 { 0 } else { column };
            output.push(operation(
                runtime,
                left_matrix
                    .get(left_row)
                    .and_then(|row| row.get(left_column))
                    .cloned()
                    .unwrap_or(MiraValue::NIL),
                right_matrix
                    .get(right_row)
                    .and_then(|row| row.get(right_column))
                    .cloned()
                    .unwrap_or(MiraValue::NIL),
            )?);
        }
        result.push(output);
    }
    from_matrix(runtime, result)
}

fn broadcast_scalar(
    runtime: &mut Runtime,
    value: MiraValue,
    dimensions: &[usize],
    operation: &mut impl FnMut(&mut Runtime, MiraValue) -> Result<MiraValue>,
) -> Result<MiraValue> {
    if dimensions.len() == 1 {
        let iter = operations::iterate_array(runtime, value)?;
        let mut result = Vec::with_capacity(iter.len());
        for entry in iter {
            let value = entry.get(runtime)?;
            result.push(operation(runtime, value)?);
        }
        return runtime.insert(result);
    }
    let matrix = as_matrix(runtime, value)?;
    let mut result = Vec::with_capacity(dimensions[0]);
    for row in 0..dimensions[0] {
        let mut output = Vec::with_capacity(dimensions[1]);
        for column in 0..dimensions[1] {
            let value = matrix
                .get(row)
                .and_then(|row| row.get(column))
                .cloned()
                .unwrap_or(MiraValue::NIL);
            output.push(operation(runtime, value)?);
        }
        result.push(output);
    }
    from_matrix(runtime, result)
}

pub(in crate::standard_library::module) fn map_nested(
    runtime: &mut Runtime,
    value: MiraValue,
    operation: &mut impl FnMut(&mut Runtime, MiraValue) -> Result<MiraValue>,
) -> Result<MiraValue> {
    let iter = operations::iterate_array(runtime, value)?;
    let mut result = Vec::with_capacity(iter.len());
    for entry in iter {
        let value = entry.get(runtime)?;
        if operations::array_len(runtime, value)?.is_some() {
            result.push(map_nested(runtime, value, operation)?);
        } else {
            result.push(operation(runtime, value)?);
        }
    }
    runtime.insert(result)
}

pub(in crate::standard_library::module) fn multiply(
    runtime: &mut Runtime,
    args: &[MiraValue],
) -> Result<MiraValue> {
    let left = *required(args, 0, "a")?;
    let right = *required(args, 1, "b")?;
    let left_shape = shape(runtime, left)?;
    let right_shape = shape(runtime, right)?;
    match (left_shape.len(), right_shape.len()) {
        (0, _) | (_, 0) => numeric_entrywise(runtime, args, |a, b| a * b),
        (1, 1) => {
            let (iterated, other, iterated_is_left) = if left_shape[0] >= right_shape[0] {
                (left, right, true)
            } else {
                (right, left, false)
            };
            let other_handle = other.as_array().expect("one-dimensional value is an array");
            let other_length = if iterated_is_left {
                right_shape[0]
            } else {
                left_shape[0]
            };
            let mut sum = 0.0;
            for entry in operations::iterate_array(runtime, iterated)? {
                let index = entry.index();
                let value = entry.get(runtime)?;
                let other = if index < other_length {
                    other_handle.get(runtime, index)?
                } else {
                    MiraValue::NIL
                };
                let (left, right) = if iterated_is_left {
                    (value, other)
                } else {
                    (other, value)
                };
                sum += numeric(runtime, left)? * numeric(runtime, right)?;
            }
            Ok(MiraValue::number(sum))
        }
        (2, 2) => {
            if left_shape[1] != right_shape[0] {
                return Err(MiraError::runtime(
                    RuntimeErrorKind::IncompatibleMatrixDimensions,
                ));
            }
            let left = as_matrix(runtime, left)?;
            let right = as_matrix(runtime, right)?;
            let right_columns: Vec<Vec<_>> = (0..right_shape[1])
                .map(|column| right.iter().map(|row| row[column]).collect())
                .collect();
            let mut result = Vec::new();
            for left_row in &left {
                let mut output = Vec::new();
                for right_column in &right_columns {
                    let mut sum = 0.0;
                    for (left_value, right_value) in left_row.iter().zip(right_column) {
                        sum += numeric(runtime, *left_value)? * numeric(runtime, *right_value)?;
                    }
                    output.push(MiraValue::number(sum));
                }
                result.push(output);
            }
            from_matrix(runtime, result)
        }
        (1, 2) => {
            if left_shape[0] != right_shape[0] {
                return Err(MiraError::runtime(
                    RuntimeErrorKind::IncompatibleMatrixDimensions,
                ));
            }
            let left = operations::iterable_array(runtime, left)?;
            let right = as_matrix(runtime, right)?;
            let right_columns: Vec<Vec<_>> = (0..right_shape[1])
                .map(|column| right.iter().map(|row| row[column]).collect())
                .collect();
            let mut result = Vec::new();
            for right_column in &right_columns {
                let mut sum = 0.0;
                for (left_value, right_value) in left.iter().zip(right_column) {
                    sum += numeric(runtime, *left_value)? * numeric(runtime, *right_value)?;
                }
                result.push(MiraValue::number(sum));
            }
            runtime.insert(result)
        }
        (2, 1) => {
            if left_shape[1] != right_shape[0] {
                return Err(MiraError::runtime(
                    RuntimeErrorKind::IncompatibleMatrixDimensions,
                ));
            }
            let left = as_matrix(runtime, left)?;
            let right = operations::iterable_array(runtime, right)?;
            let mut result = Vec::new();
            for left_row in &left {
                let mut sum = 0.0;
                for (left_value, right_value) in left_row.iter().zip(&right) {
                    sum += numeric(runtime, *left_value)? * numeric(runtime, *right_value)?;
                }
                result.push(MiraValue::number(sum));
            }
            runtime.insert(result)
        }
        _ => unreachable!(),
    }
}

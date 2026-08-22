use crate::standard_library::required;
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

use super::helpers::{as_matrix, from_matrix, numeric, shape};

pub(super) fn numeric_entrywise(
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

pub(super) fn entrywise(
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
        let left = operations::iterable_array(runtime, left)?;
        let right = operations::iterable_array(runtime, right)?;
        let length = left.len().max(right.len());
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            result.push(operation(
                runtime,
                left.get(index).cloned().unwrap_or(MiraValue::nil()),
                right.get(index).cloned().unwrap_or(MiraValue::nil()),
            )?);
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
                    .unwrap_or(MiraValue::nil()),
                right_matrix
                    .get(right_row)
                    .and_then(|row| row.get(right_column))
                    .cloned()
                    .unwrap_or(MiraValue::nil()),
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
        let values = operations::iterable_array(runtime, value)?;
        let mut result = Vec::with_capacity(values.len());
        for value in values {
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
                .unwrap_or(MiraValue::nil());
            output.push(operation(runtime, value)?);
        }
        result.push(output);
    }
    from_matrix(runtime, result)
}

pub(super) fn map_nested(
    runtime: &mut Runtime,
    value: MiraValue,
    operation: &mut impl FnMut(&mut Runtime, MiraValue) -> Result<MiraValue>,
) -> Result<MiraValue> {
    let values = operations::iterable_array(runtime, value)?;
    let mut result = Vec::with_capacity(values.len());
    for value in values {
        if operations::array_len(runtime, value)?.is_some() {
            result.push(map_nested(runtime, value, operation)?);
        } else {
            result.push(operation(runtime, value)?);
        }
    }
    runtime.insert(result)
}

pub(super) fn multiply(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
    let left = *required(args, 0, "a")?;
    let right = *required(args, 1, "b")?;
    let left_shape = shape(runtime, left)?;
    let right_shape = shape(runtime, right)?;
    match (left_shape.len(), right_shape.len()) {
        (0, _) | (_, 0) => numeric_entrywise(runtime, args, |a, b| a * b),
        (1, 1) => {
            let left = operations::iterable_array(runtime, left)?;
            let right = operations::iterable_array(runtime, right)?;
            let length = left.len().max(right.len());
            let mut sum = 0.0;
            for index in 0..length {
                sum += numeric(
                    runtime,
                    left.get(index).cloned().unwrap_or(MiraValue::nil()),
                )? * numeric(
                    runtime,
                    right.get(index).cloned().unwrap_or(MiraValue::nil()),
                )?;
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

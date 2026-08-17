use crate::standard_library::required;
use crate::{MiraAny, MiraCallContext, MiraError, Result, operations};

use super::helpers::{as_matrix, numeric, shape};

pub(super) fn numeric_entrywise(
    args: &[MiraAny],
    operation: impl Fn(f64, f64) -> f64,
) -> Result<MiraAny> {
    let left = required(args, 0, "a")?;
    let right = required(args, 1, "b")?;
    entrywise(left, right, &mut |a, b| {
        Ok(MiraAny::Number(operation(numeric(&a)?, numeric(&b)?)))
    })
}

pub(super) fn entrywise(
    left: &MiraAny,
    right: &MiraAny,
    operation: &mut impl FnMut(MiraAny, MiraAny) -> Result<MiraAny>,
) -> Result<MiraAny> {
    let left_shape = shape(left)?;
    let right_shape = shape(right)?;
    if left_shape.is_empty() && right_shape.is_empty() {
        return operation(left.clone(), right.clone());
    }
    if left_shape.is_empty() {
        return broadcast_scalar(right, &right_shape, &mut |value| {
            operation(left.clone(), value)
        });
    }
    if right_shape.is_empty() {
        return broadcast_scalar(left, &left_shape, &mut |value| {
            operation(value, right.clone())
        });
    }
    if left_shape.len() == 1 && right_shape.len() == 1 {
        let left = operations::materialize_array(left)?;
        let right = operations::materialize_array(right)?;
        let length = left.len().max(right.len());
        return Ok(MiraAny::Array(
            (0..length)
                .map(|index| {
                    operation(
                        left.get(index).cloned().unwrap_or(MiraAny::Nil),
                        right.get(index).cloned().unwrap_or(MiraAny::Nil),
                    )
                })
                .collect::<Result<Vec<_>>>()?
                .into(),
        ));
    }

    let left_matrix = as_matrix(left)?;
    let right_matrix = as_matrix(right)?;
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
                left_matrix
                    .get(left_row)
                    .and_then(|row| row.get(left_column))
                    .cloned()
                    .unwrap_or(MiraAny::Nil),
                right_matrix
                    .get(right_row)
                    .and_then(|row| row.get(right_column))
                    .cloned()
                    .unwrap_or(MiraAny::Nil),
            )?);
        }
        result.push(MiraAny::Array(output.into()));
    }
    Ok(MiraAny::Array(result.into()))
}

pub(super) fn broadcast_scalar(
    value: &MiraAny,
    dimensions: &[usize],
    operation: &mut impl FnMut(MiraAny) -> Result<MiraAny>,
) -> Result<MiraAny> {
    if dimensions.len() == 1 {
        return Ok(MiraAny::Array(
            operations::materialize_array(value)?
                .into_iter()
                .map(operation)
                .collect::<Result<Vec<_>>>()?
                .into(),
        ));
    }
    let matrix = as_matrix(value)?;
    Ok(MiraAny::Array(
        (0..dimensions[0])
            .map(|row| {
                Ok(MiraAny::Array(
                    (0..dimensions[1])
                        .map(|column| {
                            operation(
                                matrix
                                    .get(row)
                                    .and_then(|row| row.get(column))
                                    .cloned()
                                    .unwrap_or(MiraAny::Nil),
                            )
                        })
                        .collect::<Result<Vec<_>>>()?
                        .into(),
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .into(),
    ))
}

pub(super) fn map_nested(
    value: &MiraAny,
    operation: &mut impl FnMut(MiraAny) -> Result<MiraAny>,
) -> Result<MiraAny> {
    let values = operations::materialize_array(value)?;
    Ok(MiraAny::Array(
        values
            .into_iter()
            .map(|value| {
                if value.array_len()?.is_some() {
                    map_nested(&value, operation)
                } else {
                    operation(value)
                }
            })
            .collect::<Result<Vec<_>>>()?
            .into(),
    ))
}

pub(super) fn multiply(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let left = required(args, 0, "a")?;
    let right = required(args, 1, "b")?;
    let left_shape = shape(left)?;
    let right_shape = shape(right)?;
    match (left_shape.len(), right_shape.len()) {
        (0, _) | (_, 0) => numeric_entrywise(args, |a, b| a * b),
        (1, 1) => {
            let left = operations::materialize_array(left)?;
            let right = operations::materialize_array(right)?;
            let length = left.len().max(right.len());
            let mut sum = 0.0;
            for index in 0..length {
                sum += numeric(left.get(index).unwrap_or(&MiraAny::Nil))?
                    * numeric(right.get(index).unwrap_or(&MiraAny::Nil))?;
            }
            Ok(MiraAny::Number(sum))
        }
        (2, 2) => {
            if left_shape[1] != right_shape[0] {
                return Err(MiraError::runtime("Incompatible matrix dimensions"));
            }
            let left = as_matrix(left)?;
            let right = as_matrix(right)?;
            let right_columns: Vec<Vec<_>> = (0..right_shape[1])
                .map(|column| right.iter().map(|row| row[column].clone()).collect())
                .collect();
            let mut result = Vec::new();
            for left_row in &left {
                let mut output = Vec::new();
                for right_column in &right_columns {
                    let mut sum = 0.0;
                    for (left_value, right_value) in left_row.iter().zip(right_column) {
                        sum += numeric(left_value)? * numeric(right_value)?;
                    }
                    output.push(MiraAny::Number(sum));
                }
                result.push(MiraAny::Array(output.into()));
            }
            Ok(MiraAny::Array(result.into()))
        }
        (1, 2) => {
            if left_shape[0] != right_shape[0] {
                return Err(MiraError::runtime("Incompatible matrix dimensions"));
            }
            let left = operations::materialize_array(left)?;
            let right = as_matrix(right)?;
            let right_columns: Vec<Vec<_>> = (0..right_shape[1])
                .map(|column| right.iter().map(|row| row[column].clone()).collect())
                .collect();
            let mut result = Vec::new();
            for right_column in &right_columns {
                let mut sum = 0.0;
                for (left_value, right_value) in left.iter().zip(right_column) {
                    sum += numeric(left_value)? * numeric(right_value)?;
                }
                result.push(MiraAny::Number(sum));
            }
            Ok(MiraAny::Array(result.into()))
        }
        (2, 1) => {
            if left_shape[1] != right_shape[0] {
                return Err(MiraError::runtime("Incompatible matrix dimensions"));
            }
            let left = as_matrix(left)?;
            let right = operations::materialize_array(right)?;
            let mut result = Vec::new();
            for left_row in &left {
                let mut sum = 0.0;
                for (left_value, right_value) in left_row.iter().zip(&right) {
                    sum += numeric(left_value)? * numeric(right_value)?;
                }
                result.push(MiraAny::Number(sum));
            }
            Ok(MiraAny::Array(result.into()))
        }
        _ => unreachable!(),
    }
}

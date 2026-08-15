use crate::standard_library::required;
use crate::{MiraAny, MiraCallContext, MiraError, Result};

use super::helpers::{as_matrix, numeric, shape};
use super::map_nested;

pub(super) fn invert(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let value = required(args, 0, "a")?;
    let dimensions = shape(value)?;
    if dimensions.is_empty() {
        return Ok(MiraAny::Number(1.0 / numeric(value)?));
    }
    if dimensions.len() == 1 {
        return map_nested(value, &mut |value| {
            Ok(MiraAny::Number(1.0 / numeric(&value)?))
        });
    }
    if dimensions[0] != dimensions[1] {
        return Err(MiraError::runtime("Matrix must be square"));
    }
    let size = dimensions[0];
    let matrix = as_matrix(value)?;
    if size == 1 {
        return Ok(MiraAny::Array(vec![MiraAny::Array(vec![MiraAny::Number(
            1.0 / numeric(
                matrix
                    .first()
                    .and_then(|row| row.first())
                    .unwrap_or(&MiraAny::Nil),
            )?,
        )])]));
    }
    if size == 2 {
        let a = numeric(&matrix[0][0])?;
        let b = numeric(&matrix[0][1])?;
        let c = numeric(&matrix[1][0])?;
        let d = numeric(&matrix[1][1])?;
        let determinant = a * d - b * c;
        return Ok(MiraAny::Array(vec![
            MiraAny::Array(vec![
                MiraAny::Number(d / determinant),
                MiraAny::Number(-b / determinant),
            ]),
            MiraAny::Array(vec![
                MiraAny::Number(-c / determinant),
                MiraAny::Number(a / determinant),
            ]),
        ]));
    }
    let mut left = vec![vec![0.0; size]; size];
    let mut right = vec![vec![0.0; size]; size];
    for row in 0..size {
        for column in 0..size {
            left[row][column] = numeric(
                matrix
                    .get(row)
                    .and_then(|row| row.get(column))
                    .unwrap_or(&MiraAny::Nil),
            )?;
            right[row][column] = if row == column { 1.0 } else { 0.0 };
        }
    }
    for column in 0..size {
        let mut pivot = column;
        let mut largest = left[column][column].abs();
        for (row, values) in left.iter().enumerate().skip(column + 1) {
            if values[column].abs() > largest {
                largest = values[column].abs();
                pivot = row;
            }
        }
        left.swap(column, pivot);
        right.swap(column, pivot);
        let pivot_left = left[column].clone();
        let pivot_right = right[column].clone();
        let pivot_value = pivot_left[column];
        for (row, (left_row, right_row)) in left.iter_mut().zip(right.iter_mut()).enumerate() {
            if row != column {
                if left_row[column] != 0.0 {
                    let factor = -left_row[column] / pivot_value;
                    for (value, pivot) in left_row[column..].iter_mut().zip(&pivot_left[column..]) {
                        *value += factor * pivot;
                    }
                    for (value, pivot) in right_row.iter_mut().zip(&pivot_right) {
                        *value += factor * pivot;
                    }
                }
            } else {
                for value in &mut left_row[column..] {
                    *value /= pivot_value;
                }
                for value in right_row {
                    *value /= pivot_value;
                }
            }
        }
    }
    Ok(MiraAny::Array(
        right
            .into_iter()
            .map(|row| MiraAny::Array(row.into_iter().map(MiraAny::Number).collect()))
            .collect(),
    ))
}

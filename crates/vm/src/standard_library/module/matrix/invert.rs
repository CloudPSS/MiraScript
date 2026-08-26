use crate::standard_library::required;
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind};

use super::helpers::{as_matrix, from_matrix, numeric, shape};
use super::map_nested;

pub(in crate::standard_library::module) fn invert(
    call: &mut Runtime,
    args: &[MiraValue],
) -> Result<MiraValue> {
    let value = *required(args, 0, "a")?;
    let dimensions = shape(call, value)?;
    if dimensions.is_empty() {
        return Ok(MiraValue::number(1.0 / numeric(call, value)?));
    }
    if dimensions.len() == 1 {
        return map_nested(call, value, &mut |runtime, value| {
            Ok(MiraValue::number(1.0 / numeric(runtime, value)?))
        });
    }
    if dimensions[0] != dimensions[1] {
        return Err(MiraError::runtime(RuntimeErrorKind::MatrixMustBeSquare));
    }
    let size = dimensions[0];
    let matrix = as_matrix(call, value)?;
    if size == 1 {
        let value = matrix
            .first()
            .and_then(|row| row.first())
            .cloned()
            .unwrap_or(MiraValue::NIL);
        return from_matrix(
            call,
            vec![vec![MiraValue::number(1.0 / numeric(call, value)?)]],
        );
    }
    if size == 2 {
        let a = numeric(call, matrix[0][0])?;
        let b = numeric(call, matrix[0][1])?;
        let c = numeric(call, matrix[1][0])?;
        let d = numeric(call, matrix[1][1])?;
        let determinant = a * d - b * c;
        return from_matrix(
            call,
            vec![
                vec![
                    MiraValue::number(d / determinant),
                    MiraValue::number(-b / determinant),
                ],
                vec![
                    MiraValue::number(-c / determinant),
                    MiraValue::number(a / determinant),
                ],
            ],
        );
    }
    let mut left = vec![vec![0.0; size]; size];
    let mut right = vec![vec![0.0; size]; size];
    for row in 0..size {
        for column in 0..size {
            left[row][column] = numeric(
                call,
                matrix
                    .get(row)
                    .and_then(|row| row.get(column))
                    .cloned()
                    .unwrap_or(MiraValue::NIL),
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
    from_matrix(
        call,
        right
            .into_iter()
            .map(|row| row.into_iter().map(MiraValue::number).collect())
            .collect(),
    )
}

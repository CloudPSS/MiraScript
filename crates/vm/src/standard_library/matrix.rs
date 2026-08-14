use indexmap::IndexMap;

use crate::{MiraAny, MiraCallContext, MiraError, MiraModule, Result, operations};

use super::{const_value, is_callable, native, required};

pub(super) fn module() -> MiraAny {
    let mut values = IndexMap::new();
    values.insert(
        "zeros".into(),
        native("matrix.zeros", |call, args| filled(call, args, 0.0)),
    );
    values.insert(
        "ones".into(),
        native("matrix.ones", |call, args| filled(call, args, 1.0)),
    );
    values.insert("identity".into(), native("matrix.identity", identity));
    values.insert("diagonal".into(), native("matrix.diagonal", diagonal));
    values.insert(
        "size".into(),
        native("matrix.size", |_, args| {
            Ok(MiraAny::Array(
                shape(required(args, 0, "matrix")?)?
                    .into_iter()
                    .map(|value| MiraAny::Number(value as f64))
                    .collect(),
            ))
        }),
    );
    values.insert("transpose".into(), native("matrix.transpose", transpose));
    values.insert("invert".into(), native("matrix.invert", invert));
    values.insert(
        "add".into(),
        native("matrix.add", |_, args| {
            numeric_entrywise(args, |a, b| a + b)
        }),
    );
    values.insert(
        "subtract".into(),
        native("matrix.subtract", |_, args| {
            numeric_entrywise(args, |a, b| a - b)
        }),
    );
    values.insert(
        "entrywise_multiply".into(),
        native("matrix.entrywise_multiply", |_, args| {
            numeric_entrywise(args, |a, b| a * b)
        }),
    );
    values.insert(
        "entrywise_divide".into(),
        native("matrix.entrywise_divide", |_, args| {
            numeric_entrywise(args, |a, b| a / b)
        }),
    );
    values.insert("multiply".into(), native("matrix.multiply", multiply));
    values.insert(
        "entrywise".into(),
        native("matrix.entrywise", |call, args| {
            let left = required(args, 0, "a")?;
            let right = required(args, 1, "b")?;
            let function = required(args, 2, "f")?;
            if !is_callable(function)? {
                return Err(MiraError::runtime("Argument `f` is not callable"));
            }
            entrywise(left, right, &mut |a, b| {
                call.checkpoint()?;
                const_value(call.call(function, &[a, b])?)
            })
        }),
    );
    MiraAny::Module(MiraModule::new("matrix", values))
}

fn shape(value: &MiraAny) -> Result<Vec<usize>> {
    let Some(rows) = value.array_len()? else {
        return Ok(Vec::new());
    };
    if rows == 0 {
        return Ok(vec![0]);
    }
    let mut columns = 0;
    for row in operations::materialize_array(value)? {
        let Some(length) = row.array_len()? else {
            return Ok(vec![rows]);
        };
        columns = columns.max(length);
    }
    Ok(vec![rows, columns])
}

fn numeric(value: &MiraAny) -> Result<f64> {
    operations::to_number(value)
}

fn numeric_entrywise(args: &[MiraAny], operation: impl Fn(f64, f64) -> f64) -> Result<MiraAny> {
    let left = required(args, 0, "a")?;
    let right = required(args, 1, "b")?;
    entrywise(left, right, &mut |a, b| {
        Ok(MiraAny::Number(operation(numeric(&a)?, numeric(&b)?)))
    })
}

fn entrywise(
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
                .collect::<Result<Vec<_>>>()?,
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
        result.push(MiraAny::Array(output));
    }
    Ok(MiraAny::Array(result))
}

fn broadcast_scalar(
    value: &MiraAny,
    dimensions: &[usize],
    operation: &mut impl FnMut(MiraAny) -> Result<MiraAny>,
) -> Result<MiraAny> {
    if dimensions.len() == 1 {
        return Ok(MiraAny::Array(
            operations::materialize_array(value)?
                .into_iter()
                .map(operation)
                .collect::<Result<Vec<_>>>()?,
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
                        .collect::<Result<Vec<_>>>()?,
                ))
            })
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn map_nested(
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
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn as_matrix(value: &MiraAny) -> Result<Vec<Vec<MiraAny>>> {
    let values = operations::materialize_array(value)?;
    if shape(value)?.len() == 1 {
        Ok(vec![values])
    } else {
        values.iter().map(operations::materialize_array).collect()
    }
}

fn filled(call: &mut MiraCallContext<'_>, args: &[MiraAny], value: f64) -> Result<MiraAny> {
    let dimensions = dimensions(args, call.options().max_array_len)?;
    if dimensions.is_empty() {
        return Ok(MiraAny::Array(Vec::new()));
    }
    let mut result = MiraAny::Number(value);
    for length in dimensions.into_iter().rev() {
        call.checkpoint()?;
        result = MiraAny::Array(vec![result; length]);
    }
    Ok(result)
}

fn dimensions(args: &[MiraAny], max_len: usize) -> Result<Vec<usize>> {
    let values = if args.len() == 1 && args[0].array_len()?.is_some() {
        operations::materialize_array(&args[0])?
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

fn identity(call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let dimensions = dimensions(args, call.options().max_array_len)?;
    if dimensions.is_empty() {
        return Ok(MiraAny::Array(Vec::new()));
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

fn diagonal(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    let value = required(args, 0, "x")?;
    let values = operations::materialize_array(value)?;
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
            let row = operations::materialize_array(values)?;
            if column as usize >= row.len() {
                break;
            }
            result.push(row[column as usize].clone());
        }
        return Ok(MiraAny::Array(result));
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

fn transpose(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
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

fn invert(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
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

fn multiply(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
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
                result.push(MiraAny::Array(output));
            }
            Ok(MiraAny::Array(result))
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
            Ok(MiraAny::Array(result))
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
            Ok(MiraAny::Array(result))
        }
        _ => unreachable!(),
    }
}

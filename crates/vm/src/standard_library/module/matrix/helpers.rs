use crate::{MiraValue, Result, Runtime, operations};

pub(super) fn shape(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<usize>> {
    let Some(rows) = operations::array_len(runtime, value)? else {
        return Ok(Vec::new());
    };
    if rows == 0 {
        return Ok(vec![0]);
    }
    let mut columns = 0;
    for row in operations::iterable_array(runtime, value)? {
        let Some(length) = operations::array_len(runtime, row)? else {
            return Ok(vec![rows]);
        };
        columns = columns.max(length);
    }
    Ok(vec![rows, columns])
}

pub(super) fn numeric(runtime: &Runtime, value: MiraValue) -> Result<f64> {
    operations::to_number(runtime, value)
}

pub(super) fn as_matrix(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<Vec<MiraValue>>> {
    let values = operations::iterable_array(runtime, value)?;
    if shape(runtime, value)?.len() == 1 {
        Ok(vec![values])
    } else {
        values
            .into_iter()
            .map(|value| operations::iterable_array(runtime, value))
            .collect()
    }
}

pub(super) fn from_matrix(runtime: &mut Runtime, rows: Vec<Vec<MiraValue>>) -> Result<MiraValue> {
    let rows = rows
        .into_iter()
        .map(|row| runtime.insert(row))
        .collect::<Result<Vec<_>>>()?;
    runtime.insert(rows)
}

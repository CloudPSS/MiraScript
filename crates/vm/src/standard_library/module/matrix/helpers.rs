use crate::{MiraValue, Result, Runtime, operations};

pub(super) fn shape(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<usize>> {
    let Some(rows) = operations::array_len(runtime, value)? else {
        return Ok(Vec::new());
    };
    if rows == 0 {
        return Ok(vec![0]);
    }
    let mut columns = 0;
    for entry in operations::iterate_array(runtime, value)? {
        let row = entry.get(runtime)?;
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
    if shape(runtime, value)?.len() == 1 {
        Ok(vec![operations::iterable_array(runtime, value)?])
    } else {
        let iter = operations::iterate_array(runtime, value)?;
        let mut rows = Vec::with_capacity(iter.len());
        for entry in iter {
            let value = entry.get(runtime)?;
            rows.push(operations::iterable_array(runtime, value)?);
        }
        Ok(rows)
    }
}

pub(super) fn from_matrix(runtime: &mut Runtime, rows: Vec<Vec<MiraValue>>) -> Result<MiraValue> {
    let rows = rows
        .into_iter()
        .map(|row| runtime.insert(row))
        .collect::<Result<Vec<_>>>()?;
    runtime.insert(rows)
}

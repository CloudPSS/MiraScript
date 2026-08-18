use crate::{MiraAny, Result, operations};

pub(super) fn shape(value: &MiraAny) -> Result<Vec<usize>> {
    let Some(rows) = value.array_len()? else {
        return Ok(Vec::new());
    };
    if rows == 0 {
        return Ok(vec![0]);
    }
    let mut columns = 0;
    for row in operations::iterable_array(value)? {
        let Some(length) = row.array_len()? else {
            return Ok(vec![rows]);
        };
        columns = columns.max(length);
    }
    Ok(vec![rows, columns])
}

pub(super) fn numeric(value: &MiraAny) -> Result<f64> {
    operations::to_number(value)
}

pub(super) fn as_matrix(value: &MiraAny) -> Result<Vec<Vec<MiraAny>>> {
    let values = operations::iterable_array(value)?;
    if shape(value)?.len() == 1 {
        Ok(vec![values])
    } else {
        values.iter().map(operations::iterable_array).collect()
    }
}

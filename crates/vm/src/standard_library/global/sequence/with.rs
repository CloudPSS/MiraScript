use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "with", |call, args| {
        update_with(
            required(args, 0, "data")?,
            &args[1..],
            call.options().max_array_len,
        )
    });
}

fn update_with(data: &MiraAny, entries: &[MiraAny], max_len: usize) -> Result<MiraAny> {
    if !entries.len().is_multiple_of(2) {
        return Err(MiraError::runtime("Expected even number of entries"));
    }
    let mut result = match Data::from_value(data)? {
        Data::Array(values) => MiraAny::Array(values.into()),
        Data::Record(values) => MiraAny::Record(values.into()),
        Data::Primitive(_) => {
            return Err(MiraError::runtime("Argument `data` is not array | record"));
        }
    };
    for pair in entries.chunks_exact(2) {
        let path = if pair[0].array_len()?.is_some() {
            operations::iterable_array(&pair[0])?
        } else if pair[0] == MiraAny::Nil {
            continue;
        } else {
            vec![pair[0].clone()]
        };
        if path.is_empty() || path.contains(&MiraAny::Nil) {
            continue;
        }
        result = set_path(result, &path, const_value(pair[1].clone())?, max_len)?;
    }
    Ok(result)
}

fn set_path(
    mut data: MiraAny,
    path: &[MiraAny],
    value: MiraAny,
    max_len: usize,
) -> Result<MiraAny> {
    if path.is_empty() {
        return Ok(value);
    }
    match &mut data {
        MiraAny::Array(values) => {
            let index = array_index(&path[0], max_len)?;
            while values.len() <= index {
                values.push(MiraAny::Nil);
            }
            let current = values[index].clone();
            values[index] = set_path(
                container_for(&current, path.get(1)),
                &path[1..],
                value,
                max_len,
            )?;
        }
        MiraAny::Record(values) => {
            let key = operations::to_string(&path[0])?;
            let current = values.get(&key).cloned().unwrap_or(MiraAny::Nil);
            values.insert(
                key,
                set_path(
                    container_for(&current, path.get(1)),
                    &path[1..],
                    value,
                    max_len,
                )?,
            );
        }
        _ => {
            data = container_for(&data, path.first());
            return set_path(data, path, value, max_len);
        }
    }
    Ok(data)
}

fn container_for(current: &MiraAny, next: Option<&MiraAny>) -> MiraAny {
    if matches!(current, MiraAny::Array(_) | MiraAny::Record(_)) {
        return current.clone();
    }
    if next.is_some_and(
        |value| matches!(value, MiraAny::Number(number) if number.fract() == 0.0 && *number >= 0.0),
    ) {
        MiraAny::Array(Vec::new().into())
    } else {
        MiraAny::Record(IndexMap::new().into())
    }
}

fn array_index(value: &MiraAny, max_len: usize) -> Result<usize> {
    let index = operations::to_number(value)?;
    if !index.is_finite() || index < 0.0 {
        return Err(MiraError::runtime(
            "Array index must be a non-negative integer",
        ));
    }
    let index = index.trunc() as usize;
    if index >= max_len {
        return Err(MiraError::runtime(format!(
            "Array index exceeds maximum limit of {max_len}"
        )));
    }
    Ok(index)
}

pub(super) fn array_length(value: &MiraAny, max_len: usize) -> Result<usize> {
    let length = operations::to_number(value)?;
    if !length.is_finite() || length <= -1.0 {
        return Err(MiraError::runtime(
            "Array length must be a non-negative integer",
        ));
    }
    let length = length.trunc() as usize;
    if length > max_len {
        return Err(MiraError::runtime(format!(
            "Array length exceeds maximum limit of {max_len}"
        )));
    }
    Ok(length)
}

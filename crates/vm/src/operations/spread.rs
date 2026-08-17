use super::*;

pub(crate) fn array_spread(value: &MiraAny) -> Result<Vec<MiraAny>> {
    assert_initialized(value)?;
    match value {
        MiraAny::Nil => Ok(Vec::new()),
        MiraAny::Array(_) | MiraAny::RustArray(_) => materialize_array(value),
        MiraAny::Extern(value) => {
            if let Some(iterable) = value.iterate()? {
                Ok(iterable)
            } else if let Some(length) = value.array_len()? {
                (0..length)
                    .map(|index| Ok(value.get_index(index)?.unwrap_or(MiraAny::Nil)))
                    .collect()
            } else {
                Err(MiraError::runtime(format!(
                    "Expected array, iterable extern or nil, got {}",
                    display(&MiraAny::Extern(value.clone()))
                ))
                .into())
            }
        }
        _ => Err(MiraError::runtime(format!(
            "Expected array, iterable extern or nil, got {}",
            display(value)
        ))
        .into()),
    }
}

pub(crate) fn record_spread(value: &MiraAny) -> Result<IndexMap<String, MiraAny>> {
    assert_initialized(value)?;
    let mut result = IndexMap::new();
    match value {
        MiraAny::Nil => {}
        MiraAny::Record(_) | MiraAny::RustRecord(_) => {
            for key in value.record_keys()?.unwrap_or_default() {
                let item = value
                    .record_get(&key)?
                    .unwrap_or(MiraAny::Nil)
                    .into_element()?;
                result.insert(key, item);
            }
        }
        MiraAny::Array(_) | MiraAny::RustArray(_) => {
            for (index, item) in materialize_array(value)?.into_iter().enumerate() {
                result.insert(index.to_string(), item.into_element()?);
            }
        }
        MiraAny::Extern(value) => {
            for key in value.keys()? {
                if let Some(item) = value.get(&key)? {
                    result.insert(key, item.into_element()?);
                }
            }
        }
        _ => {
            return Err(MiraError::runtime(format!(
                "Expected record, array, extern or nil, got {}",
                display(value)
            ))
            .into());
        }
    }
    Ok(result)
}

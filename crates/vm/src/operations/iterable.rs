use super::*;

pub(crate) fn length(value: &MiraAny) -> Result<usize> {
    assert_initialized(value)?;
    if let Some(length) = value.array_len()? {
        return Ok(length);
    }
    match value {
        MiraAny::Record(_) | MiraAny::RustRecord(_) => {
            Ok(value.record_keys()?.unwrap_or_default().len())
        }
        MiraAny::Extern(value) => Ok(value.keys()?.len()),
        MiraAny::Module(module) => Ok(module.keys().len()),
        _ => Err(MiraError::runtime(format!(
            "Value has no length: {}",
            display(value)
        ))),
    }
}

pub(crate) fn iterable(value: &MiraAny) -> Result<Vec<MiraAny>> {
    assert_initialized(value)?;
    match value {
        MiraAny::Array(_) | MiraAny::RustArray(_) => iterable_array(value),
        MiraAny::Record(_) | MiraAny::RustRecord(_) => Ok(value
            .record_keys()?
            .unwrap_or_default()
            .into_iter()
            .map(MiraAny::from)
            .collect()),
        MiraAny::Extern(value) => Ok(value.keys()?.into_iter().map(MiraAny::from).collect()),
        MiraAny::Module(module) => Ok(module.keys().into_iter().map(MiraAny::from).collect()),
        _ => Err(MiraError::runtime(format!(
            "Value is not iterable: {}",
            display(value)
        ))),
    }
}

struct ArrayIterator {
    value: MiraAny,
    index: usize,
    length: usize,
}

impl Iterator for ArrayIterator {
    type Item = Result<MiraAny>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }
        let result = self.value.array_get(self.index);
        self.index += 1;
        Some(result.map(|v| v.unwrap_or(MiraAny::Nil)))
    }
}

pub(crate) fn iterable_array(value: &MiraAny) -> Result<Vec<MiraAny>> {
    let Some(length) = value.array_len()? else {
        return Err(MiraError::runtime(format!(
            "Expected array, got {}",
            display(value)
        )));
    };
    (0..length)
        .map(|index| Ok(value.array_get(index)?.unwrap_or(MiraAny::Nil)))
        .collect()
}

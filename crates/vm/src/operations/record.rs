use super::*;

pub(crate) fn has(
    runtime: &mut Runtime,
    value: MiraValue,
    key: MiraValue,
    known_key: Option<&str>,
) -> Result<bool> {
    let key = match known_key {
        Some(key) => key.to_owned(),
        None => to_string(runtime, key)?,
    };
    match value.kind() {
        MiraValueKind::Array(_) => {
            let Ok(index) = key.parse::<usize>() else {
                return Ok(false);
            };
            Ok(index < array_len(runtime, value)?.unwrap_or(0))
        }
        MiraValueKind::Record(handle) => Ok(handle.index_of(runtime, &key)?.is_some()),
        MiraValueKind::Module(handle) => Ok(handle.index_of(runtime, &key)?.is_some()),
        _ => Ok(false),
    }
}

pub(crate) fn has_i(runtime: &mut Runtime, value: MiraValue, key: i64) -> Result<bool> {
    let (MiraValueKind::Record(handle), Ok(key)) = (value.kind(), u32::try_from(key)) else {
        return has(runtime, value, MiraValue::number(key as f64), None);
    };
    Ok(runtime.get_record_dyn(handle)?.index_of_i(key).is_some())
}

pub(crate) fn get(runtime: &mut Runtime, value: MiraValue, key: &str) -> Result<MiraValue> {
    get_value(runtime, value, MiraValue::NIL, Some(key))
}

pub(crate) fn get_value(
    runtime: &mut Runtime,
    value: MiraValue,
    key: MiraValue,
    known_key: Option<&str>,
) -> Result<MiraValue> {
    if value.is_array() {
        let index = match to_number(runtime, key) {
            Ok(index) if index.is_finite() => index.trunc() as isize,
            _ => return Ok(MiraValue::NIL),
        };
        let length = array_len(runtime, value)?.unwrap_or(0);
        let index = if index < 0 {
            length.checked_add_signed(index)
        } else {
            Some(index as usize)
        };
        return match index {
            Some(index) if index < length => Ok(into_element(
                array_get(runtime, value, index)?.unwrap_or(MiraValue::NIL),
            )),
            _ => Ok(MiraValue::NIL),
        };
    }

    let key = match known_key {
        Some(key) => key.to_owned(),
        None => to_string(runtime, key)?,
    };
    match value.kind() {
        MiraValueKind::Record(_) => Ok(into_element(
            record_get(runtime, value, &key)?.unwrap_or(MiraValue::NIL),
        )),
        MiraValueKind::Module(_) => Ok(module_get(runtime, value, &key)?.unwrap_or(MiraValue::NIL)),
        _ => Ok(MiraValue::NIL),
    }
}

pub(crate) fn get_i(runtime: &mut Runtime, value: MiraValue, key: i64) -> Result<MiraValue> {
    let (MiraValueKind::Record(_), Ok(key)) = (value.kind(), u32::try_from(key)) else {
        return get_value(runtime, value, MiraValue::number(key as f64), None);
    };
    Ok(into_element(
        record_get_i(runtime, value, key)?.unwrap_or(MiraValue::NIL),
    ))
}

pub(crate) fn set(
    _runtime: &mut Runtime,
    obj: MiraValue,
    _key: MiraValue,
    _value: MiraValue,
) -> Result<()> {
    Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
        expected: "mutable extern",
        actual: obj.value_type(),
    }))
}

pub(crate) fn pick(runtime: &mut Runtime, value: MiraValue, keys: &[String]) -> Result<MiraValue> {
    let mut result = IndexMap::new();
    if value.is_record() {
        for key in keys {
            if has(runtime, value, MiraValue::NIL, Some(key))? {
                result.insert(key.clone(), get(runtime, value, key)?);
            }
        }
    }
    runtime.insert(result)
}

pub(crate) fn omit(runtime: &mut Runtime, value: MiraValue, keys: &[String]) -> Result<MiraValue> {
    let mut result = IndexMap::new();
    if value.is_record() {
        for entry in iterate_record(runtime, value)? {
            let key = entry.key(runtime)?;
            if keys.iter().any(|candidate| candidate == key) {
                continue;
            }
            let key = key.to_owned();
            let value = into_element(entry.get(runtime)?);
            result.insert(key, value);
        }
    }
    runtime.insert(result)
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::{MiraHandle, MiraManageable, MiraRecord};

    use super::*;

    struct LookupCounters {
        string: Rc<Cell<usize>>,
        integer: Rc<Cell<usize>>,
    }

    impl MiraRecord for LookupCounters {
        fn len(&self) -> usize {
            1
        }

        fn index_of(&self, key: &str) -> Option<usize> {
            self.string.set(self.string.get() + 1);
            (key == "0").then_some(0)
        }

        fn index_of_i(&self, key: u32) -> Option<usize> {
            self.integer.set(self.integer.get() + 1);
            (key == 0).then_some(0)
        }

        fn key(&self, index: usize) -> Result<&str> {
            (index == 0)
                .then_some("0")
                .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
        }

        fn get(
            &self,
            _self_handle: MiraHandle<dyn MiraRecord>,
            _runtime: &Runtime,
            index: usize,
        ) -> Result<MiraManageable> {
            (index == 0)
                .then(|| MiraManageable::from(42))
                .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
        }
    }

    #[test]
    fn constant_record_indexes_use_integer_lookup() {
        let string = Rc::new(Cell::new(0));
        let integer = Rc::new(Cell::new(0));
        let mut runtime = Runtime::new();
        runtime
            .insert_global(
                "record",
                MiraManageable::from_record(LookupCounters {
                    string: string.clone(),
                    integer: integer.clone(),
                }),
            )
            .unwrap();

        assert_eq!(runtime.eval_unchecked("record.0").as_number(), Some(42.0));
        assert!(runtime.eval_unchecked("record[-1]").is_nil());

        assert_eq!(integer.get(), 1);
        assert_eq!(string.get(), 1);
    }
}

use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "unique", |call, args| {
        let values = array_value(call, *required(args, 0, "data")?)?;
        validate_optional_callable(args.get(1))?;
        let mut result = Vec::new();
        for value in values {
            let mut found = false;
            for existing in &result {
                if equal(call, &value, existing, args.get(1))? {
                    found = true;
                    break;
                }
            }
            if !found {
                result.push(value);
            }
        }
        call.insert(result)
    });
    insert_native(context, "unique_by", |call, args| {
        let values = array_value(call, *required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: key_function.value_type(),
            }));
        }
        validate_optional_callable(args.get(2))?;
        let original = call.insert(values.clone())?;
        let mut result = Vec::new();
        let mut keys = Vec::new();
        for (index, value) in values.into_iter().enumerate() {
            let key = call.call(
                *key_function,
                &[value, MiraValue::number(index as f64), original],
            )?;
            let mut found = false;
            for existing in &keys {
                if equal(call, &key, existing, args.get(2))? {
                    found = true;
                    break;
                }
            }
            if !found {
                keys.push(key);
                result.push(value);
            }
        }
        call.insert(result)
    });
}

fn validate_optional_callable(value: Option<&MiraValue>) -> Result<()> {
    if let Some(value) = value.filter(|value| **value != MiraValue::NIL)
        && !is_callable(value)?
    {
        return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
            actual: value.value_type(),
        }));
    }
    Ok(())
}

fn equal(
    call: &mut Runtime,
    left: &MiraValue,
    right: &MiraValue,
    equaler: Option<&MiraValue>,
) -> Result<bool> {
    if let Some(equaler) = equaler.filter(|value| **value != MiraValue::NIL) {
        if !is_callable(equaler)? {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: equaler.value_type(),
            }));
        }
        operations::to_boolean(call.call(*equaler, &[*left, *right])?)
    } else {
        operations::same_value(call, *left, *right)
    }
}

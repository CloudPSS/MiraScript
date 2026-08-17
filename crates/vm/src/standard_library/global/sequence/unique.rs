use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "unique", |call, args| {
        let values = array_value(required(args, 0, "data")?)?;
        validate_optional_callable(args.get(1), "equal")?;
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
        Ok(MiraAny::Array(result.into()))
    });
    insert_native(context, "unique_by", |call, args| {
        let values = array_value(required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime("Argument `key` is not callable"));
        }
        validate_optional_callable(args.get(2), "equal")?;
        let original = MiraAny::Array(values.clone().into());
        let mut result = Vec::new();
        let mut keys = Vec::new();
        for (index, value) in values.into_iter().enumerate() {
            let key = call.call(
                key_function,
                &[
                    value.clone(),
                    MiraAny::Number(index as f64),
                    original.clone(),
                ],
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
        Ok(MiraAny::Array(result.into()))
    });
}

fn validate_optional_callable(value: Option<&MiraAny>, name: &str) -> Result<()> {
    if let Some(value) = value.filter(|value| **value != MiraAny::Nil)
        && !is_callable(value)?
    {
        return Err(MiraError::runtime(format!(
            "Argument `{name}` is not callable"
        )));
    }
    Ok(())
}

fn equal(
    call: &mut MiraCallContext<'_>,
    left: &MiraAny,
    right: &MiraAny,
    equaler: Option<&MiraAny>,
) -> Result<bool> {
    if let Some(equaler) = equaler.filter(|value| **value != MiraAny::Nil) {
        if !is_callable(equaler)? {
            return Err(MiraError::runtime("Argument `equal` is not callable"));
        }
        operations::to_boolean(&call.call(equaler, &[left.clone(), right.clone()])?)
    } else {
        Ok(left == right)
    }
}

use super::*;

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(runtime, fn unique(call, args) {
        let values = array_value(call, *required(args, 0, "data")?)?;
        let equaler = optional_callable(args, 1, "equaler")?;
        let mut result = Vec::new();
        for value in values {
            let mut found = false;
            for existing in &result {
                if equal(call, &value, existing, equaler)? {
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
    global_builtin!(runtime, fn unique_by(call, args) {
        let values = array_value(call, *required(args, 0, "data")?)?;
        let key_function = callable(args, 1, "key")?;
        let equaler = optional_callable(args, 2, "equaler")?;
        let original = call.insert(values.clone())?;
        let mut result = Vec::new();
        let mut keys = Vec::new();
        for (index, value) in values.into_iter().enumerate() {
            let key = key_function.call(
                call,
                &[value, MiraValue::number(index as f64), original],
            )?;
            let mut found = false;
            for existing in &keys {
                if equal(call, &key, existing, equaler)? {
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

fn equal(
    call: &mut Runtime,
    left: &MiraValue,
    right: &MiraValue,
    equaler: Option<MiraFunctionHandle>,
) -> Result<bool> {
    if let Some(equaler) = equaler {
        operations::to_boolean(equaler.call(call, &[*left, *right])?)
    } else {
        operations::same_value(call, *left, *right)
    }
}

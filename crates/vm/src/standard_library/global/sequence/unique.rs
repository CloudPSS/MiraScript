use super::*;

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(runtime, fn unique(call, args) {
        let value = *required(args, 0, "data")?;
        let equaler = optional_callable(args, 1, "equaler")?;
        let iter = operations::iterate_array(call, value)?;
        let mut result = Vec::with_capacity(iter.len());
        for entry in iter {
            let value = entry.get(call)?;
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
        let data = *required(args, 0, "data")?;
        let key_function = callable(args, 1, "key")?;
        let equaler = optional_callable(args, 2, "equaler")?;
        let iter = operations::iterate_array(call, data)?;
        let mut result = Vec::with_capacity(iter.len());
        let mut keys = Vec::with_capacity(iter.len());
        for entry in iter {
            let index = entry.index();
            let value = entry.get(call)?;
            let key = key_function.call(
                call,
                &[value, MiraValue::number(index as f64), data],
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

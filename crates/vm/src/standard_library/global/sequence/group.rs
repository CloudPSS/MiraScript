use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "group_by", |call, args| {
        let data = array_value(call, *required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: key_function.value_type(),
            }));
        }
        let original = call.insert(data.clone())?;
        let mut groups: IndexMap<String, Vec<MiraValue>> = IndexMap::new();
        for (index, value) in data.into_iter().enumerate() {
            call.checkpoint()?;
            let key = call.call(
                *key_function,
                &[value, MiraValue::Number(index as f64), original],
            )?;
            groups
                .entry(operations::to_string(call, key)?)
                .or_default()
                .push(value);
        }
        let mut result = IndexMap::with_capacity(groups.len());
        for (key, values) in groups {
            result.insert(key, call.insert(values)?);
        }
        call.insert(result)
    });
}

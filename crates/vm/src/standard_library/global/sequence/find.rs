use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "find", |call, args| {
        let data = Data::from_value(call, *required(args, 0, "data")?)?;
        let predicate = required(args, 1, "predicate")?;
        let callable = is_callable(predicate)?;
        let original = data.original(call)?;
        for (key, value) in data_items(call, &data)? {
            call.checkpoint()?;
            let found = if callable {
                operations::to_boolean(call.call(*predicate, &[value, key, original])?)?
            } else {
                operations::same_value(call, value, *predicate)?
            };
            if found {
                return pair(call, key, value);
            }
        }
        Ok(MiraValue::nil())
    });
}

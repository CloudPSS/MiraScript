use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn find(call, args) {
        let data = Data::from_value(call, *required(args, 0, "data")?)?;
        let predicate = required(args, 1, "predicate")?;
        let callable = predicate.as_function();
        let original = data.original(call)?;
        for (key, value) in data_items(call, &data)? {
            call.checkpoint()?;
            let found = if let Some(callable) = callable {
                operations::to_boolean(callable.call(call, &[value, key, original])?)?
            } else {
                operations::same_value(call, value, *predicate)?
            };
            if found {
                return pair(call, key, value);
            }
        }
        Ok(MiraValue::NIL)
    });
}

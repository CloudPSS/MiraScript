use super::*;

pub(super) fn install(context: &mut Runtime) {
    for (name, every) in [("all", true), ("any", false)] {
        insert_native(context, name, move |call, args| {
            let data = Data::from_value(call, *required(args, 0, "data")?)?;
            let predicate = required(args, 1, "predicate")?;
            if !is_callable(predicate)? {
                return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                    actual: predicate.value_type(),
                }));
            }
            let original = data.original(call)?;
            for (key, value) in data_items(call, &data)? {
                call.checkpoint()?;
                let matched =
                    operations::to_boolean(call.call(*predicate, &[value, key, original])?)?;
                if every && !matched {
                    return Ok(MiraValue::Boolean(false));
                }
                if !every && matched {
                    return Ok(MiraValue::Boolean(true));
                }
            }
            Ok(MiraValue::Boolean(every))
        });
    }
}

use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "find", |call, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        let predicate = required(args, 1, "predicate")?;
        let callable = is_callable(predicate)?;
        let original = data.original();
        for (key, value) in data_items(&data) {
            call.checkpoint()?;
            let found = if callable {
                operations::to_boolean(
                    &call.call(predicate, &[value.clone(), key.clone(), original.clone()])?,
                )?
            } else {
                &value == predicate
            };
            if found {
                return Ok(pair(key, value));
            }
        }
        Ok(MiraAny::Nil)
    });
}

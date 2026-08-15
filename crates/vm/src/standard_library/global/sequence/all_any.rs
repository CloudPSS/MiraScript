use super::*;

pub(super) fn install(context: &mut MiraContext) {
    for (name, every) in [("all", true), ("any", false)] {
        insert_native(context, name, move |call, args| {
            let data = Data::from_value(required(args, 0, "data")?)?;
            let predicate = required(args, 1, "predicate")?;
            if !is_callable(predicate)? {
                return Err(MiraError::runtime("Argument `predicate` is not callable"));
            }
            let original = data.original();
            for (key, value) in data_items(&data) {
                call.checkpoint()?;
                let matched = operations::to_boolean(
                    &call.call(predicate, &[value, key, original.clone()])?,
                )?;
                if every && !matched {
                    return Ok(MiraAny::Boolean(false));
                }
                if !every && matched {
                    return Ok(MiraAny::Boolean(true));
                }
            }
            Ok(MiraAny::Boolean(every))
        });
    }
}

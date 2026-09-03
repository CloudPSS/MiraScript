use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn all: all_any::<true>);
    global_builtin!(context, fn any: all_any::<false>);
}

fn all_any<const EVERY: bool>(call: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
    let data = Data::from_value(call, *required(args, 0, "data")?)?;
    let predicate = callable(args, 1, "predicate")?;
    let original = data.original(call)?;
    let result = iterate_data(
        call,
        &data,
        |_, _| Ok(EVERY),
        |call, key, value, result| {
            call.checkpoint()?;
            let matched = operations::to_boolean(predicate.call(call, &[value, key, original])?)?;
            *result = if EVERY { matched } else { *result || matched };
            Ok(if EVERY { *result } else { !*result })
        },
    )?;
    Ok(MiraValue::boolean(result))
}

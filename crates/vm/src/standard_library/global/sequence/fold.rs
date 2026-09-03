use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn fold(call, args) {
        let data = Data::from_value(call, *required(args, 0, "data")?)?;
        let mut accumulator = *required(args, 1, "initial")?;
        let function = callable(args, 2, "f")?;
        let original = data.original(call)?;
        for (key, value) in data_items(call, &data)? {
            call.checkpoint()?;
            accumulator = function.call(call, &[accumulator, value, key, original])?;
        }
        Ok(accumulator)
    });
}

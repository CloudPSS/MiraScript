use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn fold(call, args) {
        let data = Data::from_value(call, *required(args, 0, "data")?)?;
        let mut accumulator = *required(args, 1, "initial")?;
        let function = required(args, 2, "f")?;
        if !is_callable(function)? {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: function.value_type(),
            }));
        }
        let original = data.original(call)?;
        for (key, value) in data_items(call, &data)? {
            call.checkpoint()?;
            accumulator = call.call(*function, &[accumulator, value, key, original])?;
        }
        Ok(accumulator)
    });
}

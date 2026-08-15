use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "fold", |call, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        let mut accumulator = required(args, 1, "initial")?.clone();
        let function = required(args, 2, "f")?;
        if !is_callable(function)? {
            return Err(MiraError::runtime("Argument `f` is not callable"));
        }
        let original = data.original();
        for (key, value) in data_items(&data) {
            call.checkpoint()?;
            accumulator = call.call(function, &[accumulator, value, key, original.clone()])?;
        }
        Ok(accumulator)
    });
}

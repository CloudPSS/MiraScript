use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "new_array", |call, args| {
        let max = call.options().max_array_len;
        let length = array_length(call, *required(args, 0, "length")?, max)?;
        let generator = required(args, 1, "generator")?;
        if !is_callable(generator)? {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: generator.value_type(),
            }));
        }
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            call.checkpoint()?;
            result.push(const_value(
                call.call(*generator, &[MiraValue::number(index as f64)])?,
            )?);
        }
        call.insert(result)
    });
    insert_native(context, "new_record", |call, args| {
        let max = call.options().max_array_len;
        let length = array_length(call, *required(args, 0, "size")?, max)?;
        let generator = required(args, 1, "generator")?;
        if !is_callable(generator)? {
            return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                actual: generator.value_type(),
            }));
        }
        let mut result = IndexMap::new();
        for index in 0..length {
            call.checkpoint()?;
            let entry = call.call(*generator, &[MiraValue::number(index as f64)])?;
            if entry == MiraValue::NIL {
                continue;
            }
            let key_value = operations::get_value(call, entry, MiraValue::number(0.0), None)?;
            let key = operations::to_string(call, key_value)?;
            let value = operations::get_value(call, entry, MiraValue::number(1.0), None)?;
            result.insert(key, const_value(value)?);
        }
        call.insert(result)
    });
}

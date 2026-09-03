use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn new_array(call, args) {
        let max = call.options().max_array_len;
        let length = array_length(call, *required(args, 0, "length")?, max)?;
        let generator = callable(args, 1, "generator")?;
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            call.checkpoint()?;
            result.push(const_value(generator.call(call, &[MiraValue::number(index as f64)])?)?);
        }
        call.insert(result)
    });
    global_builtin!(context, fn new_record(call, args) {
        let max = call.options().max_array_len;
        let length = array_length(call, *required(args, 0, "size")?, max)?;
        let generator = callable(args, 1, "generator")?;
        let mut result = IndexMap::new();
        for index in 0..length {
            call.checkpoint()?;
            let entry = generator.call(call, &[MiraValue::number(index as f64)])?;
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

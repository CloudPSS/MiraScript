use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "new_array", |call, args| {
        let length = array_length(required(args, 0, "length")?, call.options().max_array_len)?;
        let generator = required(args, 1, "generator")?;
        if !is_callable(generator)? {
            return Err(MiraError::runtime("Argument `generator` is not callable"));
        }
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            call.checkpoint()?;
            result.push(const_value(
                call.call(generator, &[MiraAny::Number(index as f64)])?,
            )?);
        }
        Ok(MiraAny::Array(result.into()))
    });
    insert_native(context, "new_record", |call, args| {
        let length = array_length(required(args, 0, "size")?, call.options().max_array_len)?;
        let generator = required(args, 1, "generator")?;
        if !is_callable(generator)? {
            return Err(MiraError::runtime("Argument `generator` is not callable"));
        }
        let mut result = IndexMap::new();
        for index in 0..length {
            call.checkpoint()?;
            let entry = call.call(generator, &[MiraAny::Number(index as f64)])?;
            if entry == MiraAny::Nil {
                continue;
            }
            let key =
                operations::to_string(&operations::get_value(&entry, &MiraAny::Number(0.0))?)?;
            let value = operations::get_value(&entry, &MiraAny::Number(1.0))?.into_element()?;
            result.insert(key, value);
        }
        Ok(MiraAny::Record(result.into()))
    });
}

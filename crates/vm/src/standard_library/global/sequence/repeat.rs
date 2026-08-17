use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "repeat", |call, args| {
        let value = const_value(required(args, 0, "data")?.clone())?;
        let length = array_length(required(args, 1, "times")?, call.options().max_array_len)?;
        Ok(MiraAny::Array(vec![value; length].into()))
    });
}

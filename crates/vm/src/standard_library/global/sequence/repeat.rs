use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "repeat", |call, args| {
        let value = const_value(*required(args, 0, "data")?)?;
        let length = array_length(
            call,
            *required(args, 1, "times")?,
            call.options().max_array_len,
        )?;
        call.insert(vec![value; length])
    });
}

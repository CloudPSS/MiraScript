use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "reverse", |call, args| {
        let mut values = array_value(call, *required(args, 0, "arr")?)?;
        values.reverse();
        call.insert(values)
    });
}

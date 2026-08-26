use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn reverse(call, args) {
        let mut values = array_value(call, *required(args, 0, "arr")?)?;
        values.reverse();
        call.insert(values)
    });
}

use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn len(call, args) {
        let value = *required(args, 0, "arr")?;
        let Some(length) = operations::array_len(call, value)? else {
            return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                expected: "array",
                actual: value.value_type(),
            }));
        };
        Ok(MiraValue::number(length as f64))
    });
}

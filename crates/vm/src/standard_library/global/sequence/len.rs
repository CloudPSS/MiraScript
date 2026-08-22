use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "len", |call, args| {
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

use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn keys(call, args) {
        let value = *required(args, 0, "data")?;
        let keys: Vec<MiraValue> = match value.kind() {
            MiraValueKind::Array(_) => (0..operations::array_len(call, value)?.unwrap_or(0))
                .map(|index| MiraValue::number(index as f64))
                .collect(),
            MiraValueKind::Record(_) => operations::record_keys(call, value)?
                .unwrap_or_default()
                .into_iter()
                .map(|key| call.insert(key))
                .collect::<Result<Vec<_>>>()?,
            MiraValueKind::Module(_) => operations::module_keys(call, value)?
                .unwrap_or_default()
                .into_iter()
                .map(|key| call.insert(key))
                .collect::<Result<Vec<_>>>()?,
            _ => {
                return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                    expected: "compound value",
                    actual: value.value_type(),
                }));
            }
        };
        call.insert(keys)
    });
    global_builtin!(context, fn values(call, args) {
        let data = Data::from_value(call, *required(args, 0, "data")?)?;
        match data {
            Data::Array(values) => call.insert(values),
            Data::Record(values) => call.insert(values.into_values().collect::<Vec<_>>()),
            Data::Primitive(value) => Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                expected: "array or record",
                actual: value.value_type(),
            })),
        }
    });
    global_builtin!(context, fn entries(call, args) {
        let data = Data::from_value(call, *required(args, 0, "data")?)?;
        let mut entries = Vec::new();
        match data {
            Data::Array(values) => {
                for (index, value) in values.into_iter().enumerate() {
                    entries.push(pair(call, MiraValue::number(index as f64), value)?);
                }
            }
            Data::Record(values) => {
                for (key, value) in values {
                    let key = call.insert(key)?;
                    entries.push(pair(call, key, value)?);
                }
            }
            Data::Primitive(value) => {
                return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                    expected: "array or record",
                    actual: value.value_type(),
                }));
            }
        }
        call.insert(entries)
    });
}

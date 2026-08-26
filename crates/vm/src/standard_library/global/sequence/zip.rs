use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "zip", |call, args| {
        zip(call, *required(args, 0, "data")?)
    });
}

fn zip(call: &mut Runtime, value: MiraValue) -> Result<MiraValue> {
    let data = Data::from_value(call, value)?;
    let items = data_items(call, &data)?;
    let mut arrays = Vec::new();
    let mut length = 0;
    for (key, value) in items {
        let array = array_value(call, value)?;
        length = length.max(array.len());
        arrays.push((key, array));
    }
    let mut result = Vec::with_capacity(length);
    for index in 0..length {
        call.checkpoint()?;
        match &data {
            Data::Array(_) => result.push(
                call.insert(
                    arrays
                        .iter()
                        .map(|(_, array)| array.get(index).cloned().unwrap_or(MiraValue::NIL))
                        .collect::<Vec<_>>(),
                )?,
            ),
            Data::Record(_) => {
                let mut record = IndexMap::new();
                for (key, array) in &arrays {
                    let key = operations::to_string(call, *key)?;
                    record.insert(key, array.get(index).cloned().unwrap_or(MiraValue::NIL));
                }
                result.push(call.insert(record)?);
            }
            Data::Primitive(_) => {
                return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
                    expected: "array or record",
                    actual: value.value_type(),
                }));
            }
        }
    }
    call.insert(result)
}

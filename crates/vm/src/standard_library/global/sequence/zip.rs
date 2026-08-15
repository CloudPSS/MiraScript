use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "zip", |call, args| {
        zip(call, required(args, 0, "data")?)
    });
}

fn zip(call: &mut MiraCallContext<'_>, value: &MiraAny) -> Result<MiraAny> {
    let data = Data::from_value(value)?;
    let items = data_items(&data);
    let mut arrays = Vec::new();
    let mut length = 0;
    for (key, value) in items {
        let array = array_value(&value)?;
        length = length.max(array.len());
        arrays.push((key, array));
    }
    let mut result = Vec::with_capacity(length);
    for index in 0..length {
        call.checkpoint()?;
        match &data {
            Data::Array(_) => result.push(MiraAny::Array(
                arrays
                    .iter()
                    .map(|(_, array)| array.get(index).cloned().unwrap_or(MiraAny::Nil))
                    .collect(),
            )),
            Data::Record(_) => result.push(MiraAny::Record(
                arrays
                    .iter()
                    .map(|(key, array)| {
                        let MiraAny::String(key) = key else {
                            unreachable!()
                        };
                        (
                            key.clone(),
                            array.get(index).cloned().unwrap_or(MiraAny::Nil),
                        )
                    })
                    .collect(),
            )),
            Data::Primitive(_) => {
                return Err(MiraError::runtime("Argument `data` is not array | record"));
            }
        }
    }
    Ok(MiraAny::Array(result))
}

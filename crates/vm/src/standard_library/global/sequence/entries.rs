use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "keys", |_, args| {
        let value = required(args, 0, "data")?;
        let keys: Vec<MiraAny> = match value {
            MiraAny::Array(_) | MiraAny::RustArray(_) => (0..value.array_len()?.unwrap_or(0))
                .map(|index| MiraAny::Number(index as f64))
                .collect(),
            MiraAny::Record(_) | MiraAny::RustRecord(_) => value
                .record_keys()?
                .unwrap_or_default()
                .into_iter()
                .map(MiraAny::from)
                .collect(),
            MiraAny::Module(module) => module.keys().into_iter().map(MiraAny::from).collect(),
            _ => {
                return Err(MiraError::runtime(
                    "Argument `data` is not a compound value",
                ));
            }
        };
        Ok(MiraAny::Array(keys.into()))
    });
    insert_native(context, "values", |_, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        match data {
            Data::Array(values) => Ok(MiraAny::Array(values.into())),
            Data::Record(values) => Ok(MiraAny::Array(values.into_values().collect())),
            Data::Primitive(_) => Err(MiraError::runtime("Argument `data` is not array | record")),
        }
    });
    insert_native(context, "entries", |_, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        let entries: Vec<MiraAny> = match data {
            Data::Array(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| pair(MiraAny::Number(index as f64), value))
                .collect(),
            Data::Record(values) => values
                .into_iter()
                .map(|(key, value)| pair(MiraAny::String(key.into()), value))
                .collect(),
            Data::Primitive(_) => {
                return Err(MiraError::runtime("Argument `data` is not array | record"));
            }
        };
        Ok(MiraAny::Array(entries.into()))
    });
}

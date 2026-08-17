use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "group_by", |call, args| {
        let data = array_value(required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime("Argument `key` is not callable"));
        }
        let original = MiraAny::Array(data.clone().into());
        let mut groups: IndexMap<String, Vec<MiraAny>> = IndexMap::new();
        for (index, value) in data.into_iter().enumerate() {
            call.checkpoint()?;
            let key = call.call(
                key_function,
                &[
                    value.clone(),
                    MiraAny::Number(index as f64),
                    original.clone(),
                ],
            )?;
            groups
                .entry(operations::to_string(&key)?)
                .or_default()
                .push(value);
        }
        Ok(MiraAny::Record(
            groups
                .into_iter()
                .map(|(key, values)| (key, MiraAny::Array(values.into())))
                .collect(),
        ))
    });
}

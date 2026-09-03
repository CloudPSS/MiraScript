use super::*;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn group_by(call, args) {
        let data = *required(args, 0, "data")?;
        let key_function = callable(args, 1, "key")?;
        let mut groups: IndexMap<String, Vec<MiraValue>> = IndexMap::new();
        for entry in operations::iterate_array(call, data)? {
            let index = entry.index();
            let value = entry.get(call)?;
            call.checkpoint()?;
            let key = key_function.call(
                call,
                &[value, MiraValue::number(index as f64), data],
            )?;
            groups
                .entry(operations::to_string(call, key)?)
                .or_default()
                .push(value);
        }
        let mut result = IndexMap::with_capacity(groups.len());
        for (key, values) in groups {
            result.insert(key, call.insert(values)?);
        }
        call.insert(result)
    });
}

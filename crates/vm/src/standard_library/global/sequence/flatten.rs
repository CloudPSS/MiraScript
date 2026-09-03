use super::*;

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(runtime, fn flatten(call, args) {
        let value = *required(args, 0, "data")?;
        let depth = match args.get(1) {
            None => 1,
            Some(value) if value.is_nil() => 1,
            Some(value) => operations::to_number(call, *value)?.trunc().max(0.0) as usize,
        };
        let values = flatten_impl(call, value, depth)?;
        call.insert(values)
    });
}

fn flatten_impl(runtime: &mut Runtime, value: MiraValue, depth: usize) -> Result<Vec<MiraValue>> {
    if depth == 0 {
        return operations::iterable_array(runtime, value);
    }
    let iter = operations::iterate_array(runtime, value)?;
    let mut result = Vec::with_capacity(iter.len());
    for entry in iter {
        let value = entry.get(runtime)?;
        if operations::array_len(runtime, value)?.is_some() {
            result.extend(flatten_impl(runtime, value, depth - 1)?);
        } else {
            result.push(value);
        }
    }
    Ok(result)
}

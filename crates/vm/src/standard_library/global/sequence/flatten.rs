use super::*;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "flatten", |call, args| {
        let values = array_value(call, *required(args, 0, "data")?)?;
        let depth = match args.get(1) {
            None => 1,
            Some(value) if value.is_nil() => 1,
            Some(value) => operations::to_number(call, *value)?.trunc().max(0.0) as usize,
        };
        let values = flatten(call, values, depth)?;
        call.insert(values)
    });
}

fn flatten(runtime: &mut Runtime, values: Vec<MiraValue>, depth: usize) -> Result<Vec<MiraValue>> {
    if depth == 0 {
        return Ok(values);
    }
    let mut result = Vec::new();
    for value in values {
        if operations::array_len(runtime, value)?.is_some() {
            let values = operations::iterable_array(runtime, value)?;
            result.extend(flatten(runtime, values, depth - 1)?);
        } else {
            result.push(value);
        }
    }
    Ok(result)
}

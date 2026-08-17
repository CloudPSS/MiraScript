use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "flatten", |_, args| {
        let values = array_value(required(args, 0, "data")?)?;
        let depth = match args.get(1) {
            None | Some(MiraAny::Nil) => 1,
            Some(value) => operations::to_number(value)?.trunc().max(0.0) as usize,
        };
        Ok(MiraAny::Array(flatten(values, depth)?.into()))
    });
}

fn flatten(values: Vec<MiraAny>, depth: usize) -> Result<Vec<MiraAny>> {
    if depth == 0 {
        return Ok(values);
    }
    let mut result = Vec::new();
    for value in values {
        if value.array_len()?.is_some() {
            result.extend(flatten(operations::materialize_array(&value)?, depth - 1)?);
        } else {
            result.push(value);
        }
    }
    Ok(result)
}

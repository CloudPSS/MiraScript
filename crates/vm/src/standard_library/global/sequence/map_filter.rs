use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "map", |call, args| {
        map_like(call, args, MapMode::Map)
    });
    insert_native(context, "filter", |call, args| {
        map_like(call, args, MapMode::Filter)
    });
    insert_native(context, "filter_map", |call, args| {
        map_like(call, args, MapMode::FilterMap)
    });
}

#[derive(Clone, Copy)]
enum MapMode {
    Map,
    Filter,
    FilterMap,
}

fn map_like(call: &mut MiraCallContext<'_>, args: &[MiraAny], mode: MapMode) -> Result<MiraAny> {
    let data = Data::from_value(required(args, 0, "data")?)?;
    let function = required(args, 1, "f")?;
    if !is_callable(function)? {
        return Err(MiraError::runtime("Argument `f` is not callable"));
    }
    let original = data.original();
    match data {
        Data::Primitive(value) => {
            let mapped = call.call(function, &[value.clone(), MiraAny::Nil, value.clone()])?;
            match mode {
                MapMode::Map => const_value(mapped),
                MapMode::Filter => Ok(if operations::to_boolean(&mapped)? {
                    value
                } else {
                    MiraAny::Nil
                }),
                MapMode::FilterMap => Ok(if mapped == MiraAny::Nil {
                    MiraAny::Nil
                } else {
                    const_value(mapped)?
                }),
            }
        }
        Data::Array(values) => {
            let mut result = Vec::new();
            for (index, value) in values.into_iter().enumerate() {
                call.checkpoint()?;
                let mapped = call.call(
                    function,
                    &[
                        value.clone(),
                        MiraAny::Number(index as f64),
                        original.clone(),
                    ],
                )?;
                match mode {
                    MapMode::Map => result.push(const_value(mapped)?),
                    MapMode::Filter if operations::to_boolean(&mapped)? => result.push(value),
                    MapMode::FilterMap if mapped != MiraAny::Nil => {
                        result.push(const_value(mapped)?)
                    }
                    _ => {}
                }
            }
            Ok(MiraAny::Array(result))
        }
        Data::Record(values) => {
            let mut result = IndexMap::new();
            for (key, value) in values {
                call.checkpoint()?;
                let mapped = call.call(
                    function,
                    &[
                        value.clone(),
                        MiraAny::String(key.clone()),
                        original.clone(),
                    ],
                )?;
                match mode {
                    MapMode::Map => {
                        result.insert(key, const_value(mapped)?);
                    }
                    MapMode::Filter if operations::to_boolean(&mapped)? => {
                        result.insert(key, value);
                    }
                    MapMode::FilterMap if mapped != MiraAny::Nil => {
                        result.insert(key, const_value(mapped)?);
                    }
                    _ => {}
                }
            }
            Ok(MiraAny::Record(result))
        }
    }
}

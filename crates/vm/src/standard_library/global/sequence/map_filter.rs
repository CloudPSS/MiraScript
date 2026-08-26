use super::*;

pub(super) fn install(context: &mut Runtime) {
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

fn map_like(call: &mut Runtime, args: &[MiraValue], mode: MapMode) -> Result<MiraValue> {
    let data = Data::from_value(call, *required(args, 0, "data")?)?;
    let function = required(args, 1, "f")?;
    if !is_callable(function)? {
        return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
            actual: function.value_type(),
        }));
    }
    let original = data.original(call)?;
    match data {
        Data::Primitive(value) => {
            let mapped = call.call(*function, &[value, MiraValue::NIL, value])?;
            match mode {
                MapMode::Map => const_value(mapped),
                MapMode::Filter => Ok(if operations::to_boolean(mapped)? {
                    value
                } else {
                    MiraValue::NIL
                }),
                MapMode::FilterMap => Ok(if mapped == MiraValue::NIL {
                    MiraValue::NIL
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
                    *function,
                    &[value, MiraValue::number(index as f64), original],
                )?;
                match mode {
                    MapMode::Map => result.push(const_value(mapped)?),
                    MapMode::Filter if operations::to_boolean(mapped)? => result.push(value),
                    MapMode::FilterMap if mapped != MiraValue::NIL => {
                        result.push(const_value(mapped)?)
                    }
                    _ => {}
                }
            }
            call.insert(result)
        }
        Data::Record(values) => {
            let mut result = IndexMap::new();
            for (key, value) in values {
                call.checkpoint()?;
                let key_value = call.insert(key.clone())?;
                let mapped = call.call(*function, &[value, key_value, original])?;
                match mode {
                    MapMode::Map => {
                        result.insert(key, const_value(mapped)?);
                    }
                    MapMode::Filter if operations::to_boolean(mapped)? => {
                        result.insert(key, value);
                    }
                    MapMode::FilterMap if mapped != MiraValue::NIL => {
                        result.insert(key, const_value(mapped)?);
                    }
                    _ => {}
                }
            }
            call.insert(result)
        }
    }
}

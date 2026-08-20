use indexmap::IndexMap;

use crate::standard_library::{insert_native, string};
use crate::{MiraAny, MiraContext, MiraError, Result, Runtime, operations};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "to_json", |call, args| {
        let Some(value) = args.first() else {
            return Err(MiraError::runtime("Argument `data` is required"));
        };
        if matches!(value, MiraAny::Function(_)) {
            return Ok(MiraAny::Nil);
        }
        let json = to_json_value(call, value, false)?;
        Ok(match json {
            Some(value) => MiraAny::String(
                serde_json::to_string(&value)
                    .map_err(|error| {
                        MiraError::runtime(format!("Failed to serialize JSON: {error}"))
                    })?
                    .into(),
            ),
            None => MiraAny::Nil,
        })
    });
    insert_native(context, "from_json", |_, args| {
        let source = string(args, 0, "json")?;
        match serde_json::from_str::<serde_json::Value>(&source) {
            Ok(value) => Ok(from_json_value(value)),
            Err(_) if args.len() > 1 => Ok(args[1].clone()),
            Err(error) => Err(MiraError::runtime(format!("Invalid JSON: {error}"))),
        }
    });
}

fn to_json_value(
    call: &mut Runtime,
    value: &MiraAny,
    in_container: bool,
) -> Result<Option<serde_json::Value>> {
    Ok(match value {
        MiraAny::Uninitialized | MiraAny::Function(_) => {
            if in_container {
                None
            } else {
                return Ok(None);
            }
        }
        MiraAny::Nil => Some(serde_json::Value::Null),
        MiraAny::Boolean(value) => Some((*value).into()),
        MiraAny::Number(value) => Some(if !value.is_finite() {
            serde_json::Value::Null
        } else if *value == 0.0 {
            serde_json::Value::Number(0.into())
        } else if value.fract() == 0.0 && *value >= i64::MIN as f64 && *value <= i64::MAX as f64 {
            serde_json::Value::Number((*value as i64).into())
        } else {
            serde_json::Value::Number(serde_json::Number::from_f64(*value).unwrap())
        }),
        MiraAny::String(value) => Some(value.to_string().into()),
        MiraAny::Array(_) | MiraAny::RustArray(_) => Some(serde_json::Value::Array(
            operations::iterable_array(value)?
                .iter()
                .map(|item| Ok(to_json_value(call, item, true)?.unwrap_or(serde_json::Value::Null)))
                .collect::<Result<Vec<_>>>()?,
        )),
        MiraAny::Record(_) | MiraAny::RustRecord(_) => {
            let mut map = serde_json::Map::new();
            for key in value.record_keys()?.unwrap_or_default() {
                if let Some(item) =
                    to_json_value(call, &value.record_get(&key)?.unwrap_or(MiraAny::Nil), true)?
                {
                    map.insert(key, item);
                }
            }
            Some(serde_json::Value::Object(map))
        }
        MiraAny::Module(module) => {
            let mut map = serde_json::Map::new();
            for key in module.keys() {
                let item = call.get(value, key.clone())?;
                if let Some(item) = to_json_value(call, &item, true)? {
                    map.insert(key, item);
                }
            }
            Some(serde_json::Value::Object(map))
        }
    })
}

fn from_json_value(value: serde_json::Value) -> MiraAny {
    match value {
        serde_json::Value::Null => MiraAny::Nil,
        serde_json::Value::Bool(value) => MiraAny::Boolean(value),
        serde_json::Value::Number(value) => MiraAny::Number(value.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(value) => MiraAny::String(value.into()),
        serde_json::Value::Array(values) => {
            MiraAny::Array(values.into_iter().map(from_json_value).collect())
        }
        serde_json::Value::Object(values) => MiraAny::Record(
            values
                .into_iter()
                .map(|(key, value)| (key, from_json_value(value)))
                .collect::<IndexMap<_, _>>()
                .into(),
        ),
    }
}

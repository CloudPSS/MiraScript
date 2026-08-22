use indexmap::IndexMap;

use crate::standard_library::{insert_native, string};
use crate::{MiraError, MiraValue, MiraValueKind, Result, Runtime, RuntimeErrorKind, operations};

pub(super) fn install(runtime: &mut Runtime) {
    insert_native(runtime, "to_json", |call, args| {
        let Some(value) = args.first().cloned() else {
            return Err(MiraError::runtime(RuntimeErrorKind::MissingArgument {
                name: "data",
            }));
        };
        if value.is_function() {
            return Ok(MiraValue::nil());
        }
        let Some(json) = to_json_value(call, value, false)? else {
            return Ok(MiraValue::nil());
        };
        let source = serde_json::to_string(&json)
            .map_err(|source| MiraError::runtime(RuntimeErrorKind::JsonSerialization { source }))?;
        call.insert(source)
    });
    insert_native(runtime, "from_json", |call, args| {
        let source = string(call, args, 0, "json")?;
        match serde_json::from_str::<serde_json::Value>(&source) {
            Ok(value) => from_json_value(call, value),
            Err(_) if args.len() > 1 => Ok(args[1]),
            Err(source) => Err(MiraError::runtime(RuntimeErrorKind::InvalidJson { source })),
        }
    });
}

fn to_json_value(
    runtime: &mut Runtime,
    value: MiraValue,
    in_container: bool,
) -> Result<Option<serde_json::Value>> {
    Ok(match value.kind() {
        MiraValueKind::Function(_) => {
            if in_container {
                None
            } else {
                return Ok(None);
            }
        }
        MiraValueKind::Nil => Some(serde_json::Value::Null),
        MiraValueKind::Boolean(value) => Some(value.into()),
        MiraValueKind::Number(value) => Some(if !value.is_finite() {
            serde_json::Value::Null
        } else if value == 0.0 {
            serde_json::Value::Number(0.into())
        } else if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            serde_json::Value::Number((value as i64).into())
        } else {
            serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap())
        }),
        MiraValueKind::String(handle) => Some(runtime.get_string(handle)?.to_owned().into()),
        MiraValueKind::StaticStr(value) => Some(value.to_string().into()),
        MiraValueKind::Array(_) => {
            let values = operations::iterable_array(runtime, value)?;
            let mut result = Vec::with_capacity(values.len());
            for item in values {
                result.push(to_json_value(runtime, item, true)?.unwrap_or(serde_json::Value::Null));
            }
            Some(serde_json::Value::Array(result))
        }
        MiraValueKind::Record(_) => {
            let mut map = serde_json::Map::new();
            for key in operations::record_keys(runtime, value)?.unwrap_or_default() {
                let item =
                    operations::record_get(runtime, value, &key)?.unwrap_or_else(MiraValue::nil);
                if let Some(item) = to_json_value(runtime, item, true)? {
                    map.insert(key, item);
                }
            }
            Some(serde_json::Value::Object(map))
        }
        MiraValueKind::Module(_) => {
            let mut map = serde_json::Map::new();
            for key in operations::module_keys(runtime, value)?.unwrap_or_default() {
                let item =
                    operations::module_get(runtime, value, &key)?.unwrap_or_else(MiraValue::nil);
                if let Some(item) = to_json_value(runtime, item, true)? {
                    map.insert(key, item);
                }
            }
            Some(serde_json::Value::Object(map))
        }
        MiraValueKind::Extern(_) => None,
    })
}

fn from_json_value(runtime: &mut Runtime, value: serde_json::Value) -> Result<MiraValue> {
    match value {
        serde_json::Value::Null => Ok(MiraValue::nil()),
        serde_json::Value::Bool(value) => Ok(MiraValue::boolean(value)),
        serde_json::Value::Number(value) => {
            Ok(MiraValue::number(value.as_f64().unwrap_or(f64::NAN)))
        }
        serde_json::Value::String(value) => runtime.insert(value),
        serde_json::Value::Array(values) => {
            let values = values
                .into_iter()
                .map(|value| from_json_value(runtime, value))
                .collect::<Result<Vec<_>>>()?;
            runtime.insert(values)
        }
        serde_json::Value::Object(values) => {
            let mut result = IndexMap::with_capacity(values.len());
            for (key, value) in values {
                result.insert(key, from_json_value(runtime, value)?);
            }
            runtime.insert(result)
        }
    }
}

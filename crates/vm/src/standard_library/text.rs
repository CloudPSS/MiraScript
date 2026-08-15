use indexmap::IndexMap;

use crate::{MiraAny, MiraCallContext, MiraContext, MiraError, Result, operations};

use super::{array, insert_native, string};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "to_string", |_, args| {
        Ok(MiraAny::String(operations::to_string(
            args.first()
                .ok_or_else(|| MiraError::runtime("Parameter 'data' is required"))?,
        )?))
    });
    insert_native(context, "to_number", |_, args| {
        let value = args
            .first()
            .ok_or_else(|| MiraError::runtime("Parameter 'data' is required"))?;
        match operations::to_number(value) {
            Ok(value) => Ok(MiraAny::Number(value)),
            Err(_) if args.len() > 1 => Ok(args[1].clone()),
            Err(error) => Err(error),
        }
    });
    insert_native(context, "format", |_, args| {
        let value = args
            .first()
            .ok_or_else(|| MiraError::runtime("Parameter 'data' is required"))?;
        if args.len() < 2 {
            return Err(MiraError::runtime("Parameter 'format' is required"));
        }
        let specifier = match args.get(1) {
            Some(MiraAny::Nil) => None,
            Some(value) => Some(operations::to_string(value)?),
            None => unreachable!(),
        };
        Ok(MiraAny::String(operations::format_value(
            value,
            specifier.as_deref(),
        )?))
    });

    insert_native(context, "chars", |_, args| {
        Ok(MiraAny::Array(
            string(args, 0, "str")?
                .chars()
                .map(|value| MiraAny::String(value.to_string()))
                .collect(),
        ))
    });
    insert_native(context, "trim_start", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?
                .trim_start_matches(is_javascript_whitespace)
                .into(),
        ))
    });
    insert_native(context, "trim_end", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?
                .trim_end_matches(is_javascript_whitespace)
                .into(),
        ))
    });
    insert_native(context, "trim", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?
                .trim_matches(is_javascript_whitespace)
                .into(),
        ))
    });
    insert_native(context, "to_uppercase", |_, args| {
        Ok(MiraAny::String(string(args, 0, "str")?.to_uppercase()))
    });
    insert_native(context, "to_lowercase", |_, args| {
        Ok(MiraAny::String(string(args, 0, "str")?.to_lowercase()))
    });
    insert_native(context, "starts_with", |_, args| {
        Ok(MiraAny::Boolean(
            string(args, 0, "str")?.starts_with(&string(args, 1, "search")?),
        ))
    });
    insert_native(context, "ends_with", |_, args| {
        Ok(MiraAny::Boolean(
            string(args, 0, "str")?.ends_with(&string(args, 1, "search")?),
        ))
    });
    insert_native(context, "contains", |_, args| {
        Ok(MiraAny::Boolean(
            string(args, 0, "str")?.contains(&string(args, 1, "search")?),
        ))
    });
    insert_native(context, "replace", |_, args| {
        let source = string(args, 0, "str")?;
        let search = string(args, 1, "search")?;
        let replacement = match args.get(2) {
            None => String::new(),
            Some(value) => operations::to_string(value)?,
        };
        Ok(MiraAny::String(source.replace(&search, &replacement)))
    });
    insert_native(context, "split", |_, args| {
        let source = string(args, 0, "str")?;
        let separator = match args.get(1) {
            None => String::new(),
            Some(value) => operations::to_string(value)?,
        };
        let parts: Vec<_> = if separator.is_empty() {
            source.chars().map(|value| value.to_string()).collect()
        } else {
            source.split(&separator).map(str::to_owned).collect()
        };
        Ok(MiraAny::Array(
            parts.into_iter().map(MiraAny::String).collect(),
        ))
    });
    insert_native(context, "join", |_, args| {
        let values = array(args, 0, "arr")?;
        let separator = match args.get(1) {
            None => String::new(),
            Some(value) => operations::to_string(value)?,
        };
        let parts = values
            .iter()
            .map(operations::to_string)
            .collect::<Result<Vec<_>>>()?;
        Ok(MiraAny::String(parts.join(&separator)))
    });

    install_json(context);
    install_debug(context);
}

fn is_javascript_whitespace(value: char) -> bool {
    value.is_whitespace() || value == '\u{feff}'
}

fn install_debug(context: &mut MiraContext) {
    insert_native(context, "debug_print", |call, args| {
        let message = args
            .iter()
            .map(operations::display)
            .collect::<Vec<_>>()
            .join(" ");
        call.options().providers.debug(&message);
        Ok(MiraAny::Nil)
    });
    insert_native(context, "panic", |_, args| {
        let message = args
            .first()
            .map(operations::to_string)
            .transpose()?
            .unwrap_or_else(|| "MiraScript panic".into());
        Err(MiraError::runtime(message))
    });
}

fn install_json(context: &mut MiraContext) {
    insert_native(context, "to_json", |call, args| {
        let Some(value) = args.first() else {
            return Err(MiraError::runtime("Argument `data` is required"));
        };
        if matches!(value, MiraAny::Function(_)) {
            return Ok(MiraAny::Nil);
        }
        let json = to_json_value(call, value, false)?;
        Ok(match json {
            Some(value) => MiraAny::String(serde_json::to_string(&value).map_err(|error| {
                MiraError::runtime(format!("Failed to serialize JSON: {error}"))
            })?),
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
    call: &mut MiraCallContext<'_>,
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
        MiraAny::String(value) => Some(value.clone().into()),
        MiraAny::Array(_) | MiraAny::RustArray(_) => Some(serde_json::Value::Array(
            operations::materialize_array(value)?
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
        MiraAny::Extern(value) => {
            let mut map = serde_json::Map::new();
            for key in value.keys()? {
                let item = call.get(&MiraAny::Extern(value.clone()), key.clone())?;
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
        serde_json::Value::String(value) => MiraAny::String(value),
        serde_json::Value::Array(values) => {
            MiraAny::Array(values.into_iter().map(from_json_value).collect())
        }
        serde_json::Value::Object(values) => MiraAny::Record(
            values
                .into_iter()
                .map(|(key, value)| (key, from_json_value(value)))
                .collect::<IndexMap<_, _>>(),
        ),
    }
}

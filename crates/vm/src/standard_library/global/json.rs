use std::{cell::RefCell, fmt};

use indexmap::IndexMap;
use serde::{
    Deserializer, Serialize, Serializer,
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
};

use crate::{MiraError, MiraValue, MiraValueKind, Result, Runtime, RuntimeErrorKind};
use crate::{
    MiraType,
    standard_library::{insert_native, string},
};

pub(super) fn install(runtime: &mut Runtime) {
    insert_native(runtime, "to_json", |call, args| {
        let Some(value) = args.first().cloned() else {
            return Err(MiraError::runtime(RuntimeErrorKind::MissingArgument {
                name: "data",
            }));
        };
        if matches!(
            value.kind(),
            MiraValueKind::Function(_) | MiraValueKind::Extern(_)
        ) {
            return Ok(MiraValue::NIL);
        }
        let context = SerializeContext::new(call);
        let source = serde_json::to_string(&SerializeValue {
            context: &context,
            value,
        });
        let runtime_error = context.into_runtime_error();
        match (source, runtime_error) {
            (Ok(source), _) => call.insert(source),
            (Err(_), Some(error)) => Err(error),
            (Err(source), None) => Err(MiraError::runtime(RuntimeErrorKind::JsonSerialization {
                source,
            })),
        }
    });
    insert_native(runtime, "from_json", |call, args| {
        let source = string(call, args, 0, "json")?;
        match from_json(call, &source) {
            Ok(value) => Ok(value),
            Err(DeserializeError::Invalid(_)) if args.len() > 1 => Ok(args[1]),
            Err(DeserializeError::Invalid(source)) => {
                Err(MiraError::runtime(RuntimeErrorKind::InvalidJson { source }))
            }
            Err(DeserializeError::Runtime(error)) => Err(error),
        }
    });
}

struct SerializeContext<'runtime> {
    // `Serialize::serialize` only receives `&self`. Keep each Runtime borrow shorter than the
    // recursive serde call so nested values can allocate their projected fields in the arena.
    runtime: RefCell<&'runtime mut Runtime>,
    runtime_error: RefCell<Option<Box<MiraError>>>,
}

impl<'runtime> SerializeContext<'runtime> {
    fn new(runtime: &'runtime mut Runtime) -> Self {
        Self {
            runtime: RefCell::new(runtime),
            runtime_error: RefCell::new(None),
        }
    }

    fn serializer_error<E: serde::ser::Error>(&self, error: Box<MiraError>) -> E {
        let message = error.to_string();
        *self.runtime_error.borrow_mut() = Some(error);
        E::custom(message)
    }

    fn into_runtime_error(self) -> Option<Box<MiraError>> {
        self.runtime_error.into_inner()
    }
}

struct SerializeValue<'context, 'runtime> {
    context: &'context SerializeContext<'runtime>,
    value: MiraValue,
}

impl SerializeValue<'_, '_> {
    fn array_item<E: serde::ser::Error>(
        &self,
        handle: crate::MiraHandle<dyn crate::MiraArray>,
        index: usize,
    ) -> std::result::Result<MiraValue, E> {
        let result: Result<MiraValue> = (|| {
            let mut runtime = self.context.runtime.borrow_mut();
            let manageable = {
                let array = runtime.get_array_dyn(handle)?;
                array.get(handle, &runtime, index)?
            };
            runtime.insert(manageable)
        })();
        result.map_err(|error| self.context.serializer_error(error))
    }

    fn record_key<M: SerializeMap>(
        &self,
        map: &mut M,
        handle: crate::MiraHandle<dyn crate::MiraRecord>,
        index: usize,
    ) -> std::result::Result<(), M::Error> {
        let runtime = self.context.runtime.borrow();
        let record = runtime
            .get_record_dyn(handle)
            .map_err(|error| self.context.serializer_error(error))?;
        let key = record
            .key(index)
            .map_err(|error| self.context.serializer_error(error))?;
        map.serialize_key(key)
    }

    fn record_item<E: serde::ser::Error>(
        &self,
        handle: crate::MiraHandle<dyn crate::MiraRecord>,
        index: usize,
    ) -> std::result::Result<MiraValue, E> {
        let result: Result<MiraValue> = (|| {
            let mut runtime = self.context.runtime.borrow_mut();
            let manageable = {
                let record = runtime.get_record_dyn(handle)?;
                record.get(handle, &runtime, index)?
            };
            runtime.insert(manageable)
        })();
        result.map_err(|error| self.context.serializer_error(error))
    }

    fn module_key<M: SerializeMap>(
        &self,
        map: &mut M,
        handle: crate::MiraHandle<dyn crate::MiraModule>,
        index: usize,
    ) -> std::result::Result<(), M::Error> {
        let runtime = self.context.runtime.borrow();
        let module = runtime
            .get_module_dyn(handle)
            .map_err(|error| self.context.serializer_error(error))?;
        let key = module
            .key(index)
            .map_err(|error| self.context.serializer_error(error))?;
        map.serialize_key(key)
    }

    fn module_item<E: serde::ser::Error>(
        &self,
        handle: crate::MiraHandle<dyn crate::MiraModule>,
        index: usize,
    ) -> std::result::Result<MiraValue, E> {
        let result: Result<MiraValue> = (|| {
            let mut runtime = self.context.runtime.borrow_mut();
            let manageable = {
                let module = runtime.get_module_dyn(handle)?;
                module.get(handle, &runtime, index)?
            };
            runtime.insert(manageable)
        })();
        result.map_err(|error| self.context.serializer_error(error))
    }
}

impl Serialize for SerializeValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self.value.kind() {
            MiraValueKind::Nil | MiraValueKind::Function(_) | MiraValueKind::Extern(_) => {
                serializer.serialize_unit()
            }
            MiraValueKind::Boolean(value) => serializer.serialize_bool(value),
            MiraValueKind::Number(value) if !value.is_finite() => serializer.serialize_unit(),
            MiraValueKind::Number(0.0) => serializer.serialize_i64(0),
            MiraValueKind::Number(value)
                if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 =>
            {
                serializer.serialize_i64(value as i64)
            }
            MiraValueKind::Number(value) => serializer.serialize_f64(value),
            MiraValueKind::String(handle) => {
                let runtime = self.context.runtime.borrow();
                let value = runtime
                    .get_string(handle)
                    .map_err(|error| self.context.serializer_error(error))?;
                serializer.serialize_str(value)
            }
            MiraValueKind::StaticStr(value) => serializer.serialize_str(value),
            MiraValueKind::Array(handle) => {
                let length = {
                    let runtime = self.context.runtime.borrow();
                    runtime
                        .get_array_dyn(handle)
                        .map_err(|error| self.context.serializer_error(error))?
                        .len()
                };
                let mut sequence = serializer.serialize_seq(Some(length))?;
                for index in 0..length {
                    let value = self.array_item(handle, index)?;
                    sequence.serialize_element(&SerializeValue {
                        context: self.context,
                        value,
                    })?;
                }
                sequence.end()
            }
            MiraValueKind::Record(handle) => {
                let length = {
                    let runtime = self.context.runtime.borrow();
                    runtime
                        .get_record_dyn(handle)
                        .map_err(|error| self.context.serializer_error(error))?
                        .len()
                };
                let mut map = serializer.serialize_map(None)?;
                for index in 0..length {
                    let value = self.record_item(handle, index)?;
                    if matches!(value.value_type(), MiraType::Function | MiraType::Extern) {
                        continue;
                    }
                    self.record_key(&mut map, handle, index)?;
                    map.serialize_value(&SerializeValue {
                        context: self.context,
                        value,
                    })?;
                }
                map.end()
            }
            MiraValueKind::Module(handle) => {
                let length = {
                    let runtime = self.context.runtime.borrow();
                    runtime
                        .get_module_dyn(handle)
                        .map_err(|error| self.context.serializer_error(error))?
                        .len()
                };
                let mut map = serializer.serialize_map(None)?;
                for index in 0..length {
                    let value = self.module_item(handle, index)?;
                    if matches!(value.value_type(), MiraType::Function | MiraType::Extern) {
                        continue;
                    }
                    self.module_key(&mut map, handle, index)?;
                    map.serialize_value(&SerializeValue {
                        context: self.context,
                        value,
                    })?;
                }
                map.end()
            }
        }
    }
}

struct DeserializeContext<'runtime> {
    runtime: &'runtime mut Runtime,
    runtime_error: Option<Box<MiraError>>,
}

impl DeserializeContext<'_> {
    fn insert<E: serde::de::Error>(
        &mut self,
        value: impl Into<crate::MiraManageable>,
    ) -> std::result::Result<MiraValue, E> {
        self.runtime.insert(value).map_err(|error| {
            let message = error.to_string();
            self.runtime_error = Some(error);
            E::custom(message)
        })
    }
}

struct DeserializeValue<'context, 'runtime> {
    context: &'context mut DeserializeContext<'runtime>,
}

impl<'de> DeserializeSeed<'de> for DeserializeValue<'_, '_> {
    type Value = MiraValue;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for DeserializeValue<'_, '_> {
    type Value = MiraValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(MiraValue::NIL)
    }

    fn visit_bool<E: serde::de::Error>(self, value: bool) -> std::result::Result<Self::Value, E> {
        Ok(MiraValue::boolean(value))
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> std::result::Result<Self::Value, E> {
        Ok(MiraValue::number(value as f64))
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> std::result::Result<Self::Value, E> {
        Ok(MiraValue::number(value as f64))
    }

    fn visit_f64<E: serde::de::Error>(self, value: f64) -> std::result::Result<Self::Value, E> {
        Ok(MiraValue::number(value))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
        self.context.insert(value)
    }

    fn visit_string<E: serde::de::Error>(
        self,
        value: String,
    ) -> std::result::Result<Self::Value, E> {
        self.context.insert(value)
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(value) = sequence.next_element_seed(DeserializeValue {
            context: &mut *self.context,
        })? {
            values.push(value);
        }
        self.context.insert(values)
    }

    fn visit_map<A: MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut values = IndexMap::with_capacity(map.size_hint().unwrap_or(0));
        while let Some(key) = map.next_key::<String>()? {
            let value = map.next_value_seed(DeserializeValue {
                context: &mut *self.context,
            })?;
            values.insert(key, value);
        }
        self.context.insert(values)
    }
}

fn from_json(
    runtime: &mut Runtime,
    source: &str,
) -> std::result::Result<MiraValue, DeserializeError> {
    let mut context = DeserializeContext {
        runtime,
        runtime_error: None,
    };
    let mut deserializer = serde_json::Deserializer::from_str(source);
    let result = DeserializeValue {
        context: &mut context,
    }
    .deserialize(&mut deserializer)
    .and_then(|value| {
        deserializer.end()?;
        Ok(value)
    });
    match (result, context.runtime_error) {
        (Ok(value), _) => Ok(value),
        (Err(_), Some(error)) => Err(DeserializeError::Runtime(error)),
        (Err(error), None) => Err(DeserializeError::Invalid(error)),
    }
}

enum DeserializeError {
    Invalid(serde_json::Error),
    Runtime(Box<MiraError>),
}

use std::cmp::Ordering;

use indexmap::IndexMap;

use crate::{MiraAny, MiraCallContext, MiraContext, MiraError, Result, operations};

use super::{const_value, insert_native, is_callable, required};

enum Data {
    Primitive(MiraAny),
    Array(Vec<MiraAny>),
    Record(IndexMap<String, MiraAny>),
}

impl Data {
    fn from_value(value: &MiraAny) -> Result<Self> {
        match value {
            MiraAny::Nil | MiraAny::Boolean(_) | MiraAny::Number(_) | MiraAny::String(_) => {
                Ok(Self::Primitive(value.clone()))
            }
            MiraAny::Array(_) | MiraAny::RustArray(_) => {
                Ok(Self::Array(operations::materialize_array(value)?))
            }
            MiraAny::Record(_) | MiraAny::RustRecord(_) => {
                let mut record = IndexMap::new();
                for key in value.record_keys()?.unwrap_or_default() {
                    record.insert(key.clone(), value.record_get(&key)?.unwrap_or(MiraAny::Nil));
                }
                Ok(Self::Record(record))
            }
            _ => Err(MiraError::runtime(format!(
                "Expected nil, number, boolean, string, array or record, got {}",
                operations::display(value)
            ))),
        }
    }

    fn original(&self) -> MiraAny {
        match self {
            Self::Primitive(value) => value.clone(),
            Self::Array(value) => MiraAny::Array(value.clone()),
            Self::Record(value) => MiraAny::Record(value.clone()),
        }
    }
}

pub(super) fn install(context: &mut MiraContext) {
    install_entries(context);
    install_mapping(context);
    install_search(context);
    install_ordering(context);
    install_construction(context);
}

fn install_entries(context: &mut MiraContext) {
    insert_native(context, "len", |_, args| {
        let value = required(args, 0, "arr")?;
        let Some(length) = value.array_len()? else {
            return Err(MiraError::runtime("Argument `arr` is not an array"));
        };
        Ok(MiraAny::Number(length as f64))
    });
    insert_native(context, "keys", |_, args| {
        let value = required(args, 0, "data")?;
        let keys = match value {
            MiraAny::Array(_) | MiraAny::RustArray(_) => (0..value.array_len()?.unwrap_or(0))
                .map(|index| MiraAny::Number(index as f64))
                .collect(),
            MiraAny::Record(_) | MiraAny::RustRecord(_) => value
                .record_keys()?
                .unwrap_or_default()
                .into_iter()
                .map(MiraAny::String)
                .collect(),
            MiraAny::Module(module) => module.keys().into_iter().map(MiraAny::String).collect(),
            MiraAny::Extern(value) => value.keys()?.into_iter().map(MiraAny::String).collect(),
            _ => {
                return Err(MiraError::runtime(
                    "Argument `data` is not a compound value",
                ));
            }
        };
        Ok(MiraAny::Array(keys))
    });
    insert_native(context, "values", |_, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        match data {
            Data::Array(values) => Ok(MiraAny::Array(values)),
            Data::Record(values) => Ok(MiraAny::Array(values.into_values().collect())),
            Data::Primitive(_) => Err(MiraError::runtime("Argument `data` is not array | record")),
        }
    });
    insert_native(context, "entries", |_, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        let entries = match data {
            Data::Array(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| pair(MiraAny::Number(index as f64), value))
                .collect(),
            Data::Record(values) => values
                .into_iter()
                .map(|(key, value)| pair(MiraAny::String(key), value))
                .collect(),
            Data::Primitive(_) => {
                return Err(MiraError::runtime("Argument `data` is not array | record"));
            }
        };
        Ok(MiraAny::Array(entries))
    });
}

fn install_mapping(context: &mut MiraContext) {
    insert_native(context, "map", |call, args| {
        map_like(call, args, MapMode::Map)
    });
    insert_native(context, "filter", |call, args| {
        map_like(call, args, MapMode::Filter)
    });
    insert_native(context, "filter_map", |call, args| {
        map_like(call, args, MapMode::FilterMap)
    });
    insert_native(context, "fold", |call, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        let mut accumulator = required(args, 1, "initial")?.clone();
        let function = required(args, 2, "f")?;
        if !is_callable(function)? {
            return Err(MiraError::runtime("Argument `f` is not callable"));
        }
        let original = data.original();
        for (key, value) in data_items(&data) {
            call.checkpoint()?;
            accumulator = call.call(function, &[accumulator, value, key, original.clone()])?;
        }
        Ok(accumulator)
    });
    insert_native(context, "group_by", |call, args| {
        let data = array_value(required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime("Argument `key` is not callable"));
        }
        let original = MiraAny::Array(data.clone());
        let mut groups: IndexMap<String, Vec<MiraAny>> = IndexMap::new();
        for (index, value) in data.into_iter().enumerate() {
            call.checkpoint()?;
            let key = call.call(
                key_function,
                &[
                    value.clone(),
                    MiraAny::Number(index as f64),
                    original.clone(),
                ],
            )?;
            groups
                .entry(operations::to_string(&key)?)
                .or_default()
                .push(value);
        }
        Ok(MiraAny::Record(
            groups
                .into_iter()
                .map(|(key, values)| (key, MiraAny::Array(values)))
                .collect(),
        ))
    });
    insert_native(context, "flatten", |_, args| {
        let values = array_value(required(args, 0, "data")?)?;
        let depth = match args.get(1) {
            None | Some(MiraAny::Nil) => 1,
            Some(value) => operations::to_number(value)?.trunc().max(0.0) as usize,
        };
        Ok(MiraAny::Array(flatten(values, depth)?))
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

fn install_search(context: &mut MiraContext) {
    insert_native(context, "find", |call, args| {
        let data = Data::from_value(required(args, 0, "data")?)?;
        let predicate = required(args, 1, "predicate")?;
        let callable = is_callable(predicate)?;
        let original = data.original();
        for (key, value) in data_items(&data) {
            call.checkpoint()?;
            let found = if callable {
                operations::to_boolean(
                    &call.call(predicate, &[value.clone(), key.clone(), original.clone()])?,
                )?
            } else {
                &value == predicate
            };
            if found {
                return Ok(pair(key, value));
            }
        }
        Ok(MiraAny::Nil)
    });
    for (name, every) in [("all", true), ("any", false)] {
        insert_native(context, name, move |call, args| {
            let data = Data::from_value(required(args, 0, "data")?)?;
            let predicate = required(args, 1, "predicate")?;
            if !is_callable(predicate)? {
                return Err(MiraError::runtime("Argument `predicate` is not callable"));
            }
            let original = data.original();
            for (key, value) in data_items(&data) {
                call.checkpoint()?;
                let matched = operations::to_boolean(
                    &call.call(predicate, &[value, key, original.clone()])?,
                )?;
                if every && !matched {
                    return Ok(MiraAny::Boolean(false));
                }
                if !every && matched {
                    return Ok(MiraAny::Boolean(true));
                }
            }
            Ok(MiraAny::Boolean(every))
        });
    }
}

fn install_ordering(context: &mut MiraContext) {
    insert_native(context, "reverse", |_, args| {
        let mut values = array_value(required(args, 0, "arr")?)?;
        values.reverse();
        Ok(MiraAny::Array(values))
    });
    insert_native(context, "sort", |call, args| {
        let mut values = array_value(required(args, 0, "data")?)?;
        insertion_sort(call, &mut values, args.get(1))?;
        Ok(MiraAny::Array(values))
    });
    insert_native(context, "sort_by", |call, args| {
        let values = array_value(required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime("Argument `key` is not callable"));
        }
        let original = MiraAny::Array(values.clone());
        let mut keyed = Vec::new();
        for (index, value) in values.into_iter().enumerate() {
            let key = call.call(
                key_function,
                &[
                    value.clone(),
                    MiraAny::Number(index as f64),
                    original.clone(),
                ],
            )?;
            keyed.push(pair(key, value));
        }
        insertion_sort_by(call, &mut keyed, args.get(2), |value| {
            value
                .record_get("0")
                .map(|value| value.unwrap_or(MiraAny::Nil))
        })?;
        Ok(MiraAny::Array(
            keyed
                .into_iter()
                .map(|value| {
                    value
                        .record_get("1")
                        .map(|item| item.unwrap_or(MiraAny::Nil))
                })
                .collect::<Result<Vec<_>>>()?,
        ))
    });
    insert_native(context, "unique", |call, args| {
        let values = array_value(required(args, 0, "data")?)?;
        validate_optional_callable(args.get(1), "equal")?;
        let mut result = Vec::new();
        for value in values {
            let mut found = false;
            for existing in &result {
                if equal(call, &value, existing, args.get(1))? {
                    found = true;
                    break;
                }
            }
            if !found {
                result.push(value);
            }
        }
        Ok(MiraAny::Array(result))
    });
    insert_native(context, "unique_by", |call, args| {
        let values = array_value(required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime("Argument `key` is not callable"));
        }
        validate_optional_callable(args.get(2), "equal")?;
        let original = MiraAny::Array(values.clone());
        let mut result = Vec::new();
        let mut keys = Vec::new();
        for (index, value) in values.into_iter().enumerate() {
            let key = call.call(
                key_function,
                &[
                    value.clone(),
                    MiraAny::Number(index as f64),
                    original.clone(),
                ],
            )?;
            let mut found = false;
            for existing in &keys {
                if equal(call, &key, existing, args.get(2))? {
                    found = true;
                    break;
                }
            }
            if !found {
                keys.push(key);
                result.push(value);
            }
        }
        Ok(MiraAny::Array(result))
    });
}

fn validate_optional_callable(value: Option<&MiraAny>, name: &str) -> Result<()> {
    if let Some(value) = value.filter(|value| **value != MiraAny::Nil)
        && !is_callable(value)?
    {
        return Err(MiraError::runtime(format!(
            "Argument `{name}` is not callable"
        )));
    }
    Ok(())
}

fn install_construction(context: &mut MiraContext) {
    insert_native(context, "repeat", |call, args| {
        let value = const_value(required(args, 0, "data")?.clone())?;
        let length = array_length(required(args, 1, "times")?, call.options().max_array_len)?;
        Ok(MiraAny::Array(vec![value; length]))
    });
    insert_native(context, "new_array", |call, args| {
        let length = array_length(required(args, 0, "length")?, call.options().max_array_len)?;
        let generator = required(args, 1, "generator")?;
        if !is_callable(generator)? {
            return Err(MiraError::runtime("Argument `generator` is not callable"));
        }
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            call.checkpoint()?;
            result.push(const_value(
                call.call(generator, &[MiraAny::Number(index as f64)])?,
            )?);
        }
        Ok(MiraAny::Array(result))
    });
    insert_native(context, "new_record", |call, args| {
        let length = array_length(required(args, 0, "size")?, call.options().max_array_len)?;
        let generator = required(args, 1, "generator")?;
        if !is_callable(generator)? {
            return Err(MiraError::runtime("Argument `generator` is not callable"));
        }
        let mut result = IndexMap::new();
        for index in 0..length {
            call.checkpoint()?;
            let entry = call.call(generator, &[MiraAny::Number(index as f64)])?;
            if entry == MiraAny::Nil {
                continue;
            }
            let key =
                operations::to_string(&operations::get_value(&entry, &MiraAny::Number(0.0))?)?;
            let value = operations::get_value(&entry, &MiraAny::Number(1.0))?.into_element()?;
            result.insert(key, value);
        }
        Ok(MiraAny::Record(result))
    });
    insert_native(context, "zip", |call, args| {
        zip(call, required(args, 0, "data")?)
    });
    insert_native(context, "with", |call, args| {
        update_with(
            required(args, 0, "data")?,
            &args[1..],
            call.options().max_array_len,
        )
    });
}

fn zip(call: &mut MiraCallContext<'_>, value: &MiraAny) -> Result<MiraAny> {
    let data = Data::from_value(value)?;
    let items = data_items(&data);
    let mut arrays = Vec::new();
    let mut length = 0;
    for (key, value) in items {
        let array = array_value(&value)?;
        length = length.max(array.len());
        arrays.push((key, array));
    }
    let mut result = Vec::with_capacity(length);
    for index in 0..length {
        call.checkpoint()?;
        match &data {
            Data::Array(_) => result.push(MiraAny::Array(
                arrays
                    .iter()
                    .map(|(_, array)| array.get(index).cloned().unwrap_or(MiraAny::Nil))
                    .collect(),
            )),
            Data::Record(_) => result.push(MiraAny::Record(
                arrays
                    .iter()
                    .map(|(key, array)| {
                        let MiraAny::String(key) = key else {
                            unreachable!()
                        };
                        (
                            key.clone(),
                            array.get(index).cloned().unwrap_or(MiraAny::Nil),
                        )
                    })
                    .collect(),
            )),
            Data::Primitive(_) => {
                return Err(MiraError::runtime("Argument `data` is not array | record"));
            }
        }
    }
    Ok(MiraAny::Array(result))
}

fn update_with(data: &MiraAny, entries: &[MiraAny], max_len: usize) -> Result<MiraAny> {
    if !entries.len().is_multiple_of(2) {
        return Err(MiraError::runtime("Expected even number of entries"));
    }
    let mut result = match Data::from_value(data)? {
        Data::Array(values) => MiraAny::Array(values),
        Data::Record(values) => MiraAny::Record(values),
        Data::Primitive(_) => {
            return Err(MiraError::runtime("Argument `data` is not array | record"));
        }
    };
    for pair in entries.chunks_exact(2) {
        let path = if pair[0].array_len()?.is_some() {
            operations::materialize_array(&pair[0])?
        } else if pair[0] == MiraAny::Nil {
            continue;
        } else {
            vec![pair[0].clone()]
        };
        if path.is_empty() || path.contains(&MiraAny::Nil) {
            continue;
        }
        result = set_path(result, &path, const_value(pair[1].clone())?, max_len)?;
    }
    Ok(result)
}

fn set_path(
    mut data: MiraAny,
    path: &[MiraAny],
    value: MiraAny,
    max_len: usize,
) -> Result<MiraAny> {
    if path.is_empty() {
        return Ok(value);
    }
    match &mut data {
        MiraAny::Array(values) => {
            let index = array_index(&path[0], max_len)?;
            while values.len() <= index {
                values.push(MiraAny::Nil);
            }
            let current = values[index].clone();
            values[index] = set_path(
                container_for(&current, path.get(1)),
                &path[1..],
                value,
                max_len,
            )?;
        }
        MiraAny::Record(values) => {
            let key = operations::to_string(&path[0])?;
            let current = values.get(&key).cloned().unwrap_or(MiraAny::Nil);
            values.insert(
                key,
                set_path(
                    container_for(&current, path.get(1)),
                    &path[1..],
                    value,
                    max_len,
                )?,
            );
        }
        _ => {
            data = container_for(&data, path.first());
            return set_path(data, path, value, max_len);
        }
    }
    Ok(data)
}

fn container_for(current: &MiraAny, next: Option<&MiraAny>) -> MiraAny {
    if matches!(current, MiraAny::Array(_) | MiraAny::Record(_)) {
        return current.clone();
    }
    if next.is_some_and(
        |value| matches!(value, MiraAny::Number(number) if number.fract() == 0.0 && *number >= 0.0),
    ) {
        MiraAny::Array(Vec::new())
    } else {
        MiraAny::Record(IndexMap::new())
    }
}

fn array_index(value: &MiraAny, max_len: usize) -> Result<usize> {
    let index = operations::to_number(value)?;
    if !index.is_finite() || index < 0.0 {
        return Err(MiraError::runtime(
            "Array index must be a non-negative integer",
        ));
    }
    let index = index.trunc() as usize;
    if index >= max_len {
        return Err(MiraError::runtime(format!(
            "Array index exceeds maximum limit of {max_len}"
        )));
    }
    Ok(index)
}

fn array_length(value: &MiraAny, max_len: usize) -> Result<usize> {
    let length = operations::to_number(value)?;
    if !length.is_finite() || length <= -1.0 {
        return Err(MiraError::runtime(
            "Array length must be a non-negative integer",
        ));
    }
    let length = length.trunc() as usize;
    if length > max_len {
        return Err(MiraError::runtime(format!(
            "Array length exceeds maximum limit of {max_len}"
        )));
    }
    Ok(length)
}

fn data_items(data: &Data) -> Vec<(MiraAny, MiraAny)> {
    match data {
        Data::Primitive(value) => vec![(MiraAny::Nil, value.clone())],
        Data::Array(values) => values
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| (MiraAny::Number(index as f64), value))
            .collect(),
        Data::Record(values) => values
            .iter()
            .map(|(key, value)| (MiraAny::String(key.clone()), value.clone()))
            .collect(),
    }
}

fn array_value(value: &MiraAny) -> Result<Vec<MiraAny>> {
    operations::materialize_array(value)
}

fn pair(first: MiraAny, second: MiraAny) -> MiraAny {
    MiraAny::Record(IndexMap::from([("0".into(), first), ("1".into(), second)]))
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

fn insertion_sort(
    call: &mut MiraCallContext<'_>,
    values: &mut [MiraAny],
    comparator: Option<&MiraAny>,
) -> Result<()> {
    insertion_sort_by(call, values, comparator, |value| Ok(value.clone()))
}

fn insertion_sort_by<T>(
    call: &mut MiraCallContext<'_>,
    values: &mut [T],
    comparator: Option<&MiraAny>,
    key: impl Fn(&T) -> Result<MiraAny>,
) -> Result<()> {
    if let Some(value) = comparator.filter(|value| **value != MiraAny::Nil)
        && !is_callable(value)?
    {
        return Err(MiraError::runtime("Argument `comparator` is not callable"));
    }
    for index in 1..values.len() {
        let mut position = index;
        while position > 0 {
            let left = key(&values[position - 1])?;
            let right = key(&values[position])?;
            let ordering =
                if let Some(comparator) = comparator.filter(|value| **value != MiraAny::Nil) {
                    operations::to_number(&call.call(comparator, &[left, right])?)?
                        .partial_cmp(&0.0)
                        .unwrap_or(Ordering::Equal)
                } else {
                    default_compare(&left, &right)
                };
            if ordering != Ordering::Greater {
                break;
            }
            values.swap(position - 1, position);
            position -= 1;
        }
    }
    Ok(())
}

fn default_compare(left: &MiraAny, right: &MiraAny) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }
    if matches!(left, MiraAny::Nil | MiraAny::String(_))
        && matches!(right, MiraAny::Nil | MiraAny::String(_))
    {
        return operations::to_string(left)
            .unwrap_or_default()
            .cmp(&operations::to_string(right).unwrap_or_default());
    }
    let left = operations::to_number(left).unwrap_or(0.0);
    let right = operations::to_number(right).unwrap_or(0.0);
    let left = if left == 0.0 || left.is_nan() {
        0.0
    } else {
        left
    };
    let right = if right == 0.0 || right.is_nan() {
        0.0
    } else {
        right
    };
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

fn equal(
    call: &mut MiraCallContext<'_>,
    left: &MiraAny,
    right: &MiraAny,
    equaler: Option<&MiraAny>,
) -> Result<bool> {
    if let Some(equaler) = equaler.filter(|value| **value != MiraAny::Nil) {
        if !is_callable(equaler)? {
            return Err(MiraError::runtime("Argument `equal` is not callable"));
        }
        operations::to_boolean(&call.call(equaler, &[left.clone(), right.clone()])?)
    } else {
        Ok(left == right)
    }
}

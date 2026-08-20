use std::cmp::Ordering;

use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "sort", |call, args| {
        let mut values = array_value(required(args, 0, "data")?)?;
        insertion_sort(call, &mut values, args.get(1))?;
        Ok(MiraAny::Array(values.into()))
    });
    insert_native(context, "sort_by", |call, args| {
        let values = array_value(required(args, 0, "data")?)?;
        let key_function = required(args, 1, "key")?;
        if !is_callable(key_function)? {
            return Err(MiraError::runtime("Argument `key` is not callable"));
        }
        let original = MiraAny::Array(values.clone().into());
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
                .collect::<Result<Vec<_>>>()?
                .into(),
        ))
    });
}

fn insertion_sort(
    call: &mut Runtime<'_>,
    values: &mut [MiraAny],
    comparator: Option<&MiraAny>,
) -> Result<()> {
    insertion_sort_by(call, values, comparator, |value| Ok(value.clone()))
}

fn insertion_sort_by<T>(
    call: &mut Runtime<'_>,
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

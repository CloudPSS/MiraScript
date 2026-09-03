use std::cmp::Ordering;

use crate::MiraFunctionHandle;

use super::*;

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(runtime, fn sort(call, args) {
        let mut values = array_value(call, *required(args, 0, "data")?)?;
        insertion_sort(call, &mut values, optional_callable(args, 1, "comparator")?)?;
        call.insert(values)
    });
    global_builtin!(runtime, fn sort_by(call, args) {
        let values = array_value(call, *required(args, 0, "data")?)?;
        let key_function = callable(args, 1, "key")?;
        let original = call.insert(values.clone())?;
        let mut keyed = Vec::new();
        for (index, value) in values.into_iter().enumerate() {
            let key = key_function.call(call, &[value, MiraValue::number(index as f64), original])?;
            keyed.push((key, value));
        }
        insertion_sort_by(call, &mut keyed, optional_callable(args, 2, "comparator")?, |value| Ok(value.0))?;
        call.insert(
            keyed
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
        )
    });
}

fn insertion_sort(
    call: &mut Runtime,
    values: &mut [MiraValue],
    comparator: Option<MiraFunctionHandle>,
) -> Result<()> {
    insertion_sort_by(call, values, comparator, |value| Ok(*value))
}

fn insertion_sort_by<T>(
    call: &mut Runtime,
    values: &mut [T],
    comparator: Option<MiraFunctionHandle>,
    key: impl Fn(&T) -> Result<MiraValue>,
) -> Result<()> {
    for index in 1..values.len() {
        let mut position = index;
        while position > 0 {
            let left = key(&values[position - 1])?;
            let right = key(&values[position])?;
            let ordering = if let Some(comparator) = comparator {
                let compared = comparator.call(call, &[left, right])?;
                operations::to_number(call, compared)?
                    .partial_cmp(&0.0)
                    .unwrap_or(Ordering::Equal)
            } else {
                default_compare(call, left, right)
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

fn default_compare(runtime: &mut Runtime, left: MiraValue, right: MiraValue) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }
    if (left.is_nil() || left.is_string()) && (right.is_nil() || right.is_string()) {
        return operations::to_string(runtime, left)
            .unwrap_or_default()
            .cmp(&operations::to_string(runtime, right).unwrap_or_default());
    }
    let left = operations::to_number(runtime, left).unwrap_or(0.0);
    let right = operations::to_number(runtime, right).unwrap_or(0.0);
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

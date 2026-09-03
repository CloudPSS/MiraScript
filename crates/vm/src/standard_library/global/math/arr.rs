use crate::standard_library::global_builtin;
use crate::{MiraValue, Result, Runtime, operations};

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn max(call, args) {
        let values = numbers(call, args)?;
        let mut result = f64::NEG_INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraValue::number(f64::NAN));
            }
            if value > result || (value == 0.0 && result == 0.0 && value.is_sign_positive()) {
                result = value;
            }
        }
        Ok(MiraValue::number(result))
    });
    global_builtin!(context, fn min(call, args) {
        let values = numbers(call, args)?;
        let mut result = f64::INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraValue::number(f64::NAN));
            }
            if value < result || (value == 0.0 && result == 0.0 && value.is_sign_negative()) {
                result = value;
            }
        }
        Ok(MiraValue::number(result))
    });
    global_builtin!(context, fn hypot(call, args) {
        let values = numbers(call, args)?;
        if values.iter().any(|value| value.is_infinite()) {
            return Ok(MiraValue::number(f64::INFINITY));
        }
        if values.iter().any(|value| value.is_nan()) {
            return Ok(MiraValue::number(f64::NAN));
        }
        Ok(MiraValue::number(
            values.into_iter().fold(0.0_f64, f64::hypot),
        ))
    });
    global_builtin!(context, fn sum(call, args) {
        let mut total = -0.0;
        for value in numbers(call, args)? {
            total += value;
        }
        Ok(MiraValue::number(total))
    });
    global_builtin!(context, fn product(call, args) {
        Ok(MiraValue::number(
            numbers(call, args)?.into_iter().product(),
        ))
    });
}

fn numbers(runtime: &mut Runtime, args: &[MiraValue]) -> Result<Vec<f64>> {
    if args.len() == 1 && operations::array_len(runtime, args[0])?.is_some() {
        let iter = operations::iterate_array(runtime, args[0])?;
        let mut numbers = Vec::with_capacity(iter.len());
        for entry in iter {
            let value = entry.get(runtime)?;
            numbers.push(operations::to_number(runtime, value)?);
        }
        Ok(numbers)
    } else {
        args.iter()
            .map(|value| operations::to_number(runtime, *value))
            .collect()
    }
}

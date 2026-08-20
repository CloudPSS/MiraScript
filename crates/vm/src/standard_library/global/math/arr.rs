use crate::standard_library::insert_native;
use crate::{MiraValue, Result, Runtime, operations};

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "max", |call, args| {
        let values = numbers(call, args)?;
        let mut result = f64::NEG_INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraValue::Number(f64::NAN));
            }
            if value > result || (value == 0.0 && result == 0.0 && value.is_sign_positive()) {
                result = value;
            }
        }
        Ok(MiraValue::Number(result))
    });
    insert_native(context, "min", |call, args| {
        let values = numbers(call, args)?;
        let mut result = f64::INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraValue::Number(f64::NAN));
            }
            if value < result || (value == 0.0 && result == 0.0 && value.is_sign_negative()) {
                result = value;
            }
        }
        Ok(MiraValue::Number(result))
    });
    insert_native(context, "hypot", |call, args| {
        let values = numbers(call, args)?;
        if values.iter().any(|value| value.is_infinite()) {
            return Ok(MiraValue::Number(f64::INFINITY));
        }
        if values.iter().any(|value| value.is_nan()) {
            return Ok(MiraValue::Number(f64::NAN));
        }
        Ok(MiraValue::Number(
            values.into_iter().fold(0.0_f64, f64::hypot),
        ))
    });
    insert_native(context, "sum", |call, args| {
        let mut total = -0.0;
        for value in numbers(call, args)? {
            total += value;
        }
        Ok(MiraValue::Number(total))
    });
    insert_native(context, "product", |call, args| {
        Ok(MiraValue::Number(
            numbers(call, args)?.into_iter().product(),
        ))
    });
}

fn numbers(runtime: &mut Runtime, args: &[MiraValue]) -> Result<Vec<f64>> {
    let values = if args.len() == 1 && operations::array_len(runtime, args[0])?.is_some() {
        operations::iterable_array(runtime, args[0])?
    } else {
        args.to_vec()
    };
    values
        .iter()
        .map(|value| operations::to_number(runtime, *value))
        .collect()
}

use crate::standard_library::insert_native;
use crate::{MiraAny, MiraContext, Result, operations};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "max", |_, args| {
        let values = numbers(args)?;
        let mut result = f64::NEG_INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraAny::Number(f64::NAN));
            }
            if value > result || (value == 0.0 && result == 0.0 && value.is_sign_positive()) {
                result = value;
            }
        }
        Ok(MiraAny::Number(result))
    });
    insert_native(context, "min", |_, args| {
        let values = numbers(args)?;
        let mut result = f64::INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraAny::Number(f64::NAN));
            }
            if value < result || (value == 0.0 && result == 0.0 && value.is_sign_negative()) {
                result = value;
            }
        }
        Ok(MiraAny::Number(result))
    });
    insert_native(context, "hypot", |_, args| {
        let values = numbers(args)?;
        if values.iter().any(|value| value.is_infinite()) {
            return Ok(MiraAny::Number(f64::INFINITY));
        }
        if values.iter().any(|value| value.is_nan()) {
            return Ok(MiraAny::Number(f64::NAN));
        }
        Ok(MiraAny::Number(
            values.into_iter().fold(0.0_f64, f64::hypot),
        ))
    });
    insert_native(context, "sum", |_, args| {
        let mut total = -0.0;
        for value in numbers(args)? {
            total += value;
        }
        Ok(MiraAny::Number(total))
    });
    insert_native(context, "product", |_, args| {
        Ok(MiraAny::Number(numbers(args)?.into_iter().product()))
    });
}

fn numbers(args: &[MiraAny]) -> Result<Vec<f64>> {
    let values = if args.len() == 1 && args[0].array_len()?.is_some() {
        operations::iterable_array(&args[0])?
    } else {
        args.to_vec()
    };
    values.iter().map(operations::to_number).collect()
}

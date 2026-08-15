use crate::standard_library::{insert_native, number};
use crate::{MiraAny, MiraContext, MiraError, operations};

pub(super) fn install(context: &mut MiraContext) {
    for (name, operation) in [
        ("trunc", f64::trunc as fn(f64) -> f64),
        ("floor", f64::floor),
        ("ceil", f64::ceil),
        ("round", round_ties_even),
    ] {
        insert_native(context, name, move |_, args| {
            let value = number(args, 0, "x")?;
            let digits = match args.get(1) {
                None | Some(MiraAny::Nil) => 0,
                Some(value) => {
                    let digits = operations::to_number(value)?;
                    if !digits.is_finite() {
                        return Err(MiraError::runtime("Argument `n` must be a finite integer"));
                    }
                    digits.trunc() as i32
                }
            };
            if !(0..=15).contains(&digits) {
                return Err(MiraError::runtime("Argument `n` must be between 0 and 15"));
            }
            if digits == 0 {
                Ok(MiraAny::Number(operation(value)))
            } else {
                let factor = 10f64.powi(digits);
                Ok(MiraAny::Number(operation(value * factor) / factor))
            }
        });
    }
}

fn round_ties_even(value: f64) -> f64 {
    let rounded = value.round();
    if (value - rounded).abs() == 0.5 {
        (value / 2.0).round() * 2.0
    } else {
        rounded
    }
}

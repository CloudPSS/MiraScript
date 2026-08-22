use crate::standard_library::{insert_native, number};
use crate::{MiraError, MiraValue, Runtime, RuntimeErrorKind, operations};

pub(super) fn install(context: &mut Runtime) {
    for (name, operation) in [
        ("trunc", f64::trunc as fn(f64) -> f64),
        ("floor", f64::floor),
        ("ceil", f64::ceil),
        ("round", round_ties_even),
    ] {
        insert_native(context, name, move |call, args| {
            let value = number(call, args, 0, "x")?;
            let digits = match args.get(1) {
                None => 0,
                Some(value) if value.is_nil() => 0,
                Some(value) => {
                    let digits = operations::to_number(call, *value)?;
                    if !digits.is_finite() {
                        return Err(MiraError::runtime(
                            RuntimeErrorKind::InvalidIntegerArgument {
                                name: "n",
                                constraint: "a finite integer",
                            },
                        ));
                    }
                    digits.trunc() as i32
                }
            };
            if !(0..=15).contains(&digits) {
                return Err(MiraError::runtime(
                    RuntimeErrorKind::InvalidIntegerArgument {
                        name: "n",
                        constraint: "between 0 and 15",
                    },
                ));
            }
            if digits == 0 {
                Ok(MiraValue::number(operation(value)))
            } else {
                let factor = 10f64.powi(digits);
                Ok(MiraValue::number(operation(value * factor) / factor))
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

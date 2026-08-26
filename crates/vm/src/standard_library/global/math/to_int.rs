use crate::standard_library::{global_builtin, number};
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(
        runtime,
        fn trunc: to_int(f64::trunc);
        fn floor: to_int(f64::floor);
        fn ceil: to_int(f64::ceil);
        fn round: to_int(round_ties_even);
    );
}

fn round_ties_even(value: f64) -> f64 {
    let rounded = value.round();
    if (value - rounded).abs() == 0.5 {
        (value / 2.0).round() * 2.0
    } else {
        rounded
    }
}

fn to_int(
    operation: impl Fn(f64) -> f64,
) -> impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraValue> {
    move |call, args| {
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
    }
}

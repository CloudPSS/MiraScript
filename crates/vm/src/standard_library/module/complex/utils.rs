use crate::standard_library::required;
use crate::{MiraValue, Result, Runtime, operations};

pub(super) type C = (f64, f64);
pub(super) const INF: f64 = f64::INFINITY;
pub(super) const NAN: f64 = f64::NAN;

pub(super) fn parse(
    runtime: &mut Runtime,
    args: &[MiraValue],
    index: usize,
    name: &'static str,
) -> Result<C> {
    let value = *required(args, index, name)?;
    if let Some(handle) = value.as_record() {
        // Check both keys before materializing either field of a host record.
        let record = runtime.get_record_dyn(handle)?;
        if record.index_of("0").is_some() && record.index_of("1").is_some() {
            let re = operations::record_get(runtime, value, "0")?.unwrap_or(MiraValue::NIL);
            let im = operations::record_get(runtime, value, "1")?.unwrap_or(MiraValue::NIL);
            return Ok((
                operations::to_number(runtime, re)?,
                operations::to_number(runtime, im)?,
            ));
        }
    }
    Ok((operations::to_number(runtime, value)?, 0.0))
}

pub(super) fn unary(runtime: &mut Runtime, args: &[MiraValue], f: fn(C) -> C) -> Result<MiraValue> {
    let value = parse(runtime, args, 0, "z")?;
    runtime.insert(f(value))
}

pub(super) fn binary(
    runtime: &mut Runtime,
    args: &[MiraValue],
    f: fn(C, C) -> C,
) -> Result<MiraValue> {
    let a = parse(runtime, args, 0, "a")?;
    let b = parse(runtime, args, 1, "b")?;
    runtime.insert(f(a, b))
}

pub(super) fn radial(r: f64, x: f64) -> f64 {
    if x == 0.0 && !r.is_nan() {
        0.0_f64.copysign(r) * x
    } else {
        r * x
    }
}

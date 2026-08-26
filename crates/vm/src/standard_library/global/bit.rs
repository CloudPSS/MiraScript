use crate::standard_library::{global_builtin, number};
use crate::{MiraValue, Runtime};

pub(super) fn to_u32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}

pub(super) fn install(runtime: &mut Runtime) {
    macro_rules! binary {
        ($name: ident, $operation: expr) => {
            global_builtin!(runtime, fn $name(call, args) {
                let left = to_u32(number(call, args, 0, "x")?);
                let right = to_u32(number(call, args, 1, "y")?);
                Ok(MiraValue::number(($operation)(left, right) as f64))
            });
        };
    }
    binary!(b_and, |a: u32, b: u32| (a & b) as i32);
    binary!(b_or, |a: u32, b: u32| (a | b) as i32);
    binary!(b_xor, |a: u32, b: u32| (a ^ b) as i32);
    binary!(shl, |a: u32, b: u32| a.wrapping_shl(b & 31) as i32);
    binary!(sar, |a: u32, b: u32| (a as i32) >> (b & 31));
    binary!(shr, |a: u32, b: u32| a >> (b & 31));
    global_builtin!(runtime, fn b_not(call, args) {
        Ok(MiraValue::number(
            (!to_u32(number(call, args, 0, "x")?) as i32) as f64,
        ))
    });
}

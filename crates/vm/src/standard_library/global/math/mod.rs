mod arr;
mod constants;
mod tgamma;
mod to_int;
mod unary;

use crate::standard_library::{global_builtin, number};
use crate::{MiraValue, Runtime};

pub(super) fn install(context: &mut Runtime) {
    constants::install(context);
    unary::install(context);
    global_builtin!(context, fn atan2(call, args) {
        Ok(MiraValue::number(
            number(call, args, 0, "x")?.atan2(number(call, args, 1, "y")?),
        ))
    });
    global_builtin!(context, fn pow(call, args) {
        Ok(MiraValue::number(
            number(call, args, 0, "x")?.powf(number(call, args, 1, "y")?),
        ))
    });
    global_builtin!(context, fn random(call, _args) {
        Ok(MiraValue::number(call.options().providers.random()))
    });
    to_int::install(context);
    arr::install(context);
    tgamma::install(context);
}

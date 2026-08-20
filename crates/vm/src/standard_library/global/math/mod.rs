mod arr;
mod constants;
mod tgamma;
mod to_int;
mod unary;

use crate::standard_library::{insert_native, number};
use crate::{MiraValue, Runtime};

pub(super) fn install(context: &mut Runtime) {
    constants::install(context);
    unary::install(context);
    insert_native(context, "atan2", |call, args| {
        Ok(MiraValue::Number(
            number(call, args, 0, "x")?.atan2(number(call, args, 1, "y")?),
        ))
    });
    insert_native(context, "pow", |call, args| {
        Ok(MiraValue::Number(
            number(call, args, 0, "x")?.powf(number(call, args, 1, "y")?),
        ))
    });
    insert_native(context, "random", |call, _| {
        Ok(MiraValue::Number(call.options().providers.random()))
    });
    to_int::install(context);
    arr::install(context);
    tgamma::install(context);
}

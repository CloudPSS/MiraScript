mod arr;
mod constants;
mod tgamma;
mod to_int;
mod unary;

use crate::standard_library::{insert_native, number};
use crate::{MiraAny, MiraContext};

pub(super) fn install(context: &mut MiraContext) {
    constants::install(context);
    unary::install(context);
    insert_native(context, "atan2", |_, args| {
        Ok(MiraAny::Number(
            number(args, 0, "x")?.atan2(number(args, 1, "y")?),
        ))
    });
    insert_native(context, "pow", |_, args| {
        Ok(MiraAny::Number(
            number(args, 0, "x")?.powf(number(args, 1, "y")?),
        ))
    });
    insert_native(context, "random", |call, _| {
        Ok(MiraAny::Number(call.options().providers.random()))
    });
    to_int::install(context);
    arr::install(context);
    tgamma::install(context);
}

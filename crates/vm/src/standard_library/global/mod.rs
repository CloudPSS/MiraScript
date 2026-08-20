mod bit;
mod debug;
mod json;
mod math;
mod sequence;
mod string;
mod time;
mod to_primitive;

use crate::Runtime;

pub(super) fn install(context: &mut Runtime) {
    math::install(context);
    bit::install(context);
    sequence::install(context);
    to_primitive::install(context);
    string::install(context);
    debug::install(context);
    json::install(context);
    time::install(context);
}

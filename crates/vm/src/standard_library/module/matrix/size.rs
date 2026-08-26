use crate::standard_library::required;
use crate::{MiraValue, Result, Runtime};

use super::helpers::shape;

pub(in crate::standard_library::module) fn size(
    call: &mut Runtime,
    args: &[MiraValue],
) -> Result<MiraValue> {
    let values = shape(call, *required(args, 0, "matrix")?)?
        .into_iter()
        .map(|value| MiraValue::number(value as f64))
        .collect::<Vec<_>>();
    call.insert(values)
}

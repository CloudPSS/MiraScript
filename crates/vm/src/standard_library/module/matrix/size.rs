use crate::standard_library::required;
use crate::{MiraAny, MiraCallContext, Result};

use super::helpers::shape;

pub(super) fn size(_call: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
    Ok(MiraAny::Array(
        shape(required(args, 0, "matrix")?)?
            .into_iter()
            .map(|value| MiraAny::Number(value as f64))
            .collect(),
    ))
}

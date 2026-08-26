use std::f64::consts;

use crate::{Runtime, global_builtin};

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(
        context,
        let PI = consts::PI;
        let pi = consts::PI;
        let E = consts::E;
        let e = consts::E;
        let SQRT1_2 = consts::FRAC_1_SQRT_2;
        let SQRT2 = consts::SQRT_2;
        let LN2 = consts::LN_2;
        let LN10 = consts::LN_10;
        let LOG2E = consts::LOG2_E;
        let LOG10E = consts::LOG10_E;
    );
}

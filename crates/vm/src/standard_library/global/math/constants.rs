use std::f64::consts;

use crate::Runtime;

pub(super) fn install(context: &mut Runtime) {
    for (name, value) in [
        ("PI", consts::PI),
        ("E", consts::E),
        ("pi", consts::PI),
        ("e", consts::E),
        ("SQRT1_2", consts::FRAC_1_SQRT_2),
        ("SQRT2", consts::SQRT_2),
        ("LN2", consts::LN_2),
        ("LN10", consts::LN_10),
        ("LOG2E", consts::LOG2_E),
        ("LOG10E", consts::LOG10_E),
    ] {
        context
            .insert_global(name, value)
            .expect("numeric standard-library globals are inline");
    }
}

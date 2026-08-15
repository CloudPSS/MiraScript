use crate::standard_library::{insert_native, number};
use crate::{MiraAny, MiraContext};

pub(super) fn install(context: &mut MiraContext) {
    macro_rules! unary {
        ($name:literal, $operation:expr) => {
            insert_native(context, $name, |_, args| {
                Ok(MiraAny::Number(($operation)(number(args, 0, "x")?)))
            });
        };
    }

    unary!("sign", |value: f64| {
        if value.is_nan() || value == 0.0 {
            value
        } else {
            value.signum()
        }
    });
    unary!("abs", f64::abs);
    unary!("acos", f64::acos);
    unary!("acosh", f64::acosh);
    unary!("asin", f64::asin);
    unary!("asinh", f64::asinh);
    unary!("atan", f64::atan);
    unary!("atanh", f64::atanh);
    unary!("cos", f64::cos);
    unary!("cosh", f64::cosh);
    unary!("sin", f64::sin);
    unary!("sinh", f64::sinh);
    unary!("tan", f64::tan);
    unary!("tanh", f64::tanh);
    unary!("exp", f64::exp);
    unary!("expm1", f64::exp_m1);
    unary!("log", f64::ln);
    unary!("log10", f64::log10);
    unary!("log1p", f64::ln_1p);
    unary!("log2", f64::log2);
    unary!("sqrt", f64::sqrt);
    unary!("cbrt", f64::cbrt);
}

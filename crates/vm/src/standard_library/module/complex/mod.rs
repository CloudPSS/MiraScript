mod basic;
mod math;
mod utils;

use crate::mira;

use basic::*;
use math::*;
use utils::*;

#[mira]
pub(crate) mod complex {
    use crate::standard_library::number;
    use crate::{MiraValue, Result, Runtime};

    use super as i;

    #[mira]
    const I: (f64, f64) = (0.0, 1.0);

    #[mira]
    fn from(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, |z| z)
    }
    #[mira]
    fn neg(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::neg)
    }

    #[mira]
    fn conj(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::conj)
    }

    #[mira]
    fn sqrt(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::sqrt)
    }

    #[mira]
    fn cbrt(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::cbrt)
    }

    #[mira]
    fn exp(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::exp)
    }

    #[mira]
    fn expm1(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::expm1)
    }

    #[mira]
    fn log(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::log)
    }

    #[mira]
    fn log1p(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::log1p)
    }

    #[mira]
    fn sin(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::sin)
    }

    #[mira]
    fn cos(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::cos)
    }

    #[mira]
    fn tan(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::tan)
    }

    #[mira]
    fn sinh(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::sinh)
    }

    #[mira]
    fn cosh(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::cosh)
    }

    #[mira]
    fn tanh(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::tanh)
    }

    #[mira]
    fn asin(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::asin)
    }

    #[mira]
    fn acos(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::acos)
    }

    #[mira]
    fn atan(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::atan)
    }

    #[mira]
    fn asinh(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::asinh)
    }

    #[mira]
    fn acosh(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::acosh)
    }

    #[mira]
    fn atanh(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, i::atanh)
    }

    #[mira]
    fn add(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::binary(runtime, args, i::add)
    }

    #[mira]
    fn subtract(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::binary(runtime, args, i::subtract)
    }

    #[mira]
    fn multiply(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::binary(runtime, args, i::multiply)
    }

    #[mira]
    fn divide(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::binary(runtime, args, i::divide)
    }

    #[mira]
    fn pow(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::binary(runtime, args, i::pow)
    }

    #[mira]
    fn log2(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, |z| i::log_base(z, std::f64::consts::LN_2))
    }

    #[mira]
    fn log10(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::unary(runtime, args, |z| i::log_base(z, std::f64::consts::LN_10))
    }

    #[mira]
    fn real(runtime: &mut Runtime, args: &[MiraValue]) -> Result<f64> {
        let z = i::parse(runtime, args, 0, "z")?;
        Ok(z.0)
    }

    #[mira]
    fn imag(runtime: &mut Runtime, args: &[MiraValue]) -> Result<f64> {
        let z = i::parse(runtime, args, 0, "z")?;
        Ok(z.1)
    }

    #[mira]
    fn abs(runtime: &mut Runtime, args: &[MiraValue]) -> Result<f64> {
        let z = i::parse(runtime, args, 0, "z")?;
        Ok(z.0.hypot(z.1))
    }

    #[mira]
    fn arg(runtime: &mut Runtime, args: &[MiraValue]) -> Result<f64> {
        let z = i::parse(runtime, args, 0, "z")?;
        Ok(z.1.atan2(z.0))
    }

    #[mira]
    fn polar(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        let r = number(runtime, args, 0, "r")?;
        let theta = number(runtime, args, 1, "theta")?;
        runtime.insert(i::polar(r, theta))
    }
}

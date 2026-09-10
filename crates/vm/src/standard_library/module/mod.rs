use crate::mira;

#[path = "matrix/mod.rs"]
mod matrix_impl;
#[mira]
pub(super) mod matrix {
    use crate::standard_library::{callable, const_value, required};
    use crate::{MiraValue, Result, Runtime};

    use super::matrix_impl as i;

    #[mira]
    fn zeros(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::filled::<0>(runtime, args)
    }

    #[mira]
    fn ones(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::filled::<1>(runtime, args)
    }

    #[mira]
    fn identity(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::identity(runtime, args)
    }

    #[mira]
    fn diagonal(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::diagonal(runtime, args)
    }

    #[mira]
    fn size(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::size(runtime, args)
    }

    #[mira]
    fn transpose(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::transpose(runtime, args)
    }

    #[mira]
    fn invert(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::invert(runtime, args)
    }

    #[mira]
    fn add(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a + b)
    }

    #[mira]
    fn subtract(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a - b)
    }

    #[mira]
    fn entrywise_multiply(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a * b)
    }

    #[mira]
    fn entrywise_divide(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a / b)
    }

    #[mira]
    fn multiply(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::multiply(runtime, args)
    }

    #[mira]
    fn entrywise(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        let left = required(args, 0, "a")?;
        let right = required(args, 1, "b")?;
        let function = callable(args, 2, "f")?;
        i::entrywise(runtime, *left, *right, &mut |runtime, a, b| {
            runtime.checkpoint()?;
            const_value(function.call(runtime, &[a, b])?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;

    #[test]
    fn generated_matrix_names_are_distinct_from_export_keys() {
        let mut runtime = Runtime::new();
        let module = runtime.get_global("matrix").unwrap().as_module().unwrap();
        let module = runtime.get_module_dyn(module).unwrap();
        assert_eq!(module.name(), "matrix");
        assert_eq!(module.index_of("add"), Some(7));

        let function = runtime.eval_unchecked("matrix.add");
        let function = runtime
            .get_function_dyn(function.as_function().unwrap())
            .unwrap();
        assert_eq!(function.name(), "matrix.add");
    }
}

#[path = "complex.rs"]
mod complex_impl;

#[mira]
pub(super) mod complex {
    use super::complex_impl as i;
    use crate::standard_library::number;
    use crate::{MiraValue, Result, Runtime};

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

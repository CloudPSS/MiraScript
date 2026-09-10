mod bin_ops;
mod fill;
mod helpers;
mod invert;
mod size;
mod transpose;

use crate::mira;

use bin_ops::{entrywise, map_nested, multiply, numeric_entrywise};
use fill::{diagonal, filled, identity};
use invert::invert;
use size::size;
use transpose::transpose;

#[mira]
pub(crate) mod matrix {
    use crate::standard_library::{callable, const_value, required};
    use crate::{MiraValue, Result, Runtime};

    use super as i;

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

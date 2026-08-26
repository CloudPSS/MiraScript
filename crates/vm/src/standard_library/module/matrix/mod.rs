mod bin_ops;
mod fill;
mod helpers;
mod invert;
mod size;
mod transpose;

use indexmap::IndexMap;

use crate::standard_library::{builtin_fn, const_value, is_callable, required};
use crate::{MiraError, Result, Runtime, RuntimeErrorKind};

use bin_ops::{entrywise, map_nested, multiply, numeric_entrywise};
use fill::{diagonal, filled, identity};
use invert::invert;
use size::size;
use transpose::transpose;

pub(in crate::standard_library) fn install(runtime: &mut Runtime) -> Result<()> {
    let mut values = IndexMap::new();
    values.insert(
        "zeros".into(),
        runtime.insert(builtin_fn("matrix.zeros", filled::<0>))?,
    );
    values.insert(
        "ones".into(),
        runtime.insert(builtin_fn("matrix.ones", filled::<1>))?,
    );
    values.insert(
        "identity".into(),
        runtime.insert(builtin_fn("matrix.identity", identity))?,
    );
    values.insert(
        "diagonal".into(),
        runtime.insert(builtin_fn("matrix.diagonal", diagonal))?,
    );
    values.insert(
        "size".into(),
        runtime.insert(builtin_fn("matrix.size", size))?,
    );
    values.insert(
        "transpose".into(),
        runtime.insert(builtin_fn("matrix.transpose", transpose))?,
    );
    values.insert(
        "invert".into(),
        runtime.insert(builtin_fn("matrix.invert", invert))?,
    );
    values.insert(
        "add".into(),
        runtime.insert(builtin_fn("matrix.add", |call, args| {
            numeric_entrywise(call, args, |a, b| a + b)
        }))?,
    );
    values.insert(
        "subtract".into(),
        runtime.insert(builtin_fn("matrix.subtract", |call, args| {
            numeric_entrywise(call, args, |a, b| a - b)
        }))?,
    );
    values.insert(
        "entrywise_multiply".into(),
        runtime.insert(builtin_fn("matrix.entrywise_multiply", |call, args| {
            numeric_entrywise(call, args, |a, b| a * b)
        }))?,
    );
    values.insert(
        "entrywise_divide".into(),
        runtime.insert(builtin_fn("matrix.entrywise_divide", |call, args| {
            numeric_entrywise(call, args, |a, b| a / b)
        }))?,
    );
    values.insert(
        "multiply".into(),
        runtime.insert(builtin_fn("matrix.multiply", multiply))?,
    );
    values.insert(
        "entrywise".into(),
        runtime.insert(builtin_fn("matrix.entrywise", |call, args| {
            let left = required(args, 0, "a")?;
            let right = required(args, 1, "b")?;
            let function = required(args, 2, "f")?;
            if !is_callable(function)? {
                return Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
                    actual: function.value_type(),
                }));
            }
            entrywise(call, *left, *right, &mut |runtime, a, b| {
                runtime.checkpoint()?;
                const_value(runtime.call(*function, &[a, b])?)
            })
        }))?,
    );
    let m = runtime.insert(crate::value::types::map_module("matrix", values))?;
    runtime.insert_std("matrix", m);
    Ok(())
}

mod bin_ops;
mod fill;
mod helpers;
mod invert;
mod size;
mod transpose;

use indexmap::IndexMap;

use crate::standard_library::{const_value, is_callable, native, required};
use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind};

use bin_ops::{entrywise, map_nested, multiply, numeric_entrywise};
use fill::{diagonal, filled, identity};
use invert::invert;
use size::size;
use transpose::transpose;

pub(in crate::standard_library) fn module(runtime: &mut Runtime) -> Result<MiraValue> {
    let mut values = IndexMap::new();
    values.insert(
        "zeros".into(),
        runtime.insert(native("matrix.zeros", |call, args| filled(call, args, 0.0)))?,
    );
    values.insert(
        "ones".into(),
        runtime.insert(native("matrix.ones", |call, args| filled(call, args, 1.0)))?,
    );
    values.insert(
        "identity".into(),
        runtime.insert(native("matrix.identity", identity))?,
    );
    values.insert(
        "diagonal".into(),
        runtime.insert(native("matrix.diagonal", diagonal))?,
    );
    values.insert("size".into(), runtime.insert(native("matrix.size", size))?);
    values.insert(
        "transpose".into(),
        runtime.insert(native("matrix.transpose", transpose))?,
    );
    values.insert(
        "invert".into(),
        runtime.insert(native("matrix.invert", invert))?,
    );
    values.insert(
        "add".into(),
        runtime.insert(native("matrix.add", |call, args| {
            numeric_entrywise(call, args, |a, b| a + b)
        }))?,
    );
    values.insert(
        "subtract".into(),
        runtime.insert(native("matrix.subtract", |call, args| {
            numeric_entrywise(call, args, |a, b| a - b)
        }))?,
    );
    values.insert(
        "entrywise_multiply".into(),
        runtime.insert(native("matrix.entrywise_multiply", |call, args| {
            numeric_entrywise(call, args, |a, b| a * b)
        }))?,
    );
    values.insert(
        "entrywise_divide".into(),
        runtime.insert(native("matrix.entrywise_divide", |call, args| {
            numeric_entrywise(call, args, |a, b| a / b)
        }))?,
    );
    values.insert(
        "multiply".into(),
        runtime.insert(native("matrix.multiply", multiply))?,
    );
    values.insert(
        "entrywise".into(),
        runtime.insert(native("matrix.entrywise", |call, args| {
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
    runtime.insert(crate::value::types::map_module("matrix", values))
}

mod bin_ops;
mod fill;
mod helpers;
mod invert;
mod size;
mod transpose;

use indexmap::IndexMap;

use crate::standard_library::{const_value, is_callable, native, required};
use crate::{MiraAny, MiraError, MiraModule};

use bin_ops::{entrywise, map_nested, multiply, numeric_entrywise};
use fill::{diagonal, filled, identity};
use invert::invert;
use size::size;
use transpose::transpose;

pub(in crate::standard_library) fn module() -> MiraAny {
    let mut values = IndexMap::new();
    values.insert(
        "zeros".into(),
        native("matrix.zeros", |call, args| filled(call, args, 0.0)),
    );
    values.insert(
        "ones".into(),
        native("matrix.ones", |call, args| filled(call, args, 1.0)),
    );
    values.insert("identity".into(), native("matrix.identity", identity));
    values.insert("diagonal".into(), native("matrix.diagonal", diagonal));
    values.insert("size".into(), native("matrix.size", size));
    values.insert("transpose".into(), native("matrix.transpose", transpose));
    values.insert("invert".into(), native("matrix.invert", invert));
    values.insert(
        "add".into(),
        native("matrix.add", |_, args| {
            numeric_entrywise(args, |a, b| a + b)
        }),
    );
    values.insert(
        "subtract".into(),
        native("matrix.subtract", |_, args| {
            numeric_entrywise(args, |a, b| a - b)
        }),
    );
    values.insert(
        "entrywise_multiply".into(),
        native("matrix.entrywise_multiply", |_, args| {
            numeric_entrywise(args, |a, b| a * b)
        }),
    );
    values.insert(
        "entrywise_divide".into(),
        native("matrix.entrywise_divide", |_, args| {
            numeric_entrywise(args, |a, b| a / b)
        }),
    );
    values.insert("multiply".into(), native("matrix.multiply", multiply));
    values.insert(
        "entrywise".into(),
        native("matrix.entrywise", |call, args| {
            let left = required(args, 0, "a")?;
            let right = required(args, 1, "b")?;
            let function = required(args, 2, "f")?;
            if !is_callable(function)? {
                return Err(MiraError::runtime("Argument `f` is not callable"));
            }
            entrywise(left, right, &mut |a, b| {
                call.checkpoint()?;
                const_value(call.call(function, &[a, b])?)
            })
        }),
    );
    MiraAny::Module(MiraModule::new("matrix", values).into())
}

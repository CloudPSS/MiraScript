mod builtin;
mod global;
mod module;

use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};
use builtin::{builtin_fn, global_builtin};

pub(crate) fn install(runtime: &mut Runtime) {
    global::install(runtime);
    runtime.insert_std("matrix", module::MATRIX);
}

fn required<'a>(args: &'a [MiraValue], index: usize, name: &'static str) -> Result<&'a MiraValue> {
    args.get(index)
        .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingArgument { name }))
}

fn number(runtime: &Runtime, args: &[MiraValue], index: usize, name: &'static str) -> Result<f64> {
    operations::to_number(runtime, *required(args, index, name)?)
}

fn string(
    runtime: &mut Runtime,
    args: &[MiraValue],
    index: usize,
    name: &'static str,
) -> Result<String> {
    operations::to_string(runtime, *required(args, index, name)?)
}

fn array(
    runtime: &mut Runtime,
    args: &[MiraValue],
    index: usize,
    name: &'static str,
) -> Result<Vec<MiraValue>> {
    let value = *required(args, index, name)?;
    let actual = value.value_type();
    operations::iterable_array(runtime, value).map_err(|_| {
        MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array",
            actual,
        })
    })
}

fn is_callable(value: &MiraValue) -> Result<bool> {
    Ok(value.is_function())
}

fn const_value(value: MiraValue) -> Result<MiraValue> {
    Ok(operations::into_element(value))
}

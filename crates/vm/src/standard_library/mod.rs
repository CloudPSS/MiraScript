mod builtin;
mod global;
mod module;

use crate::{
    MiraError, MiraFunctionHandle, MiraValue, Result, Runtime, RuntimeErrorKind, operations,
};
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

fn callable(args: &[MiraValue], index: usize, name: &'static str) -> Result<MiraFunctionHandle> {
    let value = *required(args, index, name)?;
    if let Some(h) = value.as_function() {
        Ok(h)
    } else {
        Err(MiraError::runtime(RuntimeErrorKind::NotCallable {
            actual: value.value_type(),
        }))
    }
}

fn optional_callable(
    args: &[MiraValue],
    index: usize,
    name: &'static str,
) -> Result<Option<MiraFunctionHandle>> {
    let value = args.get(index).unwrap_or_default();
    if value.is_nil() {
        return Ok(None);
    }
    callable(args, index, name).map(Some)
}

fn const_value(value: MiraValue) -> Result<MiraValue> {
    Ok(operations::into_element(value))
}

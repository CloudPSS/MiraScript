mod global;
mod module;

use crate::{MiraError, MiraNativeFn, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

pub(crate) fn install(runtime: &mut Runtime) {
    global::install(runtime);
    let matrix = module::matrix::module(runtime)
        .expect("standard-library module construction must fit in a fresh Runtime arena");
    runtime
        .insert_std("matrix", matrix)
        .expect("standard-library module handle must belong to its Runtime");
}

fn native(
    name: &'static str,
    callback: impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraValue> + 'static,
) -> MiraNativeFn {
    MiraNativeFn::new(name, callback)
}

fn insert_native(
    runtime: &mut Runtime,
    name: &'static str,
    callback: impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraValue> + 'static,
) {
    let display_name = format!("global.{name}");
    runtime
        .insert_std(name, MiraNativeFn::new(display_name, callback))
        .expect("standard-library function allocation must fit in a fresh Runtime arena");
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
    operations::iterable_array(runtime, value).map_err(|_| {
        MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array",
            actual: value.value_type(),
        })
    })
}

fn is_callable(value: &MiraValue) -> Result<bool> {
    match value {
        MiraValue::Function(_) => Ok(true),
        _ => Ok(false),
    }
}

fn const_value(value: MiraValue) -> Result<MiraValue> {
    Ok(operations::into_element(value))
}

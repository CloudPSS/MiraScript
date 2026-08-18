mod global;
mod module;

use crate::{MiraAny, MiraCallContext, MiraContext, MiraError, MiraNativeFn, Result, operations};

pub(crate) fn install(context: &mut MiraContext) {
    global::install(context);
    context.insert("matrix", module::matrix::module());
}

fn native(
    name: &'static str,
    callback: impl for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
) -> MiraAny {
    MiraAny::from(MiraNativeFn::new(name, callback))
}

fn insert_native(
    context: &mut MiraContext,
    name: &'static str,
    callback: impl for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
) {
    let display_name = format!("global.{name}");
    context.insert(
        name,
        MiraAny::from(MiraNativeFn::new(display_name, callback)),
    );
}

fn required<'a>(args: &'a [MiraAny], index: usize, name: &str) -> Result<&'a MiraAny> {
    args.get(index)
        .ok_or_else(|| MiraError::runtime(format!("Argument `{name}` is required")))
}

fn number(args: &[MiraAny], index: usize, name: &str) -> Result<f64> {
    operations::to_number(required(args, index, name)?)
}

fn string(args: &[MiraAny], index: usize, name: &str) -> Result<String> {
    operations::to_string(required(args, index, name)?)
}

fn array(args: &[MiraAny], index: usize, name: &str) -> Result<Vec<MiraAny>> {
    let value = required(args, index, name)?;
    operations::iterable_array(value).map_err(|_| {
        MiraError::runtime(format!(
            "Argument `{name}` is not array: {}",
            operations::display(value)
        ))
    })
}

fn is_callable(value: &MiraAny) -> Result<bool> {
    match value {
        MiraAny::Function(_) => Ok(true),
        _ => Ok(false),
    }
}

fn const_value(value: MiraAny) -> Result<MiraAny> {
    value.into_element()
}

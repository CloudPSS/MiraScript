use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "reverse", |_, args| {
        let mut values = array_value(required(args, 0, "arr")?)?;
        values.reverse();
        Ok(MiraAny::Array(values))
    });
}

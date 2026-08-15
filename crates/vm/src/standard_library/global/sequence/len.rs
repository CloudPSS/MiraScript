use super::*;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "len", |_, args| {
        let value = required(args, 0, "arr")?;
        let Some(length) = value.array_len()? else {
            return Err(MiraError::runtime("Argument `arr` is not an array"));
        };
        Ok(MiraAny::Number(length as f64))
    });
}

use crate::standard_library::{insert_native, string};
use crate::{MiraAny, MiraContext};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "starts_with", |_, args| {
        Ok(MiraAny::Boolean(
            string(args, 0, "str")?.starts_with(&string(args, 1, "search")?),
        ))
    });
    insert_native(context, "ends_with", |_, args| {
        Ok(MiraAny::Boolean(
            string(args, 0, "str")?.ends_with(&string(args, 1, "search")?),
        ))
    });
    insert_native(context, "contains", |_, args| {
        Ok(MiraAny::Boolean(
            string(args, 0, "str")?.contains(&string(args, 1, "search")?),
        ))
    });
}

use crate::standard_library::{insert_native, string};
use crate::{MiraAny, MiraContext};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "to_uppercase", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?.to_uppercase().into(),
        ))
    });
    insert_native(context, "to_lowercase", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?.to_lowercase().into(),
        ))
    });
}

use crate::standard_library::{insert_native, string};
use crate::{MiraAny, MiraContext};

use super::is_javascript_whitespace;

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "trim_start", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?
                .trim_start_matches(is_javascript_whitespace)
                .into(),
        ))
    });
    insert_native(context, "trim_end", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?
                .trim_end_matches(is_javascript_whitespace)
                .into(),
        ))
    });
    insert_native(context, "trim", |_, args| {
        Ok(MiraAny::String(
            string(args, 0, "str")?
                .trim_matches(is_javascript_whitespace)
                .into(),
        ))
    });
}

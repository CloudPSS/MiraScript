use crate::Runtime;
use crate::standard_library::{insert_native, string};

use super::is_javascript_whitespace;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "trim_start", |call, args| {
        let value = string(call, args, 0, "str")?;
        call.insert(
            value
                .trim_start_matches(is_javascript_whitespace)
                .to_owned(),
        )
    });
    insert_native(context, "trim_end", |call, args| {
        let value = string(call, args, 0, "str")?;
        call.insert(value.trim_end_matches(is_javascript_whitespace).to_owned())
    });
    insert_native(context, "trim", |call, args| {
        let value = string(call, args, 0, "str")?;
        call.insert(value.trim_matches(is_javascript_whitespace).to_owned())
    });
}

use crate::Runtime;
use crate::standard_library::{global_builtin, string};

use super::is_javascript_whitespace;

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn trim_start(call, args) {
        let value = string(call, args, 0, "str")?;
        call.insert(
            value
                .trim_start_matches(is_javascript_whitespace)
                .to_owned(),
        )
    });
    global_builtin!(context, fn trim_end(call, args) {
        let value = string(call, args, 0, "str")?;
        call.insert(value.trim_end_matches(is_javascript_whitespace).to_owned())
    });
    global_builtin!(context, fn trim(call, args) {
        let value = string(call, args, 0, "str")?;
        call.insert(value.trim_matches(is_javascript_whitespace).to_owned())
    });
}

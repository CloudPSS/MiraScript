use crate::Runtime;
use crate::standard_library::{global_builtin, string};

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn to_uppercase(call, args) {
        let value = string(call, args, 0, "str")?.to_uppercase();
        call.insert(value)
    });
    global_builtin!(context, fn to_lowercase(call, args) {
        let value = string(call, args, 0, "str")?.to_lowercase();
        call.insert(value)
    });
}

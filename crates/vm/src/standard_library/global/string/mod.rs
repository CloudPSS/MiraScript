mod case;
mod search;
mod trim;

use crate::standard_library::{array, global_builtin, string};
use crate::{Result, Runtime, operations};

pub(super) fn is_javascript_whitespace(value: char) -> bool {
    value.is_whitespace() || value == '\u{feff}'
}

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn chars(call, args) {
        let source = string(call, args, 0, "str")?;
        let values = source
            .chars()
            .map(|value| call.insert(value.to_string()))
            .collect::<Result<Vec<_>>>()?;
        call.insert(values)
    });
    global_builtin!(context, fn replace(call, args) {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        let replacement = match args.get(2) {
            None => String::new(),
            Some(value) => operations::to_string(call, *value)?,
        };
        call.insert(source.replace(&search, &replacement))
    });
    global_builtin!(context, fn split(call, args) {
        let source = string(call, args, 0, "str")?;
        let separator = match args.get(1) {
            None => String::new(),
            Some(value) => operations::to_string(call, *value)?,
        };
        let parts: Vec<_> = if separator.is_empty() {
            source.chars().map(|value| value.to_string()).collect()
        } else {
            source.split(&separator).map(str::to_owned).collect()
        };
        let parts = parts
            .into_iter()
            .map(|value| call.insert(value))
            .collect::<Result<Vec<_>>>()?;
        call.insert(parts)
    });
    global_builtin!(context, fn join(call, args) {
        let values = array(call, args, 0, "arr")?;
        let separator = match args.get(1) {
            None => String::new(),
            Some(value) => operations::to_string(call, *value)?,
        };
        let parts = values
            .iter()
            .map(|value| operations::to_string(call, *value))
            .collect::<Result<Vec<_>>>()?;
        call.insert(parts.join(&separator))
    });
    trim::install(context);
    case::install(context);
    search::install(context);
}

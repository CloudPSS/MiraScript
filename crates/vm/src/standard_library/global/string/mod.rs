mod case;
mod search;
mod trim;

use crate::standard_library::{array, insert_native, string};
use crate::{MiraAny, MiraContext, Result, operations};

pub(super) fn is_javascript_whitespace(value: char) -> bool {
    value.is_whitespace() || value == '\u{feff}'
}

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "chars", |_, args| {
        Ok(MiraAny::Array(
            string(args, 0, "str")?
                .chars()
                .map(|value| MiraAny::String(value.to_string()))
                .collect(),
        ))
    });
    insert_native(context, "replace", |_, args| {
        let source = string(args, 0, "str")?;
        let search = string(args, 1, "search")?;
        let replacement = match args.get(2) {
            None => String::new(),
            Some(value) => operations::to_string(value)?,
        };
        Ok(MiraAny::String(source.replace(&search, &replacement)))
    });
    insert_native(context, "split", |_, args| {
        let source = string(args, 0, "str")?;
        let separator = match args.get(1) {
            None => String::new(),
            Some(value) => operations::to_string(value)?,
        };
        let parts: Vec<_> = if separator.is_empty() {
            source.chars().map(|value| value.to_string()).collect()
        } else {
            source.split(&separator).map(str::to_owned).collect()
        };
        Ok(MiraAny::Array(
            parts.into_iter().map(MiraAny::String).collect(),
        ))
    });
    insert_native(context, "join", |_, args| {
        let values = array(args, 0, "arr")?;
        let separator = match args.get(1) {
            None => String::new(),
            Some(value) => operations::to_string(value)?,
        };
        let parts = values
            .iter()
            .map(operations::to_string)
            .collect::<Result<Vec<_>>>()?;
        Ok(MiraAny::String(parts.join(&separator)))
    });
    trim::install(context);
    case::install(context);
    search::install(context);
}

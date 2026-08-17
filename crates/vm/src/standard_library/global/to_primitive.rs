use crate::standard_library::insert_native;
use crate::{MiraAny, MiraContext, MiraError, operations};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "to_string", |_, args| {
        Ok(MiraAny::String(
            operations::to_string(
                args.first()
                    .ok_or_else(|| MiraError::runtime("Parameter 'data' is required"))?,
            )?
            .into(),
        ))
    });
    insert_native(context, "to_number", |_, args| {
        let value = args
            .first()
            .ok_or_else(|| MiraError::runtime("Parameter 'data' is required"))?;
        match operations::to_number(value) {
            Ok(value) => Ok(MiraAny::Number(value)),
            Err(_) if args.len() > 1 => Ok(args[1].clone()),
            Err(error) => Err(error),
        }
    });
    insert_native(context, "format", |_, args| {
        let value = args
            .first()
            .ok_or_else(|| MiraError::runtime("Parameter 'data' is required"))?;
        if args.len() < 2 {
            return Err(MiraError::runtime("Parameter 'format' is required"));
        }
        let specifier = match args.get(1) {
            Some(MiraAny::Nil) => None,
            Some(value) => Some(operations::to_string(value)?),
            None => unreachable!(),
        };
        Ok(MiraAny::String(
            operations::format_value(value, specifier.as_deref())?.into(),
        ))
    });
}

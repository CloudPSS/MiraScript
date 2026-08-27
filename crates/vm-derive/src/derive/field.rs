use syn::{Data, Error, Field, Fields, LitStr, Result};

pub fn into_fields(input: Data, derive_name: &'static str) -> Result<Fields> {
    match input {
        Data::Struct(data) => Ok(data.fields),
        Data::Enum(value) => Err(Error::new_spanned(
            value.enum_token,
            format!("{} does not support enums", derive_name),
        )),
        Data::Union(value) => Err(Error::new_spanned(
            value.union_token,
            format!("{} does not support unions", derive_name),
        )),
    }
}

#[derive(Default)]
pub struct FieldOptions {
    pub rename: Option<LitStr>,
    pub skip: bool,
}

pub fn field_options(field: &Field, allow_rename: bool) -> Result<FieldOptions> {
    let mut options = FieldOptions::default();
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("mira"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                if options.skip {
                    return Err(meta.error("duplicate `skip` option"));
                }
                options.skip = true;
                Ok(())
            } else if meta.path.is_ident("rename") && allow_rename {
                if options.rename.is_some() {
                    return Err(meta.error("duplicate `rename` option"));
                }
                options.rename = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported Mira field option"))
            }
        })?;
    }
    Ok(options)
}

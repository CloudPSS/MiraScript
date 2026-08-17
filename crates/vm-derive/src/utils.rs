use std::collections::HashMap;

use proc_macro_crate::{FoundCrate, crate_name};
use quote::format_ident;
use syn::{Attribute, Error, Field, Generics, LitStr, Path, Result, Type, parse_quote};

pub struct ContainerOptions {
    pub crate_path: Path,
    pub tag: Option<LitStr>,
}

impl Default for ContainerOptions {
    fn default() -> Self {
        Self {
            crate_path: default_crate_path(),
            tag: None,
        }
    }
}

fn default_crate_path() -> Path {
    for (package, default_name) in [
        ("mirascript", "mirascript"),
        ("mirascript-vm", "mirascript_vm"),
    ] {
        let found = match crate_name(package) {
            Ok(found) => found,
            Err(_) => continue,
        };
        let name = match found {
            FoundCrate::Itself => default_name.to_owned(),
            FoundCrate::Name(name) => name,
        };
        let ident = format_ident!("{name}");
        return parse_quote!(::#ident);
    }
    parse_quote!(::mirascript_vm)
}

pub fn container_options(attrs: &[Attribute], allow_tag: bool) -> Result<ContainerOptions> {
    let mut options = ContainerOptions::default();
    let mut saw_crate = false;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("mira")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                if saw_crate {
                    return Err(meta.error("duplicate `crate` option"));
                }
                saw_crate = true;
                let value = meta.value()?.parse::<LitStr>()?;
                options.crate_path = value.parse()?;
                Ok(())
            } else if meta.path.is_ident("tag") && allow_tag {
                if options.tag.is_some() {
                    return Err(meta.error("duplicate `tag` option"));
                }
                options.tag = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported Mira derive option"))
            }
        })?;
    }
    Ok(options)
}

#[derive(Default)]
pub struct FieldOptions {
    pub rename: Option<LitStr>,
    pub skip: bool,
    pub readonly: bool,
}

pub fn field_options(
    field: &Field,
    allow_rename: bool,
    allow_readonly: bool,
) -> Result<FieldOptions> {
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
            } else if meta.path.is_ident("readonly") && allow_readonly {
                if options.readonly {
                    return Err(meta.error("duplicate `readonly` option"));
                }
                options.readonly = true;
                Ok(())
            } else {
                Err(meta.error("unsupported Mira field option"))
            }
        })?;
    }
    Ok(options)
}

pub fn reject_duplicate_names(names: &[(String, proc_macro2::Span)]) -> Result<()> {
    let mut seen = HashMap::new();
    for (name, span) in names {
        if let Some(previous) = seen.insert(name, span) {
            let mut error = Error::new(*span, format!("duplicate Mira field name `{name}`"));
            error.combine(Error::new(*previous, "first exported with this name here"));
            return Err(error);
        }
    }
    Ok(())
}

pub fn add_read_bounds(
    generics: &mut Generics,
    types: impl IntoIterator<Item = Type>,
    krate: &Path,
) {
    for parameter in generics.type_params_mut() {
        parameter.bounds.push(parse_quote!('static));
    }
    let lifetimes = generics
        .lifetimes()
        .map(|parameter| parameter.lifetime.clone())
        .collect::<Vec<_>>();
    let where_clause = generics.make_where_clause();
    for lifetime in lifetimes {
        where_clause
            .predicates
            .push(parse_quote!(#lifetime: 'static));
    }
    for ty in types {
        where_clause
            .predicates
            .push(parse_quote!(#ty: ::core::clone::Clone + ::core::convert::Into<#krate::MiraAny>));
    }
}

pub fn add_write_bounds(
    generics: &mut Generics,
    types: impl IntoIterator<Item = Type>,
    krate: &Path,
) {
    let where_clause = generics.make_where_clause();
    for ty in types {
        where_clause.predicates.push(parse_quote!(
            #ty: ::core::convert::TryFrom<#krate::MiraAny, Error = ::std::boxed::Box<#krate::MiraError>>
        ));
    }
}

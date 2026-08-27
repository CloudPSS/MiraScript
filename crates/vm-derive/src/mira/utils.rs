use quote::format_ident;
use syn::{Attribute, Ident};

pub fn conditional_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
        .collect()
}

pub fn rust_name(ident: &Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}

pub fn upper_ident(ident: &Ident) -> Ident {
    format_ident!("{}", rust_name(ident).to_uppercase(), span = ident.span())
}

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Generics, Ident, Path, Result, Type, parse_quote};

pub fn reject_duplicate_names(names: &[(&str, &proc_macro2::Span)]) -> Result<()> {
    let mut seen = HashMap::new();
    for &(name, span) in names {
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
            .push(parse_quote!(#ty: #krate::__private::MiraField));
    }
}

pub fn create_getter(
    krate: &Path,
    index: usize,
    field: TokenStream,
    ty: &Type,
    from: Ident,
) -> TokenStream {
    quote! {
        #index => {
            let parent = unsafe { self_handle.upcast::<Self>() };
            ::core::result::Result::Ok(
                <#ty as #krate::__private::MiraField>::#from(
                    &self.#field,
                    parent,
                    |parent: &Self| &parent.#field,
                )
            )
        },
    }
}

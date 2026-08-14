use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

use crate::utils::{add_read_bounds, container_options, field_options, reject_duplicate_names};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let options = container_options(&input.attrs, false)?;
    let krate = options.crate_path;
    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        Data::Enum(value) => {
            return Err(Error::new_spanned(
                value.enum_token,
                "MiraRecord does not support enums",
            ));
        }
        Data::Union(value) => {
            return Err(Error::new_spanned(
                value.union_token,
                "MiraRecord does not support unions",
            ));
        }
    };

    let mut exported = Vec::new();
    match fields {
        Fields::Named(fields) => {
            for field in fields.named {
                let options = field_options(&field, true, false)?;
                if options.skip {
                    continue;
                }
                let field_ident = field.ident.expect("named field");
                let exported_name = options
                    .rename
                    .map(|name| name.value())
                    .unwrap_or_else(|| field_ident.to_string());
                exported.push((field_ident, exported_name, field.ty));
            }
        }
        Fields::Unit => {}
        Fields::Unnamed(fields) => {
            return Err(Error::new_spanned(
                fields,
                "MiraRecord supports named and unit structs; use MiraArray for tuple structs",
            ));
        }
    }

    reject_duplicate_names(
        &exported
            .iter()
            .map(|(ident, name, _)| (name.clone(), ident.span()))
            .collect::<Vec<_>>(),
    )?;

    let mut generics = input.generics;
    add_read_bounds(
        &mut generics,
        exported.iter().map(|(_, _, ty)| ty.clone()),
        &krate,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let keys = exported.iter().map(|(_, name, _)| name);
    let getters = exported.iter().map(|(field, name, _)| {
        quote!(#name => ::core::result::Result::Ok(::core::option::Option::Some(
            ::core::convert::Into::<#krate::MiraAny>::into(self.#field.clone())
        )),)
    });

    Ok(quote! {
        impl #impl_generics #krate::MiraRecord for #ident #ty_generics #where_clause {
            fn keys(&self) -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(::std::string::String::from(#keys)),*]
            }

            fn get(&self, key: &str) -> #krate::Result<::core::option::Option<#krate::MiraAny>> {
                match key {
                    #(#getters)*
                    _ => ::core::result::Result::Ok(::core::option::Option::None),
                }
            }
        }

        impl #impl_generics ::core::convert::From<#ident #ty_generics> for #krate::MiraAny #where_clause {
            fn from(value: #ident #ty_generics) -> Self {
                #krate::MiraAny::from_record(value)
            }
        }

        impl #impl_generics #krate::__private::MiraBridge for #ident #ty_generics #where_clause {
            fn into_mira_shared(value: #krate::MiraShared<Self>) -> #krate::MiraAny {
                #krate::MiraAny::from_record_shared(value)
            }
        }
    })
}

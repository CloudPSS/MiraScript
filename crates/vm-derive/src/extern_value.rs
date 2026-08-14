use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

use crate::utils::{
    add_read_bounds, add_write_bounds, container_options, field_options, reject_duplicate_names,
};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let options = container_options(&input.attrs, true)?;
    let krate = options.crate_path;
    let ident = input.ident;
    let tag = options
        .tag
        .map(|tag| tag.value())
        .unwrap_or_else(|| ident.to_string());

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        Data::Enum(value) => {
            return Err(Error::new_spanned(
                value.enum_token,
                "MiraExtern does not support enums",
            ));
        }
        Data::Union(value) => {
            return Err(Error::new_spanned(
                value.union_token,
                "MiraExtern does not support unions",
            ));
        }
    };

    let Fields::Named(fields) = fields else {
        return Err(Error::new_spanned(
            fields,
            "MiraExtern supports named structs only",
        ));
    };

    let mut exported = Vec::new();
    for field in fields.named {
        let options = field_options(&field, true, true)?;
        if options.skip {
            continue;
        }
        let field_ident = field.ident.expect("named field");
        let exported_name = options
            .rename
            .map(|name| name.value())
            .unwrap_or_else(|| field_ident.to_string());
        exported.push((field_ident, exported_name, field.ty, options.readonly));
    }

    reject_duplicate_names(
        &exported
            .iter()
            .map(|(ident, name, _, _)| (name.clone(), ident.span()))
            .collect::<Vec<_>>(),
    )?;

    let mut generics = input.generics;
    add_read_bounds(
        &mut generics,
        exported.iter().map(|(_, _, ty, _)| ty.clone()),
        &krate,
    );
    add_write_bounds(
        &mut generics,
        exported
            .iter()
            .filter(|(_, _, _, readonly)| !readonly)
            .map(|(_, _, ty, _)| ty.clone()),
        &krate,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let keys = exported.iter().map(|(_, name, _, _)| name);
    let getters = exported.iter().map(|(field, name, _, _)| {
        quote!(#name => ::core::result::Result::Ok(::core::option::Option::Some(
            ::core::convert::Into::<#krate::MiraAny>::into(self.#field.clone())
        )),)
    });
    let setters = exported
        .iter()
        .filter(|(_, _, _, readonly)| !readonly)
        .map(|(field, name, ty, _)| {
            quote! {
                #name => {
                    let converted = <#ty as ::core::convert::TryFrom<#krate::MiraAny>>::try_from(value)
                        .map_err(|error| error.at_path(#name))?;
                    self.#field = converted;
                    ::core::result::Result::Ok(true)
                }
            }
        });

    Ok(quote! {
        impl #impl_generics #krate::MiraExtern for #ident #ty_generics #where_clause {
            fn tag(&self) -> &str {
                #tag
            }

            fn keys(&self) -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(::std::string::String::from(#keys)),*]
            }

            fn get(&self, key: &str) -> #krate::Result<::core::option::Option<#krate::MiraAny>> {
                match key {
                    #(#getters)*
                    _ => ::core::result::Result::Ok(::core::option::Option::None),
                }
            }

            fn set(&mut self, key: &str, value: #krate::MiraAny) -> #krate::Result<bool> {
                match key {
                    #(#setters,)*
                    _ => ::core::result::Result::Ok(false),
                }
            }
        }

        impl #impl_generics ::core::convert::From<#ident #ty_generics> for #krate::MiraAny #where_clause {
            fn from(value: #ident #ty_generics) -> Self {
                #krate::MiraAny::from_extern(value)
            }
        }

        impl #impl_generics #krate::__private::MiraBridge for #ident #ty_generics #where_clause {
            fn into_mira_shared(value: #krate::MiraShared<Self>) -> #krate::MiraAny {
                #krate::MiraAny::from_extern_shared(value)
            }
        }
    })
}

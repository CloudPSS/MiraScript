use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Result, parse_quote, spanned::Spanned};

use crate::{
    container::container_options,
    field::{field_options, into_fields},
    utils::{add_read_bounds, create_getter, impl_common, reject_duplicate_names},
};

enum ExportField {
    Named(syn::Ident),
    Unnamed(syn::Index),
}

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let options = container_options(&input.attrs)?;
    let fields = into_fields(input.data, "MiraRecord")?;

    let mut exported = Vec::new();
    match fields {
        Fields::Named(fields) => {
            for field in fields.named {
                let options = field_options(&field, true)?;
                if options.skip {
                    continue;
                }
                let field_ident = field.ident.expect("named field");
                let exported_name = options
                    .rename
                    .map(|name| name.value())
                    .unwrap_or_else(|| field_ident.to_string());
                exported.push((
                    ExportField::Named(field_ident.clone()),
                    field_ident.span(),
                    exported_name,
                    field.ty,
                ));
            }
        }
        Fields::Unit => {}
        Fields::Unnamed(fields) => {
            for (index, field) in fields.unnamed.into_iter().enumerate() {
                let options = field_options(&field, true)?;
                if options.skip {
                    continue;
                }
                let exported_name = options
                    .rename
                    .map(|name| name.value())
                    .unwrap_or_else(|| index.to_string());
                exported.push((
                    ExportField::Unnamed(syn::Index::from(index)),
                    field.span(),
                    exported_name,
                    field.ty,
                ));
            }
        }
    }

    reject_duplicate_names(
        exported
            .iter()
            .map(|(_, span, name, _)| (name.as_str(), span)),
    )?;

    let krate = options.crate_path;
    let mut generics = input.generics;
    add_read_bounds(
        &mut generics,
        exported.iter().map(|(_, _, _, ty)| ty),
        &krate,
    );
    let key_matches = exported
        .iter()
        .enumerate()
        .map(|(index, (_, _, name, _))| quote!(#name => ::core::option::Option::Some(#index),));
    let index_matches = exported
        .iter()
        .enumerate()
        .map(|(index, (_, _, name, _))| quote!(#index => ::core::result::Result::Ok(#name),));
    let getters = exported
        .iter()
        .enumerate()
        .map(|(index, (field, _, _, ty))| {
            let field = match field {
                ExportField::Named(ident) => quote!(#ident),
                ExportField::Unnamed(index) => quote!(#index),
            };
            create_getter(&krate, index, field, ty, parse_quote!(from_record))
        });

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let len = exported.len();
    let common_impl = impl_common(&ident, &generics, &krate, "record");

    Ok(quote! {
        impl #impl_generics #krate::MiraShapedRecord for #ident #ty_generics #where_clause {
            fn len() -> usize {
                #len
            }

            fn index_of(key: &str) -> ::core::option::Option<usize> {
                match key {
                    #(#key_matches)*
                    _ => ::core::option::Option::None,
                }
            }

            fn key(index: usize) -> #krate::Result<&'static str> {
                match index {
                    #(#index_matches)*
                    _ => ::core::result::Result::Err(#krate::MiraError::runtime(
                        #krate::RuntimeErrorKind::MissingIndexOrField,
                    )),
                }
            }

            fn get_shaped(
                &self,
                self_handle: #krate::MiraHandle<dyn #krate::MiraRecord>,
                _runtime: &#krate::Runtime,
                index: usize,
            ) -> #krate::Result<#krate::MiraManageable> {
                match index {
                    #(#getters)*
                    _ => ::core::result::Result::Err(#krate::MiraError::runtime(
                        #krate::RuntimeErrorKind::MissingIndexOrField,
                    )),
                }
            }
        }

        #common_impl
    })
}

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

use crate::utils::{add_read_bounds, container_options, field_options, reject_duplicate_names};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let options = container_options(&input.attrs)?;
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
                let options = field_options(&field, true)?;
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
    let len = exported.len();
    let key_matches = exported
        .iter()
        .enumerate()
        .map(|(index, (_, name, _))| quote!(#name => ::core::option::Option::Some(#index),));
    let index_matches = exported
        .iter()
        .enumerate()
        .map(|(index, (_, name, _))| quote!(#index => ::core::result::Result::Ok(#name),));
    let getters = exported.iter().enumerate().map(|(index, (field, _, ty))| {
        quote! {
            #index => {
                let parent = unsafe { self_handle.upcast::<Self>() };
                ::core::result::Result::Ok(
                    <#ty as #krate::__private::MiraField>::from_record(
                        &self.#field,
                        parent,
                        |parent: &Self| &parent.#field,
                    )
                )
            },
        }
    });

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

            fn get(
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

        impl #impl_generics ::core::convert::From<#ident #ty_generics>
            for #krate::MiraManageable #where_clause
        {
            fn from(value: #ident #ty_generics) -> Self {
                #krate::MiraManageable::from_record(value)
            }
        }

        impl #impl_generics #krate::__private::MiraField
            for #ident #ty_generics #where_clause
        {
            fn from_record<P: #krate::MiraRecord>(
                &self,
                parent: #krate::MiraHandle<P>,
                getter: fn(&P) -> &Self,
            ) -> #krate::MiraManageable {
                #krate::__private::shaped_record_from_record(parent, getter)
            }

            fn from_array<P: #krate::MiraArray>(
                &self,
                parent: #krate::MiraHandle<P>,
                getter: fn(&P) -> &Self,
            ) -> #krate::MiraManageable {
                #krate::__private::shaped_record_from_array(parent, getter)
            }
        }
    })
}

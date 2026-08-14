use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Index, Result};

use crate::utils::{add_read_bounds, container_options, field_options};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let options = container_options(&input.attrs, false)?;
    let krate = options.crate_path;
    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        Data::Enum(value) => {
            return Err(Error::new_spanned(
                value.enum_token,
                "MiraArray does not support enums",
            ));
        }
        Data::Union(value) => {
            return Err(Error::new_spanned(
                value.union_token,
                "MiraArray does not support unions",
            ));
        }
    };

    let mut exported = Vec::new();
    match fields {
        Fields::Unnamed(fields) => {
            for (index, field) in fields.unnamed.into_iter().enumerate() {
                let options = field_options(&field, false, false)?;
                if !options.skip {
                    exported.push((Index::from(index), field.ty));
                }
            }
        }
        Fields::Unit => {}
        Fields::Named(fields) => {
            return Err(Error::new_spanned(
                fields,
                "MiraArray supports tuple and unit structs only; use MiraRecord for named structs",
            ));
        }
    }

    let mut generics = input.generics;
    add_read_bounds(
        &mut generics,
        exported.iter().map(|(_, ty)| ty.clone()),
        &krate,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let len = exported.len();
    let getters = exported
        .iter()
        .enumerate()
        .map(|(exported_index, (field, _))| {
            quote!(#exported_index => ::core::option::Option::Some(
            ::core::convert::Into::<#krate::MiraAny>::into(self.#field.clone())
        ),)
        });

    Ok(quote! {
        impl #impl_generics #krate::MiraArray for #ident #ty_generics #where_clause {
            fn len(&self) -> usize {
                #len
            }

            fn get(&self, index: usize) -> #krate::Result<::core::option::Option<#krate::MiraAny>> {
                ::core::result::Result::Ok(match index {
                    #(#getters)*
                    _ => ::core::option::Option::None,
                })
            }
        }

        impl #impl_generics ::core::convert::From<#ident #ty_generics> for #krate::MiraAny #where_clause {
            fn from(value: #ident #ty_generics) -> Self {
                #krate::MiraAny::from_array(value)
            }
        }

        impl #impl_generics #krate::__private::MiraBridge for #ident #ty_generics #where_clause {
            fn into_mira_shared(value: #krate::MiraShared<Self>) -> #krate::MiraAny {
                #krate::MiraAny::from_array_shared(value)
            }
        }
    })
}

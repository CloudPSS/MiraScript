use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Fields, Index, Result, parse_quote};

use crate::field::{field_options, into_fields};
use crate::utils::add_read_bounds;
use crate::{container::container_options, utils::create_getter};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let options = container_options(&input.attrs)?;
    let fields = into_fields(input.data, "MiraArray")?;

    let mut exported = Vec::new();
    match fields {
        Fields::Unnamed(fields) => {
            for (index, field) in fields.unnamed.into_iter().enumerate() {
                let options = field_options(&field, false)?;
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

    let krate = options.crate_path;
    let mut generics = input.generics;
    add_read_bounds(
        &mut generics,
        exported.iter().map(|(_, ty)| ty.clone()),
        &krate,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let len = exported.len();
    let getters = exported.iter().enumerate().map(|(index, (field, ty))| {
        create_getter(&krate, index, quote!(#field), ty, parse_quote!(from_array))
    });

    let ident = input.ident;
    Ok(quote! {
        impl #impl_generics #krate::MiraShapedArray for #ident #ty_generics #where_clause {
            fn len() -> usize {
                #len
            }

            fn get_shaped(
                &self,
                self_handle: #krate::MiraHandle<dyn #krate::MiraArray>,
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
                #krate::MiraManageable::from_array(value)
            }
        }

        impl #impl_generics #krate::__private::MiraField
            for #ident #ty_generics #where_clause
        {
            fn from_record<P: #krate::MiraRecord>(
                &self,
                parent: #krate::MiraHandle<P>,
                index: usize,
                getter: #krate::__private::MiraFieldGetter<P, Self>,
            ) -> #krate::MiraManageable {
                #krate::__private::shaped_array_from_record(parent, index, getter)
            }

            fn from_array<P: #krate::MiraArray>(
                &self,
                parent: #krate::MiraHandle<P>,
                index: usize,
                getter: #krate::__private::MiraFieldGetter<P, Self>,
            ) -> #krate::MiraManageable {
                #krate::__private::shaped_array_from_array(parent, index, getter)
            }
        }
    })
}

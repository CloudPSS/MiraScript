use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
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
                    #index,
                    |parent: &Self, index: usize| &parent.#field,
                )
            )
        },
    }
}

pub(crate) fn impl_common(
    ident: &Ident,
    generics: &Generics,
    krate: &Path,
    impl_type: &'static str,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let from_type = Ident::new(&format!("from_{impl_type}"), Span::call_site());
    let shaped_type_from_array =
        Ident::new(&format!("shaped_{impl_type}_from_array"), Span::call_site());
    let shaped_type_from_record = Ident::new(
        &format!("shaped_{impl_type}_from_record"),
        Span::call_site(),
    );

    quote! {
        impl #impl_generics ::core::convert::From<#ident #ty_generics>
            for #krate::MiraManageable #where_clause
        {
            fn from(value: #ident #ty_generics) -> Self {
                #krate::MiraManageable::#from_type(value)
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
                #krate::__private::#shaped_type_from_record(parent, index, getter)
            }

            fn from_array<P: #krate::MiraArray>(
                &self,
                parent: #krate::MiraHandle<P>,
                index: usize,
                getter: #krate::__private::MiraFieldGetter<P, Self>,
            ) -> #krate::MiraManageable {
                #krate::__private::#shaped_type_from_array(parent, index, getter)
            }
        }
    }
}

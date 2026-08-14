use proc_macro::TokenStream;

mod array;
mod extern_value;
mod record;
mod utils;

#[proc_macro_derive(MiraRecord, attributes(mira))]
pub fn derive_record(input: TokenStream) -> TokenStream {
    record::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(MiraArray, attributes(mira))]
pub fn derive_array(input: TokenStream) -> TokenStream {
    array::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(MiraExtern, attributes(mira))]
pub fn derive_extern(input: TokenStream) -> TokenStream {
    extern_value::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

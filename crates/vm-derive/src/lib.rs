//! Derive macros for exposing live Rust values to `mirascript-vm`.

#![warn(missing_docs)]

use proc_macro::TokenStream;

mod array; 
mod record;
mod utils;

#[proc_macro_derive(MiraRecord, attributes(mira))]
/// Derive a read-only MiraScript record view for a named-field struct.
///
/// Fields may use `#[mira(rename = "name")]` or `#[mira(skip)]`. Generic field
/// types receive the conversion bounds required by the generated implementation.
pub fn derive_record(input: TokenStream) -> TokenStream {
    record::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(MiraArray, attributes(mira))]
/// Derive a read-only MiraScript array view for a tuple struct.
///
/// Tuple fields may use `#[mira(skip)]` and retain their relative order.
pub fn derive_array(input: TokenStream) -> TokenStream {
    array::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

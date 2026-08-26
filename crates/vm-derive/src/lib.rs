//! Derive macros for exposing live Rust values to `mirascript-vm`.

#![warn(missing_docs)]

use proc_macro::TokenStream;

mod array;
mod container;
mod field;
mod generic_visiter;
mod mira;
mod record;
mod utils;

#[proc_macro_attribute]
/// Expose a Rust function or inline module as a MiraScript value.
///
/// The generated companion constant defaults to the upper-case Rust item name.
/// Use `const = NAME` to override the Rust constant, `rename = "mira.name"`
/// to override the diagnostic name, and `use = "export"` to override a direct
/// parent module's export key.
pub fn mira(attr: TokenStream, input: TokenStream) -> TokenStream {
    mira::expand(attr.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

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

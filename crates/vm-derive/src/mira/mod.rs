mod expand_const;
mod expand_fn;
mod expand_mod;
mod meta;
mod utils;

use proc_macro2::{Span, TokenStream};
use syn::{Error, Item, Result};

use meta::{Context, Options};

struct Export {
    key: String,
    accessor: TokenStream,
    span: Span,
}

struct Expanded {
    tokens: TokenStream,
    export: Option<Export>,
}

pub fn expand(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let options = Options::parse(attr)?;

    let item = syn::parse2::<Item>(input)?;
    if let Some(use_name) = options.use_name.as_ref() {
        return Err(Error::new_spanned(
            use_name,
            "`use` is only valid on a direct child of a `#[mira]` module",
        ));
    }

    match item {
        Item::Fn(item) => expand_fn::expand(item, options, None).map(|expanded| expanded.tokens),
        Item::Mod(item) => expand_mod::expand(item, options, None).map(|expanded| expanded.tokens),
        Item::Const(item) => Err(Error::new_spanned(
            item,
            "`#[mira]` constants are only valid inside a `#[mira]` module",
        )),
        item => Err(Error::new_spanned(
            item,
            "`#[mira]` supports functions and inline modules",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::expand;
    use proc_macro2::TokenStream;
    use quote::quote;

    #[test]
    fn external_modules_are_rejected_without_rustc_diagnostics() {
        let error = expand(
            TokenStream::new(),
            quote!(
                mod external;
            ),
        )
        .expect_err("external modules must be rejected");

        assert_eq!(error.to_string(), "`#[mira]` requires an inline module");
    }
}

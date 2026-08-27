use quote::{ToTokens, quote};
use syn::{Error, ItemConst, LitStr, Result};

use super::{Context, Expanded, Export, Options, utils::*};

pub fn expand(item: ItemConst, options: Options, _parent: &Context) -> Result<Expanded> {
    if let Some(const_name) = options.const_name {
        return Err(Error::new_spanned(
            const_name,
            "`const` is not valid on a Rust constant",
        ));
    }
    if let Some(rename) = options.rename {
        return Err(Error::new_spanned(
            rename,
            "`rename` is not valid on a Rust constant; use `use` for its export key",
        ));
    }
    if let Some(crate_path) = options.crate_path {
        return Err(Error::new_spanned(
            crate_path,
            "`crate` is inherited from the containing `#[mira]` module",
        ));
    }
    let ident = item.ident.clone();
    let key = options
        .use_name
        .as_ref()
        .map(LitStr::value)
        .unwrap_or_else(|| rust_name(&ident));
    Ok(Expanded {
        tokens: item.into_token_stream(),
        export: Some(Export {
            key,
            accessor: quote!(#ident),
            span: ident.span(),
        }),
    })
}

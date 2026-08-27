use std::collections::HashMap;

use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, Error, Item, ItemConst, ItemMod, LitStr, Result};

use super::{Context, Expanded, Export, Options, expand_fn::expand as expand_fn, utils::*};

pub fn expand(item: ItemMod, options: Options, parent: Option<&Context>) -> Result<Expanded> {
    let ident = &item.ident;
    let rust_name = rust_name(ident);
    let full_name = options
        .rename
        .as_ref()
        .map(LitStr::value)
        .unwrap_or_else(|| {
            parent
                .map(|parent| format!("{}.{}", parent.full_name, rust_name))
                .unwrap_or_else(|| rust_name.clone())
        });
    let export_name = options
        .use_name
        .as_ref()
        .map(LitStr::value)
        .unwrap_or_else(|| rust_name.clone());
    let const_ident = options
        .const_name
        .clone()
        .unwrap_or_else(|| upper_ident(ident));
    let krate = options.crate_path(parent);
    let context = Context {
        full_name: full_name.clone(),
        crate_path: krate.clone(),
    };
    let Some((_, items)) = &item.content else {
        return Err(Error::new_spanned(
            item,
            "`#[mira]` requires an inline module",
        ));
    };

    let mut expanded_items = Vec::with_capacity(items.len());
    let mut exports = Vec::new();
    for child in items.iter().cloned() {
        let expanded = expand_child(child, &context)?;
        if let Some(export) = expanded.export {
            exports.push(export);
        }
        expanded_items.push(expanded.tokens);
    }
    reject_duplicate_exports(&exports)?;

    let attrs = &item.attrs;
    let vis = &item.vis;
    let unsafety = &item.unsafety;
    let cfg = conditional_attrs(attrs);
    let hidden = format_ident!("__MiraModule_{rust_name}", span = ident.span());
    let module_name = LitStr::new(&full_name, ident.span());
    let len = exports.len();
    let key_matches = exports.iter().enumerate().map(|(index, export)| {
        let key = LitStr::new(&export.key, export.span);
        quote!(#key => ::core::option::Option::Some(#index),)
    });
    let index_keys = exports.iter().enumerate().map(|(index, export)| {
        let key = LitStr::new(&export.key, export.span);
        quote!(#index => ::core::result::Result::Ok(#key),)
    });
    let getters = exports.iter().enumerate().map(|(index, export)| {
        let accessor = &export.accessor;
        quote!(#index => ::core::result::Result::Ok(
            ::core::convert::Into::<#krate::MiraManageable>::into(#accessor),
        ),)
    });

    let tokens = quote! {
        #(#attrs)*
        #vis #unsafety mod #ident {
            #(#expanded_items)*

            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct #hidden;

            impl #krate::MiraModule for #hidden {
                fn name(&self) -> &str {
                    #module_name
                }

                fn len(&self) -> usize {
                    #len
                }

                fn index_of(&self, key: &str) -> ::core::option::Option<usize> {
                    match key {
                        #(#key_matches)*
                        _ => ::core::option::Option::None,
                    }
                }

                fn key(&self, index: usize) -> #krate::Result<&str> {
                    match index {
                        #(#index_keys)*
                        _ => ::core::result::Result::Err(#krate::MiraError::runtime(
                            #krate::RuntimeErrorKind::MissingIndexOrField,
                        )),
                    }
                }

                fn get(
                    &self,
                    _self_handle: #krate::MiraHandle<dyn #krate::MiraModule>,
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

            impl ::core::convert::From<#hidden> for #krate::MiraManageable {
                fn from(value: #hidden) -> Self {
                    #krate::MiraManageable::from_module(value)
                }
            }
        }

        #(#cfg)*
        #[doc = concat!("MiraScript module value for [`", stringify!(#ident), "`].")]
        #[allow(non_upper_case_globals)]
        #vis const #const_ident: #ident::#hidden = #ident::#hidden;
    };

    Ok(Expanded {
        tokens,
        export: parent.map(|_| Export {
            key: export_name,
            accessor: quote!(#const_ident),
            span: ident.span(),
        }),
    })
}

fn expand_child(mut item: Item, parent: &Context) -> Result<Expanded> {
    let options = match item_attrs_mut(&mut item) {
        Some(attrs) => Options::parse_from_attrs(attrs)?,
        None => None,
    };
    let Some(options) = options else {
        return expand_unmarked(item);
    };
    if options.skip {
        return Ok(Expanded {
            tokens: item.into_token_stream(),
            export: None,
        });
    }
    match item {
        Item::Fn(item) => expand_fn(item, options, Some(parent)),
        Item::Mod(item) => expand(item, options, Some(parent)),
        Item::Const(item) => expand_constant(item, options, parent),
        item => Err(Error::new_spanned(
            item,
            "a `#[mira]` module can export functions, constants, and inline modules",
        )),
    }
}

fn expand_unmarked(item: Item) -> Result<Expanded> {
    let Item::Mod(mut module) = item else {
        return Ok(Expanded {
            tokens: item.into_token_stream(),
            export: None,
        });
    };
    let Some((_, items)) = module.content.take() else {
        return Err(Error::new_spanned(
            module,
            "file modules cannot appear inside a `#[mira]` module",
        ));
    };
    let attrs = &module.attrs;
    let vis = &module.vis;
    let unsafety = &module.unsafety;
    let ident = &module.ident;
    let mut expanded = Vec::with_capacity(items.len());
    for mut child in items {
        let options = match item_attrs_mut(&mut child) {
            Some(attrs) => Options::parse_from_attrs(attrs)?,
            None => None,
        };
        let tokens = match options {
            Some(options) if options.skip => child.into_token_stream(),
            Some(options) => {
                if let Some(use_name) = options.use_name {
                    return Err(Error::new_spanned(
                        use_name,
                        "`use` is only valid on a direct child of a `#[mira]` module",
                    ));
                }
                match child {
                    Item::Fn(item) => expand_fn(item, options, None)?.tokens,
                    Item::Mod(item) => expand(item, options, None)?.tokens,
                    Item::Const(item) => {
                        return Err(Error::new_spanned(
                            item,
                            "`#[mira]` constants are only valid inside a `#[mira]` module",
                        ));
                    }
                    item => {
                        return Err(Error::new_spanned(
                            item,
                            "`#[mira]` supports functions and inline modules",
                        ));
                    }
                }
            }
            None => expand_unmarked(child)?.tokens,
        };
        expanded.push(tokens);
    }
    Ok(Expanded {
        tokens: quote! {
            #(#attrs)*
            #vis #unsafety mod #ident {
                #(#expanded)*
            }
        },
        export: None,
    })
}

fn expand_constant(item: ItemConst, options: Options, _parent: &Context) -> Result<Expanded> {
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

fn item_attrs_mut(item: &mut Item) -> Option<&mut Vec<Attribute>> {
    match item {
        Item::Const(item) => Some(&mut item.attrs),
        Item::Enum(item) => Some(&mut item.attrs),
        Item::ExternCrate(item) => Some(&mut item.attrs),
        Item::Fn(item) => Some(&mut item.attrs),
        Item::ForeignMod(item) => Some(&mut item.attrs),
        Item::Impl(item) => Some(&mut item.attrs),
        Item::Macro(item) => Some(&mut item.attrs),
        Item::Mod(item) => Some(&mut item.attrs),
        Item::Static(item) => Some(&mut item.attrs),
        Item::Struct(item) => Some(&mut item.attrs),
        Item::Trait(item) => Some(&mut item.attrs),
        Item::TraitAlias(item) => Some(&mut item.attrs),
        Item::Type(item) => Some(&mut item.attrs),
        Item::Union(item) => Some(&mut item.attrs),
        Item::Use(item) => Some(&mut item.attrs),
        Item::Verbatim(_) => None,
        _ => None,
    }
}

fn reject_duplicate_exports(exports: &[Export]) -> Result<()> {
    let mut seen = HashMap::new();
    let mut combined: Option<Error> = None;
    for export in exports {
        if let Some(previous) = seen.insert(export.key.as_str(), export.span) {
            let mut error = Error::new(
                export.span,
                format!("duplicate Mira module export `{}`", export.key),
            );
            error.combine(Error::new(previous, "first exported with this name here"));
            if let Some(combined) = &mut combined {
                combined.combine(error);
            } else {
                combined = Some(error);
            }
        }
    }
    match combined {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

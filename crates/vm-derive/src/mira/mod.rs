mod meta;

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Error, FnArg, Ident, Item, ItemConst, ItemFn, ItemMod, LitStr, Path, PathArguments,
    Result, ReturnType, Type, spanned::Spanned,
};

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
    if options.skip {
        return Ok(item.into_token_stream());
    }
    if let Some(use_name) = options.use_name.as_ref() {
        return Err(Error::new_spanned(
            use_name,
            "`use` is only valid on a direct child of a `#[mira]` module",
        ));
    }

    match item {
        Item::Fn(item) => expand_function(item, options, None).map(|expanded| expanded.tokens),
        Item::Mod(item) => expand_module(item, options, None).map(|expanded| expanded.tokens),
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

fn expand_function(item: ItemFn, options: Options, parent: Option<&Context>) -> Result<Expanded> {
    validate_function(&item)?;
    let ident = &item.sig.ident;
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
    let hidden = format_ident!("__MiraFunction_{}", rust_name, span = ident.span());
    let vis = &item.vis;
    let cfg = conditional_attrs(&item.attrs);
    let call = function_call(&item, &krate)?;
    let name = LitStr::new(&full_name, ident.span());

    let tokens = quote! {
        #item

        #(#cfg)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #hidden;

        #(#cfg)*
        impl #krate::MiraFunction for #hidden {
            fn call(
                &self,
                runtime: &mut #krate::Runtime,
                args: &[#krate::MiraValue],
            ) -> #krate::Result<#krate::MiraManageable> {
                #call
            }

            fn name(&self) -> &str {
                #name
            }
        }

        #(#cfg)*
        impl ::core::convert::From<#hidden> for #krate::MiraManageable {
            fn from(value: #hidden) -> Self {
                #krate::MiraManageable::from_function(value)
            }
        }

        #(#cfg)*
        #[doc = concat!("MiraScript function value for [`", stringify!(#ident), "`].")]
        #[allow(non_upper_case_globals)]
        #vis const #const_ident: #hidden = #hidden;
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

fn validate_function(item: &ItemFn) -> Result<()> {
    let signature = &item.sig;
    if signature.constness.is_some() {
        return Err(Error::new_spanned(
            signature.constness,
            "Mira functions cannot be `const`",
        ));
    }
    if signature.asyncness.is_some() {
        return Err(Error::new_spanned(
            signature.asyncness,
            "Mira functions cannot be `async`",
        ));
    }
    if !matches!(
        signature.safety,
        syn::Safety::Default | syn::Safety::Safe(_)
    ) {
        return Err(Error::new_spanned(
            &signature.safety,
            "Mira functions cannot be `unsafe`",
        ));
    }
    if signature.abi.is_some() {
        return Err(Error::new_spanned(
            &signature.abi,
            "Mira functions cannot use an explicit ABI",
        ));
    }
    if !signature.generics.params.is_empty() || signature.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &signature.generics,
            "Mira functions cannot be generic",
        ));
    }
    if signature.variadic.is_some() {
        return Err(Error::new_spanned(
            &signature.variadic,
            "use a final `&[MiraValue]` parameter for remaining arguments",
        ));
    }
    if signature.receiver().is_some() {
        return Err(Error::new_spanned(
            signature.receiver(),
            "Mira functions must be free functions",
        ));
    }
    Ok(())
}

fn function_call(item: &ItemFn, krate: &Path) -> Result<TokenStream> {
    let mut runtime = false;
    let mut rest = false;
    let mut fixed = Vec::new();

    for (position, argument) in item.sig.inputs.iter().enumerate() {
        let FnArg::Typed(argument) = argument else {
            return Err(Error::new_spanned(
                argument,
                "Mira functions cannot have a receiver",
            ));
        };
        if is_runtime(&argument.ty) {
            if position != 0 {
                return Err(Error::new_spanned(
                    argument,
                    "`&mut Runtime` must be the first parameter",
                ));
            }
            runtime = true;
            continue;
        }
        if is_rest(&argument.ty) {
            if position + 1 != item.sig.inputs.len() {
                return Err(Error::new_spanned(
                    argument,
                    "`&[MiraValue]` must be the final parameter",
                ));
            }
            rest = true;
            continue;
        }
        fixed.push(argument);
    }

    let conversions = fixed.iter().enumerate().map(|(index, argument)| {
        let variable = format_ident!("__mira_arg_{index}");
        let ty = &argument.ty;
        let argument_name = LitStr::new(
            &argument.pat.to_token_stream().to_string(),
            argument.pat.span(),
        );
        quote! {
            let #variable: #ty = #krate::__private::native_argument(
                *args.get(#index).ok_or_else(|| #krate::MiraError::runtime(
                    #krate::RuntimeErrorKind::MissingArgument { name: #argument_name },
                ))?,
            )?;
        }
    });
    let fixed_variables = (0..fixed.len()).map(|index| format_ident!("__mira_arg_{index}"));
    let mut arguments = Vec::new();
    if runtime {
        arguments.push(quote!(runtime));
    }
    arguments.extend(fixed_variables.map(|variable| quote!(#variable)));
    if rest {
        let fixed_len = fixed.len();
        arguments.push(quote!(&args[#fixed_len..]));
    }
    let ident = &item.sig.ident;
    let invocation = quote!(#ident(#(#arguments),*));
    let result = if returns_result(&item.sig.output) {
        quote!(#krate::__private::native_result(#invocation))
    } else {
        quote!(::core::result::Result::Ok(
            ::core::convert::Into::<#krate::MiraManageable>::into(#invocation),
        ))
    };

    Ok(quote! {
        #(#conversions)*
        #result
    })
}

fn expand_module(item: ItemMod, options: Options, parent: Option<&Context>) -> Result<Expanded> {
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
    let hidden = format_ident!("__MiraModule_{}", rust_name, span = ident.span());
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
        Item::Fn(item) => expand_function(item, options, Some(parent)),
        Item::Mod(item) => expand_module(item, options, Some(parent)),
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
                    Item::Fn(item) => expand_function(item, options, None)?.tokens,
                    Item::Mod(item) => expand_module(item, options, None)?.tokens,
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

fn conditional_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
        .collect()
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

fn rust_name(ident: &Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}

fn upper_ident(ident: &Ident) -> Ident {
    format_ident!("{}", rust_name(ident).to_uppercase(), span = ident.span())
}

fn is_runtime(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };
    reference.mutability.is_some() && is_path_name(&reference.elem, "Runtime")
}

fn is_rest(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };
    if reference.mutability.is_some() {
        return false;
    }
    let Type::Slice(slice) = reference.elem.as_ref() else {
        return false;
    };
    is_path_name(&slice.elem, "MiraValue")
}

fn is_path_name(ty: &Type, name: &str) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == name && segment.arguments.is_empty())
}

fn returns_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    let Type::Path(path) = ty.as_ref() else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.last().is_some_and(|segment| {
            segment.ident == "Result"
                && matches!(segment.arguments, PathArguments::AngleBracketed(_))
        })
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

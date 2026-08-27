use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Error, FnArg, ItemFn, LitStr, Path, PathArguments, Result, ReturnType, Type, spanned::Spanned,
};

use super::{Context, Expanded, Export, Options, utils::*};

pub fn expand(item: ItemFn, options: Options, parent: Option<&Context>) -> Result<Expanded> {
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
    let hidden = format_ident!("__MiraFunction_{rust_name}", span = ident.span());
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
        let optional = is_optional(ty);
        let argument_name = LitStr::new(
            &argument.pat.to_token_stream().to_string(),
            argument.pat.span(),
        );
        if optional {
            quote! {
                let #variable: #ty = #krate::__private::native_argument_optional(
                    runtime,
                    args.get(#index).copied(),
                )?;
            }
        } else {
            quote! {
                let #variable: #ty = #krate::__private::native_argument(
                    runtime,
                    *args.get(#index).ok_or_else(|| #krate::MiraError::runtime(
                        #krate::RuntimeErrorKind::MissingArgument { name: #argument_name },
                    ))?,
                )?;
            }
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

fn is_optional(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.last().is_some_and(|segment| {
            segment.ident == "Option"
                && matches!(segment.arguments, PathArguments::AngleBracketed(_))
        })
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

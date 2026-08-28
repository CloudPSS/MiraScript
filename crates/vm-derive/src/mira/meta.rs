use proc_macro2::TokenStream;
use syn::{Attribute, Error, Ident, LitStr, Meta, Path, Result, parse::Parser};

use crate::utils::default_crate_path;

#[derive(Clone)]
pub(crate) struct Context {
    pub full_name: String,
    pub crate_path: Path,
}

#[derive(Default)]
pub(crate) struct Options {
    pub const_name: Option<Ident>,
    pub rename: Option<LitStr>,
    pub use_name: Option<LitStr>,
    pub crate_path: Option<Path>,
}

impl Options {
    pub fn crate_path(&self, parent: Option<&Context>) -> Path {
        self.crate_path
            .clone()
            .or_else(|| parent.map(|parent| parent.crate_path.clone()))
            .unwrap_or_else(default_crate_path)
    }
}

fn is_mira_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "mira")
}

fn parse_field<T: syn::parse::Parse>(
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
    field: &mut Option<T>,
) -> Result<bool> {
    if !meta.path.is_ident(name) {
        return Ok(false);
    }
    if field.is_some() {
        return Err(meta.error(format!("duplicate `{name}` option")));
    }
    *field = Some(meta.value()?.parse()?);
    Ok(true)
}

impl Options {
    fn parse_meta(&mut self, meta: syn::meta::ParseNestedMeta<'_>) -> Result<()> {
        if parse_field(&meta, "const", &mut self.const_name)? {
            return Ok(());
        }
        if parse_field(&meta, "rename", &mut self.rename)? {
            return Ok(());
        }
        if parse_field(&meta, "use", &mut self.use_name)? {
            return Ok(());
        }
        if parse_field(&meta, "crate", &mut self.crate_path)? {
            return Ok(());
        }
        Err(meta.error("unsupported `mira` option"))
    }

    pub fn parse(attr: TokenStream) -> Result<Self> {
        let mut options = Self::default();
        syn::meta::parser(|meta| options.parse_meta(meta)).parse2(attr)?;
        Ok(options)
    }

    pub fn parse_from_attrs(attrs: &mut Vec<Attribute>) -> Result<Option<Self>> {
        let mut options: Option<Self> = None;
        let mut retained = Vec::with_capacity(attrs.len());
        for attr in attrs.drain(..) {
            if is_mira_attr(&attr) {
                if options.is_some() {
                    return Err(Error::new_spanned(attr, "duplicate `mira` attribute"));
                }
                let mut opt = Options::default();
                match &attr.meta {
                    Meta::Path(_) => {}
                    Meta::List(_) => {
                        attr.parse_nested_meta(|meta| opt.parse_meta(meta))?;
                    }
                    Meta::NameValue(_) => {
                        return Err(Error::new_spanned(attr, "expected `#[mira(...)]`"));
                    }
                }
                options = Some(opt);
            } else {
                retained.push(attr);
            }
        }
        *attrs = retained;
        Ok(options)
    }
}

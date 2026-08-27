use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Error, Ident, LitStr, Meta, Path, Result, parse::Parser};

use crate::utils::default_crate_path;

fn is_mira_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "mira")
}

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
    pub skip: bool,
}

impl Options {
    pub fn crate_path(&self, parent: Option<&Context>) -> Path {
        self.crate_path
            .clone()
            .or_else(|| parent.map(|parent| parent.crate_path.clone()))
            .unwrap_or_else(default_crate_path)
    }
}

impl Options {
    fn parse_meta(&mut self, meta: syn::meta::ParseNestedMeta<'_>) -> Result<()> {
        if meta.path.is_ident("const") {
            if self.const_name.is_some() {
                return Err(meta.error("duplicate `const` option"));
            }
            self.const_name = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("rename") {
            if self.rename.is_some() {
                return Err(meta.error("duplicate `rename` option"));
            }
            self.rename = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("use") {
            if self.use_name.is_some() {
                return Err(meta.error("duplicate `use` option"));
            }
            self.use_name = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("crate") {
            if self.crate_path.is_some() {
                return Err(meta.error("duplicate `crate` option"));
            }
            self.crate_path = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("skip") {
            if self.skip {
                return Err(meta.error("duplicate `skip` option"));
            }
            self.skip = true;
            Ok(())
        } else {
            Err(meta.error("unsupported `mira` option"))
        }
    }

    fn validate(&self) -> Result<()> {
        if self.skip
            && (self.const_name.is_some()
                || self.rename.is_some()
                || self.use_name.is_some()
                || self.crate_path.is_some())
        {
            return Err(Error::new(
                Span::call_site(),
                "`skip` cannot be combined with another `mira` option",
            ));
        }
        Ok(())
    }

    pub fn parse(attr: TokenStream) -> Result<Self> {
        let mut options = Self::default();
        syn::meta::parser(|meta| options.parse_meta(meta)).parse2(attr)?;
        options.validate()?;
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
                        opt.validate()?;
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

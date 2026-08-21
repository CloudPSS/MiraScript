use proc_macro_crate::{FoundCrate, crate_name};
use quote::format_ident;
use syn::{Attribute, Path, Result, parse_quote};

pub struct ContainerOptions {
    pub crate_path: Path,
}

impl Default for ContainerOptions {
    fn default() -> Self {
        Self {
            crate_path: default_crate_path(),
        }
    }
}

fn default_crate_path() -> Path {
    for (package, default_name) in [
        ("mirascript", "mirascript"),
        ("mirascript-vm", "mirascript_vm"),
    ] {
        let found = match crate_name(package) {
            Ok(found) => found,
            Err(_) => continue,
        };
        let name = match found {
            FoundCrate::Itself => default_name.to_owned(),
            FoundCrate::Name(name) => name,
        };
        let ident = format_ident!("{name}");
        return parse_quote!(::#ident);
    }
    parse_quote!(::mirascript_vm)
}

pub fn container_options(attrs: &[Attribute]) -> Result<ContainerOptions> {
    let mut options = ContainerOptions::default();
    let mut saw_crate = false;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("mira")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                if saw_crate {
                    return Err(meta.error("duplicate `crate` option"));
                }
                saw_crate = true;
                let value = meta.value()?.parse::<Path>()?;
                options.crate_path = value;
                Ok(())
            } else {
                Err(meta.error("unsupported Mira derive option"))
            }
        })?;
    }
    Ok(options)
}

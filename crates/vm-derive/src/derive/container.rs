use syn::{Attribute, Path, Result};

use crate::utils::default_crate_path;

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

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use syn::{Ident, Path, parse_quote};

pub(crate) fn default_crate_path() -> Path {
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
        let ident = Ident::new(&name, Span::call_site());
        return parse_quote!(::#ident);
    }
    parse_quote!(::mirascript_vm)
}

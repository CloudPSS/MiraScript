use syn::{
    ExprPath, GenericParam, Generics, Ident, Lifetime, Type, TypePath,
    visit::{self, Visit},
};

struct GenericParameterVisitor<'a> {
    generics: &'a Generics,
    found: bool,
}

impl GenericParameterVisitor<'_> {
    fn is_generic(&self, ident: &Ident) -> bool {
        self.generics
            .params
            .iter()
            .any(|parameter| match parameter {
                GenericParam::Lifetime(parameter) => &parameter.lifetime.ident == ident,
                GenericParam::Type(parameter) => &parameter.ident == ident,
                GenericParam::Const(parameter) => &parameter.ident == ident,
            })
    }
}

pub(crate) fn type_uses_generic_parameter(generics: &Generics, ty: &Type) -> bool {
    let mut visitor = GenericParameterVisitor {
        generics,
        found: false,
    };
    visitor.visit_type(ty);
    visitor.found
}

impl<'ast> Visit<'ast> for GenericParameterVisitor<'_> {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        if path.qself.is_none()
            && path
                .path
                .segments
                .first()
                .is_some_and(|segment| self.is_generic(&segment.ident))
        {
            self.found = true;
            return;
        }
        visit::visit_type_path(self, path);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
        if self.is_generic(&lifetime.ident) {
            self.found = true;
        }
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        if path.qself.is_none()
            && path
                .path
                .segments
                .first()
                .is_some_and(|segment| self.is_generic(&segment.ident))
        {
            self.found = true;
            return;
        }
        visit::visit_expr_path(self, path);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn detects_generic_parameters_used_by_field_types() {
        let generics: Generics = parse_quote!(<'a, T, const N: usize>);

        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!(String)
        ));
        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!(Box<[Item]>)
        ));
        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!(Box<[Vec<Item>]>)
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!(Vec<T>)
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!(Vec<[T; 12]>)
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!(Vec<[String; N]>)
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!(&'a str)
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!([u8; N + 1])
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!([u8; T::LEN])
        ));
        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!([u8; Item::LEN])
        ));
        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!([u8; 12])
        ));
        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!([u8; 1 + 2])
        ));
        assert!(type_uses_generic_parameter(
            &generics,
            &parse_quote!(<T as Iterator>::Item)
        ));
        assert!(!type_uses_generic_parameter(
            &generics,
            &parse_quote!(<Item as Iterator>::Item)
        ));
    }
}

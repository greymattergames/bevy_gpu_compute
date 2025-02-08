use syn::{ItemStruct, parse_quote, visit_mut::VisitMut};
const INTERNAL_ATTRIBUTE_NAMES: [&str; 4] = [
    "wgsl_config",
    "wgsl_input_array",
    "wgsl_output_array",
    "wgsl_output_vec",
];

pub fn remove_internal_attributes(input: &mut syn::ItemMod) {
    let mut transformer = InternalAttributesRemover;
    transformer.visit_item_mod_mut(input);
}
struct InternalAttributesRemover;

impl VisitMut for InternalAttributesRemover {
    fn visit_item_mut(&mut self, i: &mut syn::Item) {
        syn::visit_mut::visit_item_mut(self, i);
        match i {
            syn::Item::Struct(item_struct) => {
                item_struct.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Fn(item_fn) => {
                item_fn.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Enum(item_enum) => {
                item_enum.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Type(item_type) => {
                item_type.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Const(item_const) => {
                item_const.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Trait(item_trait) => {
                item_trait.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Impl(item_impl) => {
                item_impl.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }
            syn::Item::Mod(item_mod) => {
                item_mod.attrs.retain(|attr| {
                    !INTERNAL_ATTRIBUTE_NAMES
                        .iter()
                        .any(|name| attr.path().is_ident(name))
                });
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    #[test]
    fn test_remove_internal_attributes() {
        let mut input: syn::ItemMod = parse_quote! {
            pub mod m{
        #[wgsl_config]
        #[wgsl_input_array]
        #[wgsl_output_array]
        #[wgsl_output_vec]
        #[valid]
        #[wgsl_config]
        #[wgsl_input_array]
        #[wgsl_output_array]
        #[wgsl_output_vec]
        struct Something {}
            }
        };
        let expected: syn::ItemMod = parse_quote! {
            pub mod m{
                #[valid]
        struct Something {}
            }
        };
        let mut transformer = InternalAttributesRemover;
        transformer.visit_item_mod_mut(&mut input);
        assert_eq!(
            input.to_token_stream().to_string(),
            expected.to_token_stream().to_string()
        );
    }
}

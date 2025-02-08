// find all user declared types, and make a list of them

// ItemStruct.ident or  ItemType.ident

use quote::ToTokens;
use syn::visit::Visit;

use super::custom_type::{CustomType, CustomTypeKind};

struct CustomTypesCollector {
    custom_types: Vec<CustomType>,
}

impl<'ast> Visit<'ast> for CustomTypesCollector {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        syn::visit::visit_item_struct(self, i);

        self.custom_types.push(CustomType::new(
            &i.ident,
            CustomTypeKind::from(&i.attrs),
            i.to_token_stream(),
        ));
    }

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        syn::visit::visit_item_type(self, i);
        self.custom_types.push(CustomType::new(
            &i.ident,
            CustomTypeKind::from(&i.attrs),
            i.to_token_stream(),
        ));
    }
}

impl CustomTypesCollector {
    pub fn new() -> Self {
        CustomTypesCollector {
            custom_types: vec![],
        }
    }
}

pub fn collect_custom_types(original_rust_module: &syn::ItemMod) -> Vec<CustomType> {
    let mut types_collector = CustomTypesCollector::new();
    types_collector.visit_item_mod(&original_rust_module);
    types_collector.custom_types
}

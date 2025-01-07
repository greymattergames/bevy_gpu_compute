// find all user declared types, and make a list of them

// ItemStruct.ident or  ItemType.ident

use syn::{
    Ident, Item, ItemMod,
    visit::{self, Visit},
};

use crate::transformer::allowed_types::AllowedRustTypes;

use super::custom_type::CustomTypeKind;

struct CustomTypesLister {
    allowed_types: AllowedRustTypes,
}

impl<'ast> Visit<'ast> for CustomTypesLister {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        syn::visit::visit_item_struct(self, i);

        self.allowed_types
            .add_user_type(i.ident.to_string(), CustomTypeKind::from(&i.attrs));
    }

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        syn::visit::visit_item_type(self, i);
        self.allowed_types
            .add_user_type(i.ident.to_string(), CustomTypeKind::from(&i.attrs));
    }
}

impl CustomTypesLister {
    pub fn new() -> Self {
        CustomTypesLister {
            allowed_types: AllowedRustTypes::new(vec![]),
        }
    }
}

pub fn get_custom_types(module: &ItemMod) -> AllowedRustTypes {
    let mut module = module.clone();
    let mut types_lister = CustomTypesLister::new();
    types_lister.visit_item_mod(&module);
    types_lister.allowed_types
}

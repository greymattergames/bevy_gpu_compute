// find all user declared types, and make a list of them

// ItemStruct.ident or  ItemType.ident

use quote::ToTokens;
use syn::{
    Ident, Item, ItemMod,
    visit::{self, Visit},
};

use crate::{state::ModuleTransformState, transformer::allowed_types::AllowedRustTypes};

use super::custom_type::{CustomType, CustomTypeKind};

struct CustomTypesLister {
    allowed_types: AllowedRustTypes,
}

impl<'ast> Visit<'ast> for CustomTypesLister {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        syn::visit::visit_item_struct(self, i);

        self.allowed_types.add_user_type(CustomType::new(
            &i.ident,
            CustomTypeKind::from(&i.attrs),
            i.to_token_stream(),
        ));
    }

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        syn::visit::visit_item_type(self, i);
        self.allowed_types.add_user_type(CustomType::new(
            &i.ident,
            CustomTypeKind::from(&i.attrs),
            i.to_token_stream(),
        ));
    }
}

impl CustomTypesLister {
    pub fn new() -> Self {
        CustomTypesLister {
            allowed_types: AllowedRustTypes::new(vec![]),
        }
    }
}

pub fn get_custom_types(state: &mut ModuleTransformState) {
    let mut types_lister = CustomTypesLister::new();
    types_lister.visit_item_mod(&state.rust_module);
    // println!("allowed types {:?}", types_lister.allowed_types);
    state.allowed_types = Some(types_lister.allowed_types);
}

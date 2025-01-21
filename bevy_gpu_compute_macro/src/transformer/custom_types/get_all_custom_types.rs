// find all user declared types, and make a list of them

// ItemStruct.ident or  ItemType.ident

use quote::ToTokens;
use syn::visit::Visit;

use crate::state::ModuleTransformState;

use super::custom_type::{CustomType, CustomTypeKind};

struct CustomTypesLister {
    custom_types: Vec<CustomType>,
}

impl<'ast> Visit<'ast> for CustomTypesLister {
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

impl CustomTypesLister {
    pub fn new() -> Self {
        CustomTypesLister {
            custom_types: vec![],
        }
    }
}

pub fn get_custom_types(state: &mut ModuleTransformState) {
    let mut types_lister = CustomTypesLister::new();
    types_lister.visit_item_mod(&state.rust_module);
    // println!("allowed types {:?}", types_lister.allowed_types);
    state.custom_types = Some(types_lister.custom_types);
}

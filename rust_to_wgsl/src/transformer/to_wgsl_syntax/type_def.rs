use quote::ToTokens;
use syn::{ItemType, parse_quote, visit_mut::VisitMut};

use crate::transformer::custom_types::custom_type::CustomType;

pub struct TypeDefToWgslTransformer {}

impl VisitMut for TypeDefToWgslTransformer {
    fn visit_item_type_mut(&mut self, t: &mut syn::ItemType) {
        syn::visit_mut::visit_item_type_mut(self, t);
        *t = type_def_to_wgsl(t);
    }
}

fn type_def_to_wgsl(type_def: &mut ItemType) -> ItemType {
    // type NAME = TYPE; becomes // alias NAME = TYPE;
    let s = format!(
        "alias {} = {};\n",
        type_def.ident.to_string(),
        type_def.ty.to_token_stream().to_string()
    );
    parse_quote!(#s)
}

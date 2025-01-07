use syn::ItemType;

use crate::transformer::{
    custom_types::custom_type::CustomType, to_wgsl_syntax::r#type::type_to_wgsl,
};

fn type_def_to_wgsl(type_def: &mut ItemType, custom_types: &Vec<CustomType>) -> String {
    // type NAME = TYPE; becomes // alias NAME = TYPE;
    format!(
        "alias {} = {};\n",
        type_def.ident.to_string(),
        type_to_wgsl(&type_def.ty, custom_types)
    )
}

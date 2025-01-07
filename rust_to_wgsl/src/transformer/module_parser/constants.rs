use shared::wgsl_components::{WgslConstAssignment, WgslShaderModuleUserPortion, WgslType};
use syn::{
    Ident, Item, ItemConst, ItemMod,
    visit::{self, Visit},
};

use crate::transformer::allowed_types::AllowedRustTypes;

struct ConstantsExtractor<'a> {
    result: &'a mut WgslShaderModuleUserPortion,
}

impl<'ast> Visit<'ast> for ConstantsExtractor<'ast> {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        syn::visit::visit_item_const(self, c);
        self.result.static_consts.push(parse_const_assignment(c));
    }
}

impl<'ast> ConstantsExtractor<'ast> {
    pub fn new(result: &'ast mut WgslShaderModuleUserPortion) -> Self {
        ConstantsExtractor { result }
    }
}

pub fn add_constants(module: &ItemMod, result: &mut WgslShaderModuleUserPortion) {
    let mut extractor = ConstantsExtractor::new(result);
    extractor.visit_item_mod(&module);
}

fn parse_const_assignment(constant: &ItemConst) -> WgslConstAssignment {
    WgslConstAssignment {
        assigner_keyword: "const".to_string(),
        var_name: constant.ident.to_string(),
        var_type: WgslType {
            name: format_type(&constant.ty),
            wgsl: type_to_wgsl(&constant.ty),
        },
        value: expr_to_wgsl(&constant.expr),
    }
}

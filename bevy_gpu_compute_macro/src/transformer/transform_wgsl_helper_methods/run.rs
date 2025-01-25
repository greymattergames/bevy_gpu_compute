use proc_macro_error::abort;
use proc_macro2::TokenStream;

use syn::{
    Expr, ExprCall, GenericArgument, PathArguments, Type, parse_quote,
    visit_mut::{self, VisitMut},
};

use crate::{
    state::ModuleTransformState,
    transformer::{
        custom_types::custom_type::CustomType,
        transform_wgsl_helper_methods::{
            helper_method::WgslHelperMethod, to_expanded_format::ToExpandedFormat,
        },
    },
};

use super::{
    category::WgslHelperCategory, matcher::WgslHelperMethodMatcher,
    method_name::WgslHelperMethodName,
};

fn get_special_function_category(call: &ExprCall) -> Option<WgslHelperCategory> {
    if let Expr::Path(path) = &*call.func {
        if let Some(first_seg) = path.path.segments.first() {
            return WgslHelperCategory::from_ident(first_seg.ident.clone());
        }
    }
    None
}
fn get_special_function_method(call: &ExprCall) -> Option<WgslHelperMethodName> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            return WgslHelperMethodName::from_ident(last_seg.ident.clone());
        }
    }
    None
}
fn get_special_function_generic_type<'a>(
    call: &'a ExprCall,
    custom_types: &'a [CustomType],
) -> Option<&'a CustomType> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                if let Some(GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
                    if let Some(last_seg) = type_path.path.segments.last() {
                        return custom_types.iter().find(|t| t.name.eq(&last_seg.ident));
                    }
                }
            }
        }
    }
    None
}

fn replace(call: ExprCall, custom_types: &[CustomType]) -> Option<TokenStream> {
    let category = get_special_function_category(&call);
    let method = get_special_function_method(&call);
    let type_name = get_special_function_generic_type(&call, custom_types);
    if let Some(cat) = category {
        if let Some(met) = method {
            if let Some(ty) = type_name {
                let mut method = WgslHelperMethod {
                    category: cat,
                    method: met,
                    t_def: ty,
                    arg1: call.args.first(),
                    arg2: call.args.get(1),
                    method_expander_kind: None,
                };
                WgslHelperMethodMatcher::choose_expand_format(&mut method);
                if method.method_expander_kind.is_some() {
                    let t = ToExpandedFormat::run(&method);
                    return Some(t);
                }
            }
        }
    }
    None
}

struct HelperFunctionConverter {
    custom_types: Vec<CustomType>,
}

impl VisitMut for HelperFunctionConverter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_mut::visit_expr_mut(self, expr);
        if let Expr::Call(call) = expr {
            let replacement = replace(call.clone(), &self.custom_types);
            if let Some(r) = replacement {
                *expr = parse_quote!(#r);
            }
        }
    }
}
impl HelperFunctionConverter {
    pub fn new(custom_types: &[CustomType]) -> Self {
        Self {
            custom_types: custom_types.to_vec(),
        }
    }
}

/// Rust's normal type checking will ensure that these helper functions are using correctly defined types
pub fn transform_wgsl_helper_methods(state: &mut ModuleTransformState) {
    assert!(
        state.custom_types.is_some(),
        "Allowed types must be defined"
    );
    let custom_types = if let Some(ct) = &state.custom_types {
        ct
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Allowed types must be set before transforming helper functions"
        );
    };
    let mut converter = HelperFunctionConverter::new(custom_types);
    converter.visit_item_mod_mut(&mut state.rust_module);
}

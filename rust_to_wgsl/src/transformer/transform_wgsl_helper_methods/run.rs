use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};

use syn::{
    AngleBracketedGenericArguments, Expr, ExprCall, ExprMethodCall, GenericArgument, Ident,
    ItemMod, Path, PathArguments, Type, parse_quote, parse_str,
    spanned::Spanned,
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
    custom_types: &'a Vec<CustomType>,
) -> Option<&'a CustomType> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                if let Some(GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
                    if let Some(last_seg) = type_path.path.segments.last() {
                        let name = last_seg.ident.to_string();
                        return custom_types.iter().find(|t| t.name.eq(&name));
                    }
                }
            }
        }
    }
    None
}

fn replace(call: ExprCall, custom_types: &Vec<CustomType>) -> Option<TokenStream> {
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
                if let Some(_) = &method.method_expander_kind {
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
    pub fn new(custom_types: &Vec<CustomType>) -> Self {
        Self {
            custom_types: custom_types.clone(),
        }
    }
}

/// Rust's normal type checking will ensure that these helper functions are using correctly defined types
pub fn transform_wgsl_helper_methods(state: &mut ModuleTransformState) {
    assert!(
        state.allowed_types.is_some(),
        "Allowed types must be defined"
    );
    let mut converter =
        HelperFunctionConverter::new(&state.allowed_types.as_ref().unwrap().custom_types);
    converter.visit_item_mod_mut(&mut state.rust_module);
}

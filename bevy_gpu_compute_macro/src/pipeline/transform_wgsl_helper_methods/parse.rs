use syn::{Expr, ExprCall, GenericArgument, PathArguments, Type};

use crate::pipeline::{
    custom_types::custom_type::CustomType,
    transform_wgsl_helper_methods::helper_method::WgslHelperMethod,
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

pub fn parse_possible_wgsl_helper<'a>(
    call: &'a ExprCall,
    custom_types: &'a [CustomType],
) -> Option<WgslHelperMethod> {
    let category = get_special_function_category(call);
    let method = get_special_function_method(call);
    let type_name = get_special_function_generic_type(call, custom_types);
    if let Some(cat) = category {
        if let Some(met) = method {
            if let Some(ty) = type_name {
                let args = call.args.clone();
                let mut method = WgslHelperMethod {
                    category: cat,
                    method: met,
                    t_def: ty.clone(),
                    arg1: args.first().cloned(),
                    arg2: args.get(1).cloned(),
                    method_expander_kind: None,
                };
                WgslHelperMethodMatcher::choose_expand_format(&mut method);
                if method.method_expander_kind.is_some() {
                    return Some(method);
                }
            }
        }
    }
    None
}

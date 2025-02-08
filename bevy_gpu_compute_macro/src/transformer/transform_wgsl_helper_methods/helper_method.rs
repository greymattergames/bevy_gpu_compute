use syn::Expr;

use crate::transformer::custom_types::custom_type::CustomType;

use super::{
    category::WgslHelperCategory, method_name::WgslHelperMethodName,
    to_expanded_format::ToExpandedFormatMethodKind,
};

pub struct WgslHelperMethod {
    pub category: WgslHelperCategory,
    pub method: WgslHelperMethodName,
    pub t_def: CustomType,
    pub arg1: Option<Expr>,
    pub arg2: Option<Expr>,
    pub method_expander_kind: Option<ToExpandedFormatMethodKind>,
}

use syn::Expr;

use crate::transformer::custom_types::custom_type::{CustomType, CustomTypeKind, CustomTypeName};

use super::{
    category::WgslHelperCategory, method_name::WgslHelperMethodName,
    to_expanded_format::ToExpandedFormatMethodKind,
};

pub struct WgslHelperMethod<'a> {
    pub category: WgslHelperCategory,
    pub method: WgslHelperMethodName,
    pub t_def: &'a CustomType,
    pub arg1: Option<&'a Expr>,
    pub arg2: Option<&'a Expr>,
    pub method_expander_kind: Option<ToExpandedFormatMethodKind>,
}

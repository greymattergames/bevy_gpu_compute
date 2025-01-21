use crate::transformer::{
    custom_types::custom_type::CustomTypeKind,
    transform_wgsl_helper_methods::to_expanded_format::ToExpandedFormatMethodKind,
};

use super::{
    category::WgslHelperCategory, helper_method::WgslHelperMethod,
    method_name::WgslHelperMethodName,
};

pub struct WgslHelperMethodMatcher {}
impl WgslHelperMethodMatcher {
    pub fn choose_expand_format(method: &mut WgslHelperMethod) {
        match (&method.category, &method.method) {
            (WgslHelperCategory::ConfigInput, WgslHelperMethodName::Get) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::Uniform,
                    "Expected {} to be an input config type, since WgslConfigInput::get is called, instead found it was of type {:?}. Put #[wgsl_config] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both a config and a input array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::ConfigGet);
            }
            (WgslHelperCategory::VecInput, WgslHelperMethodName::VecLen) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::InputArray,
                    "Expected {} to be an input array type, since WgslVecInput::vec_len is called, instead found it was of type {:?}. Put #[wgsl_input_array] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::InputLen);
            }
            (WgslHelperCategory::VecInput, WgslHelperMethodName::VecVal) => {
                assert!(
                    method.arg1.is_some(),
                    "Expected an argument for input vec value getter"
                );
                assert!(
                    method.t_def.kind == CustomTypeKind::InputArray,
                    "Expected {} to be an input array type, since WgslVecInput::vec_val is called, instead found it was of type {:?}. Put #[wgsl_input_array] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::InputVal);
            }
            (WgslHelperCategory::Output, WgslHelperMethodName::Push) => {
                assert!(
                    method.arg1.is_some(),
                    "Expected an argument for output push"
                );
                assert!(
                    method.t_def.kind == CustomTypeKind::OutputVec,
                    "Expected {} to be an output vec type, since WgslOutput::push is called, instead found it was of type {:?}. Put #[wgsl_output_vec] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array, an output cannot be both a vec and an array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::OutputPush);
            }
            (WgslHelperCategory::Output, WgslHelperMethodName::Len) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::OutputArray
                        || method.t_def.kind == CustomTypeKind::OutputVec,
                    "Expected {} to be an output array or vec type, since WgslOutput::len is called, instead found it was of type {:?}. Put #[wgsl_output_array] or #[wgsl_output_vec] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::OutputLen);
            }
            (WgslHelperCategory::Output, WgslHelperMethodName::Set) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::OutputArray,
                    "Expected {} to be an output array type, since WgslOutput::set is called, instead found it was of type {:?}. Put #[wgsl_output_array] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                assert!(
                    method.arg1.is_some() && method.arg2.is_some(),
                    "Expected two arguments for output set"
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::OutputSet);
            }
            _ => {
                method.method_expander_kind = None;
            }
        }
    }
}

use proc_macro_error::abort;
use quote::ToTokens;

use crate::state::ModuleTransformState;

use super::constants::find_constants;
use super::divide_custom_types::divide_custom_types_by_category;
use super::helper_functions::find_helper_functions;
use super::main_function::find_main_function;
use super::use_statements::handle_use_statements;
use super::validate_no_global_id_assignments::check_module_for_global_id_assignment;

pub fn parse_shader_module(state: &mut ModuleTransformState) {
    if state.rust_module.content.is_none() {
        abort!(
            state.rust_module.ident.span(),
            "Shader module must have a body"
        );
    }
    find_main_function(state);
    handle_use_statements(state);
    state.module_ident = Some(state.rust_module.ident.to_string());
    state.module_visibility = Some(state.rust_module.vis.to_token_stream().to_string());
    check_module_for_global_id_assignment(state);
    find_constants(state);
    divide_custom_types_by_category(state);
    find_helper_functions(state);
}

use crate::pipeline::{
    compilation_unit::CompilationUnit, compile_error::CompileError,
    phases::compiler_phase::CompilerPhase,
};
use quote::quote;

use super::{
    unaltered_module::generate_unaltered_module, user_facing_module::generate_user_facing_module,
};
pub struct FinalStructureGenerator;

impl CompilerPhase for FinalStructureGenerator {
    fn execute(&self, input: &mut CompilationUnit) -> Result<(), CompileError> {
        let unaltered_module_to_ensure_complete_rust_compiler_checks =
            generate_unaltered_module(input.original_rust_module());
        let user_facing_module = generate_user_facing_module(
            input.custom_types(),
            &mut input.wgsl_module_user_portion().clone(),
            input.rust_module_for_cpu(),
            input.typesafe_buffer_builders(),
        );
        input.set_compiled_tokens(quote!(
            #unaltered_module_to_ensure_complete_rust_compiler_checks

            #user_facing_module
        ));
        Ok(())
    }
}

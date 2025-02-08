use crate::pipeline::{
    compilation_unit::CompilationUnit, compile_error::CompileError,
    phases::compiler_phase::CompilerPhase,
};
use quote::quote;
pub struct FinalStructureGenerator;

impl CompilerPhase for FinalStructureGenerator {
    fn execute(&self, input: CompilationUnit) -> Result<CompilationUnit, CompileError> {
       
        let unaltered_module_to_ensure_complete_rust_compiler_checks = generate_unaltered_module(state);
        let user_facing_module = generate_user_facing_module(state);
        quote!(
            #unaltered_module
    
            #expanded_module
        )
        Ok(CompilationUnit {
            tokens: quote!{
            #unaltered_module_to_ensure_complete_rust_compiler_checks

            #user_facing_module
            }
            metadata: input.metadata,
        })
    }
}

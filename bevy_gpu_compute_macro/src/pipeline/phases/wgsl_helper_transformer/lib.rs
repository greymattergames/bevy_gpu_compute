use crate::pipeline::{
    compilation_unit::CompilationUnit, compile_error::CompileError,
    phases::compiler_phase::CompilerPhase,
};

pub struct WgslHelperTransformer;

impl CompilerPhase for WgslHelperTransformer {
    fn execute(&self, input: CompilationUnit) -> Result<CompilationUnit, CompileError> {
        let mut state = ModuleTransformState::new(input.ast.clone());
        find_custom_types(&mut state);
        Ok(CompilationUnit {
            ast: state.rust_module,
            metadata: input.metadata,
        })
    }
}

use crate::pipeline::{
    compilation_unit::CompilationUnit, compile_error::CompileError,
    phases::compiler_phase::CompilerPhase,
};

use super::lib::parse_shader_module_for_gpu;

pub struct GpuResourceMngmntAndWgslGenerator;

impl CompilerPhase for GpuResourceMngmntAndWgslGenerator {
    fn execute(&self, input: &mut CompilationUnit) -> Result<(), CompileError> {
        let (shader_module, custom_types) =
            parse_shader_module_for_gpu(input.rust_module_for_gpu(), input.custom_types());
        input.set_wgsl_module_user_portion(shader_module);
        input.set_custom_types(custom_types);
        Ok(())
    }
}

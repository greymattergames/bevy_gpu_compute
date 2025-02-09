use crate::pipeline::{compilation_unit::CompilationUnit, phases::compiler_phase::CompilerPhase};

use super::run::transform_wgsl_helper_methods;

pub struct WgslHelperTransformer;

impl CompilerPhase for WgslHelperTransformer {
    fn execute(&self, input: &mut CompilationUnit) {
        let mut mod_for_gpu = input.original_rust_module().clone();
        let mut mod_for_cpu = input.original_rust_module().clone();
        transform_wgsl_helper_methods(input.custom_types(), &mut mod_for_gpu, false);
        transform_wgsl_helper_methods(input.custom_types(), &mut mod_for_cpu, true);
        input.set_rust_module_for_cpu(mod_for_cpu);
        input.set_rust_module_for_gpu(mod_for_gpu);
    }
}

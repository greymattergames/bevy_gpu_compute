use crate::pipeline::phases::custom_type_collector::collect::collect_custom_types;
use crate::pipeline::{compilation_unit::CompilationUnit, phases::compiler_phase::CompilerPhase};

pub struct CustomTypeCollector;

impl CompilerPhase for CustomTypeCollector {
    fn execute(&self, input: &mut CompilationUnit) {
        let custom_types = collect_custom_types(input.original_rust_module());
        input.set_custom_types(custom_types);
    }
}

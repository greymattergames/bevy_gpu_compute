use super::{custom_types::custom_type::CustomType, phases::compiler_phase::CompilerPhase};

pub struct CompilerPipeline {
    phases: Vec<Box<dyn CompilerPhase>>,
}

impl Default for CompilerPipeline {
    fn default() -> Self {
        Self {
            phases: vec![
                CustomTypeCollector::default(),
                WgslHelperTransformer::default(),
                GpuResourceMngmntGenerator::default(),
                WgslCodeGenerator::default(),
                FinalStructureGenerator::default(),
            ],
        }
    }
}

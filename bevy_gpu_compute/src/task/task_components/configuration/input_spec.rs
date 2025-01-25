use bevy_gpu_compute_core::InputTypeMetadata;

#[derive(Clone, Default)]

pub struct InputSpec {
    arrays: Vec<InputTypeMetadata>,
    configs: Vec<InputTypeMetadata>,
}

impl InputSpec {
    pub fn new(arrays: Vec<InputTypeMetadata>, configs: Vec<InputTypeMetadata>) -> Self {
        InputSpec { arrays, configs }
    }
    pub fn arrays(&self) -> &Vec<InputTypeMetadata> {
        &self.arrays
    }
    pub fn configs(&self) -> &Vec<InputTypeMetadata> {
        &self.configs
    }
}
